# sqlness

[![Crates.io](https://img.shields.io/crates/v/sqlness.svg)](https://crates.io/crates/sqlness)
[![docs.rs](https://img.shields.io/docsrs/sqlness/latest)](https://docs.rs/sqlness)
![License](https://img.shields.io/badge/license-Apache--2.0-green.svg)
[![CI](https://github.com/CeresDB/sqlness/actions/workflows/ci.yml/badge.svg)](https://github.com/CeresDB/sqlness/actions/workflows/ci.yml)
[![OpenIssue](https://img.shields.io/github/issues/CeresDB/sqlness)](https://github.com/CeresDB/sqlness/issues)

**SQL** integration test har**NESS**

An ergonomic, opinionated framework for SQL integration test.

# Example

See [basic.rs](examples/basic.rs) to learn how to setup a basic test. This is the directory structure of examples

```
$ tree examples/
examples/
├── basic-case               # Testcase root directory
│   └── simple               # One environment
│       ├── config.toml      # Config file for current environment
│       ├── select.result    # Expected result file
│       └── select.sql       # Input SQL testcase
├── basic.rs                 # Entrypoint of this example
└── basic.toml               # Config of this example

```

When run it via
```bash
cargo run --example basic -- examples/basic.toml
```
It will do following things:
1. Collect all environments(first-level directory) under `basic-case`.
2. Run testcases(`.sql` files) under environment one after one.
   1. Write temporary result to `{testcase}.output`
   2. Compare `{testcase}.output` with `{testcase}.result` using `diff`
3. Report result.

Our target is to keep `*.result` file up to date, when `*.output` is equals to its corresponding result, the runner will delete it after executed.

When there is any diffs, the runner will keep `*.output` for later investigation.

Below is the output of this example:
```bash
Run testcase...
Start, env:simple, config:Some("examples/basic-case/simple/config.toml").
Test case "examples/basic-case/simple/select" finished, cost: 0ms
Environment simple run finished, cost:1ms
Stop, env:simple.
MyDB stopped.
```

# Who is using

- [CeresDB](https://github.com/CeresDB/ceresdb), a high-performance, distributed, cloud native time-series database that can handle both time-series and analytics workloads.
- [GreptimeDB](https://github.com/GreptimeTeam/greptimedb/), an open-source, cloud-native, distributed time-series database.

If you’re using sqlness and like to be added to this list, welcome to [open a PR](https://github.com/CeresDB/sqlness/pulls).

# License

This project is under [Apache License 2.0](./LICENSE).
