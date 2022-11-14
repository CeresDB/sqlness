use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

use tokio::fs::{read_dir, File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::Instant;
use walkdir::WalkDir;

use crate::case::TestCase;
use crate::error::{Result, SqlnessError};
use crate::{config::Config, environment::Environment};

/// The entrypoint of this crate.
///
/// To run your integration test cases, simply [`new`] a `Runner` and [`run`] it.
///
/// [`new`]: crate::Runner#method.new
/// [`run`]: crate::Runner#method.run
///
/// ```rust, no_run
/// async fn run_integration_test() {
///     let runner = Runner::new(root_path, env).await;
///     runner.run().await;
/// }
/// ```
///
/// For more detailed explaination, refer to crate level documentment.
pub struct Runner<E: Environment> {
    config: Config,
    env: Arc<E>,
}

impl<E: Environment> Runner<E> {
    pub async fn new<P: AsRef<Path>>(config_path: P, env: E) -> Result<Self> {
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
            env: Arc::new(env),
        })
    }

    pub async fn new_with_config(config: Config, env: E) -> Result<Self> {
        Ok(Self {
            config,
            env: Arc::new(env),
        })
    }

    pub async fn run(&self) -> Result<()> {
        let environments = self.collect_env().await?;
        for env in environments {
            self.run_env(env).await?;
        }

        Ok(())
    }

    async fn collect_env(&self) -> Result<Vec<String>> {
        let mut dirs = read_dir(&self.config.case_dir).await?;
        let mut result = vec![];

        while let Some(dir) = dirs.next_entry().await? {
            if dir.file_type().await?.is_dir() {
                let file_name = dir.file_name();
                result.push(file_name.to_str().unwrap().to_owned());
            }
        }

        Ok(result)
    }

    async fn run_env(&self, env: String) -> Result<()> {
        // todo: read env config
        let db = self.env.start(&env, None).await;
        let case_paths = self.collect_case_paths(&env).await?;
        let start = Instant::now();
        for path in case_paths {
            let case_path = path.with_extension(&self.config.test_case_extension);
            let case = TestCase::from_file(case_path, &self.config).await?;
            let output_path = path.with_extension(&self.config.output_result_extension);
            let mut output_file = Self::open_output_file(&output_path).await?;

            let timer = Instant::now();
            case.execute(&db, &mut output_file).await?;
            let elapsed = timer.elapsed();

            output_file.flush().await?;

            // todo: check diff

            println!(
                "Test case {:?} finished, cost: {}ms",
                path.as_os_str(),
                elapsed.as_millis()
            );
        }

        println!(
            "Environment {} run finished, cost:{}ms",
            env,
            start.elapsed().as_millis()
        );

        Ok(())
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
            .collect();

        // sort the cases in an os-independent order.
        cases.sort_by(|a, b| {
            let a_lower = a.to_string_lossy().to_lowercase();
            let b_lower = b.to_string_lossy().to_lowercase();
            a_lower.cmp(&b_lower)
        });

        Ok(cases)
    }

    async fn open_output_file<P: AsRef<Path>>(path: P) -> Result<File> {
        Ok(OpenOptions::default()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&path)
            .await?)
    }
}
