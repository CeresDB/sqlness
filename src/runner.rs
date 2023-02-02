// Copyright 2022 CeresDB Project Authors. Licensed under Apache-2.0.

use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use prettydiff::basic::{DiffOp, SliceChangeset};
use prettydiff::diff_lines;
use tokio::fs::{read_dir, File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::Instant;
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
    pub async fn try_new<P: AsRef<Path>>(config_path: P, env_controller: E) -> Result<Self> {
        let mut config_file =
            File::open(config_path.as_ref())
                .await
                .map_err(|e| SqlnessError::ReadPath {
                    source: e,
                    path: config_path.as_ref().to_path_buf(),
                })?;

        let mut config_buf = vec![];
        config_file.read_to_end(&mut config_buf).await?;
        let config: Config =
            toml::from_slice(&config_buf).map_err(|e| SqlnessError::ParseToml {
                source: e,
                file: config_path.as_ref().to_path_buf(),
            })?;

        Ok(Self {
            config,
            env_controller,
        })
    }

    pub async fn new_with_config(config: Config, env_controller: E) -> Result<Self> {
        Ok(Self {
            config,
            env_controller,
        })
    }

    pub async fn run(&self) -> Result<()> {
        let environments = self.collect_env().await?;
        let mut errors = Vec::new();
        for env in environments {
            let env_config = self.read_env_config(&env).await;
            let config_path = env_config.as_path();
            let config_path = if config_path.exists() {
                Some(config_path)
            } else {
                None
            };
            let db = self.env_controller.start(&env, config_path).await;
            let run_result = self.run_env(&env, &db).await;
            self.env_controller.stop(&env, db).await;

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

    async fn read_env_config(&self, env: &str) -> PathBuf {
        let mut path_buf = std::path::PathBuf::new();
        path_buf.push(&self.config.case_dir);
        path_buf.push(env);
        path_buf.push(&self.config.env_config_file);

        path_buf
    }

    async fn collect_env(&self) -> Result<Vec<String>> {
        let mut dirs = read_dir(&self.config.case_dir).await?;
        let mut result = vec![];

        while let Some(dir) = dirs.next_entry().await? {
            if dir.file_type().await?.is_dir() {
                let file_name = dir.file_name().to_str().unwrap().to_string();
                result.push(file_name);
            }
        }

        Ok(result)
    }

    async fn run_env(&self, env: &str, db: &E::DB) -> Result<()> {
        let case_paths = self.collect_case_paths(env).await?;
        let mut diff_cases = vec![];
        let mut errors = vec![];
        let start = Instant::now();
        for path in case_paths {
            let case_result = self.run_single_case(db, &path).await;
            let case_name = path.as_os_str().to_str().unwrap().to_owned();
            match case_result {
                Ok(true) => diff_cases.push(case_name),
                Ok(false) => {}
                Err(e) => {
                    if self.config.fail_fast {
                        println!("Case {case_name} failed with error {e:?}");
                        println!("Stopping environment {env} due to previous error.");
                        break;
                    } else {
                        errors.push((case_name, e))
                    }
                }
            }
        }

        println!(
            "Environment {} run finished, cost:{}ms",
            env,
            start.elapsed().as_millis()
        );

        let mut error_count = 0;
        if !diff_cases.is_empty() {
            println!("Different cases:");
            println!("{diff_cases:#?}");
            error_count += diff_cases.len();
        }
        if !errors.is_empty() {
            println!("Error cases:");
            println!("{errors:#?}");
            error_count += errors.len();
        }
        if error_count == 0 {
            Ok(())
        } else {
            Err(SqlnessError::RunFailed { count: error_count })
        }
    }

    async fn run_single_case(&self, db: &E::DB, path: &Path) -> Result<bool> {
        let case_path = path.with_extension(&self.config.test_case_extension);
        let case = TestCase::from_file(&case_path, &self.config).await?;
        let result_path = path.with_extension(&self.config.result_extension);
        let mut result_file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(&result_path)
            .await?;

        // Read old result out for compare later
        let mut old_result = String::new();
        result_file.read_to_string(&mut old_result).await?;

        // Execute testcase
        let mut new_result = Cursor::new(Vec::new());
        let timer = Instant::now();
        case.execute(db, &mut new_result).await?;
        let elapsed = timer.elapsed();

        // Truncate and write new result back
        let mut result_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&result_path)
            .await?;
        result_file.write_all(new_result.get_ref()).await?;

        // Compare old and new result
        let new_result = String::from_utf8(new_result.into_inner()).expect("not utf8 string");
        if let Some(diff) = self.compare(&old_result, &new_result) {
            println!("Result unexpected, path:{case_path:?}");
            println!("{diff}");
            return Ok(true);
        }

        println!(
            "Test case {:?} finished, cost: {}ms",
            path.as_os_str(),
            elapsed.as_millis()
        );
        Ok(false)
    }

    async fn collect_case_paths(&self, env: &str) -> Result<Vec<PathBuf>> {
        let mut root = PathBuf::from_str(&self.config.case_dir).unwrap();
        root.push(env);

        let test_case_extension = self.config.test_case_extension.as_str();
        let mut cases: Vec<_> = WalkDir::new(&root)
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
                path.file_name()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default()
                    .contains(&self.config.test_filter)
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
