//! **SQL** integration test har**NESS**
//!
//! ## Usage
//!
//! The entrypoint of this crate is [Runner] struct. It runs your test cases and
//! compare the result.
//!
//! ## Directory organization
//!
//! An example directory tree is:
//!
//! ```plaintext
//! sqlness
//! ├── local
//! │   ├── config.toml
//! │   ├── dml
//! │   │   └── basic.sql
//! │   ├── ddl
//! │   └── system
//! └── remote
//!     ├── config.toml
//!     ├── dml
//!     ├── ddl
//!     └── system
//! ```
//!
//! Here the root dir is `sqlness`, it contains two sub-directories for different
//! [`Envoronment`]s `local` and `remote`. Each environment has an env-specific
//! configuration `config.toml`. All the test cases are placed under corresponding
//! environment directory.
//!
//! Note that only the first layer of sub-directory is special (for distinguash
//! different environments). All deeper layers are treated as the same. E.g.,
//! both `sqlness/local/dml/basic.sql` and `sqlness/local/dml/another-dir/basic.sql`
//! will be run under the `local` in the same pass.

mod case;
mod config;
mod database;
mod environment;
mod error;
mod runner;

pub use config::Config;
pub use database::Database;
pub use environment::Environment;
pub use error::SqlnessError;
pub use runner::Runner;
