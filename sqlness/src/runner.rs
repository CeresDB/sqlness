// Copyright 2022 CeresDB Project Authors. Licensed under Apache-2.0.

use std::fs::{read_dir, OpenOptions};
use std::io::{Cursor, Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use prettydiff::basic::{DiffOp, SliceChangeset};
use prettydiff::diff_lines;
use regex::Regex;
use walkdir::WalkDir;

use crate::case::TestCase;
use crate::error::{Result, SqlnessError};
use crate::{config::Config, environment::EnvController};

/// The entrypoint of this crate.
///
/// To run your integration test cases, simply [`new`] a `Runner` and [`run`] it.
///
/// [`new`]: crate::Runner#method.new
/// [`run`]: crate::Runner#method.run
///
/// ```rust, ignore, no_run
/// async fn run_integration_test() {
///     let runner = Runner::new(root_path, env).await;
///     runner.run().await;
/// }
/// ```
///
/// For more detailed explaination, refer to crate level documentment.
pub struct Runner<E: EnvController> {
    config: Config,
    env_controller: E,
}

impl<E: EnvController> Runner<E> {
    pub fn new(config: Config, env_controller: E) -> Self {
        Self {
            config,
            env_controller,
        }
    }

    pub async fn run(&self) -> Result<()> {
        let environments = self.collect_env()?;
        let mut errors = Vec::new();
        let filter = Regex::new(&self.config.env_filter)?;
        for env in environments {
            if !filter.is_match(&env) {
                println!("Environment({env}) is skipped!");
                continue;
            }
            let env_config = self.read_env_config(&env);
            let config_path = env_config.as_path();
            let config_path = if config_path.exists() {
                Some(config_path)
            } else {
                None
            };
            let parallelism = self.config.parallelism.max(1);
            let mut databases = Vec::with_capacity(parallelism);
            println!("Creating enviroment with parallelism: {}", parallelism);
            for id in 0..parallelism {
                let db = self.env_controller.start(&env, id, config_path).await;
                databases.push(db);
            }
            let run_result = self.run_env(&env, &databases).await;
            for db in databases {
                self.env_controller.stop(&env, db).await;
            }

            if let Err(e) = run_result {
                println!("Environment {env} run failed, error:{e:?}.");

                if self.config.fail_fast {
                    return Err(e);
                }

                errors.push(e);
            }
        }

        // only return first error
        if let Some(e) = errors.pop() {
            return Err(e);
        }

        Ok(())
    }

    fn read_env_config(&self, env: &str) -> PathBuf {
        let mut path_buf = std::path::PathBuf::new();
        path_buf.push(&self.config.case_dir);
        path_buf.push(env);
        path_buf.push(&self.config.env_config_file);

        path_buf
    }

    fn collect_env(&self) -> Result<Vec<String>> {
        let mut result = vec![];

        for dir in read_dir(&self.config.case_dir)? {
            let dir = dir?;
            if dir.file_type()?.is_dir() {
                let file_name = dir.file_name().to_str().unwrap().to_string();
                result.push(file_name);
            }
        }

        Ok(result)
    }

    async fn run_env(&self, env: &str, databases: &[E::DB]) -> Result<()> {
        let case_paths = self.collect_case_paths(env).await?;
        let start = Instant::now();

        let case_queue = Arc::new(Mutex::new(case_paths));
        let failed_cases = Arc::new(Mutex::new(Vec::new()));
        let errors = Arc::new(Mutex::new(Vec::new()));

        let mut futures = Vec::new();

        // Create futures for each database to process cases
        for (db_idx, db) in databases.iter().enumerate() {
            let case_queue = case_queue.clone();
            let failed_cases = failed_cases.clone();
            let errors = errors.clone();
            let fail_fast = self.config.fail_fast;

            futures.push(async move {
                loop {
                    // Try to get next case from the queue
                    let next_case = {
                        let mut queue = case_queue.lock().expect("Failed to lock case_queue mutex");
                        if queue.is_empty() {
                            break;
                        }
                        queue.pop().unwrap()
                    };

                    let case_name = next_case.as_os_str().to_str().unwrap().to_owned();
                    match self.run_single_case(db, &next_case).await {
                        Ok(false) => {
                            println!("[DB-{:2}] Case {} failed", db_idx, case_name);
                            failed_cases.lock().unwrap().push(case_name);
                        }
                        Ok(true) => {
                            println!("[DB-{:2}] Case {} succeeded", db_idx, case_name);
                        }
                        Err(e) => {
                            println!(
                                "[DB-{:2}] Case {} failed with error {:?}",
                                db_idx, case_name, e
                            );
                            if fail_fast {
                                errors.lock().expect("Failed to acquire lock on errors").push((case_name, e));
                                return;
                            }
                            errors.lock().expect("Failed to acquire lock on errors").push((case_name, e));
                        }
                    }
                }
            });
        }

        futures::future::join_all(futures).await;

        println!(
            "Environment {} run finished, cost:{}ms",
            env,
            start.elapsed().as_millis()
        );

        let failed_cases = failed_cases.lock().unwrap();
        if !failed_cases.is_empty() {
            println!("Failed cases:");
            println!("{failed_cases:#?}");
        }

        let errors = errors.lock().unwrap();
        if !errors.is_empty() {
            println!("Error cases:");
            println!("{errors:#?}");
        }

        let error_count = failed_cases.len() + errors.len();
        if error_count == 0 {
            Ok(())
        } else {
            Err(SqlnessError::RunFailed { count: error_count })
        }
    }

    /// Return true when this case pass, otherwise false.
    async fn run_single_case(&self, db: &E::DB, path: &Path) -> Result<bool> {
        let case_path = path.with_extension(&self.config.test_case_extension);
        let mut case = TestCase::from_file(&case_path, &self.config)?;
        let result_path = path.with_extension(&self.config.result_extension);
        let mut result_file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .truncate(false)
            .open(&result_path)?;

        // Read old result out for compare later
        let mut old_result = String::new();
        result_file.read_to_string(&mut old_result)?;

        // Execute testcase
        let mut new_result = Cursor::new(Vec::new());
        let timer = Instant::now();
        case.execute(db, &mut new_result).await?;
        let elapsed = timer.elapsed();

        // Truncate and write new result back
        result_file.set_len(0)?;
        result_file.rewind()?;
        result_file.write_all(new_result.get_ref())?;

        // Compare old and new result
        let new_result = String::from_utf8(new_result.into_inner()).expect("not utf8 string");
        if let Some(diff) = self.compare(&old_result, &new_result) {
            println!("Result unexpected, path:{case_path:?}");
            println!("{diff}");
            return Ok(false);
        }

        println!(
            "Test case {:?} finished, cost: {}ms",
            path.as_os_str(),
            elapsed.as_millis()
        );

        Ok(true)
    }

    async fn collect_case_paths(&self, env: &str) -> Result<Vec<PathBuf>> {
        let mut root = PathBuf::from_str(&self.config.case_dir).unwrap();
        root.push(env);

        let filter = Regex::new(&self.config.test_filter)?;
        let test_case_extension = self.config.test_case_extension.as_str();
        let mut cases: Vec<_> = WalkDir::new(&root)
            .follow_links(self.config.follow_links)
            .into_iter()
            .filter_map(|entry| {
                entry
                    .map_or(None, |entry| Some(entry.path().to_path_buf()))
                    .filter(|path| {
                        path.extension()
                            .map(|ext| ext == test_case_extension)
                            .unwrap_or(false)
                    })
            })
            .map(|path| path.with_extension(""))
            .filter(|path| {
                let filename = path
                    .file_name()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default();
                let filename_with_env = format!("{env}:{filename}");
                filter.is_match(&filename_with_env)
            })
            .collect();

        // sort the cases in an os-independent order.
        cases.sort_by(|a, b| {
            let a_lower = a.to_string_lossy().to_lowercase();
            let b_lower = b.to_string_lossy().to_lowercase();
            a_lower.cmp(&b_lower)
        });

        Ok(cases)
    }

    /// Compare result, return None if them are the same, else return diff changes
    fn compare(&self, expected: &str, actual: &str) -> Option<String> {
        let diff = diff_lines(expected, actual);
        let diff = diff.diff();
        let is_different = diff.iter().any(|d| !matches!(d, DiffOp::Equal(_)));
        if is_different {
            return Some(format!("{}", SliceChangeset { diff }));
        }

        None
    }
}
