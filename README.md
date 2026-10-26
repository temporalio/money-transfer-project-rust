# Temporal Money Transfer Rust Project Template

This is the code to support the tutorial at https://learn.temporal.io/getting_started/rust/first_program_in_rust/

Until that tutorial is published, see the [Temporal Rust SDK quickstart](https://docs.temporal.io/develop/rust/quickstart).

To use this code, make sure you have a [Temporal Service running](https://docs.temporal.io/develop/rust/quickstart) first.
For local development, install the Temporal CLI and run `temporal server start-dev`.

Clone this repo and run this application.

```bash
git clone https://github.com/temporalio/money-transfer-project-template-rust
cd money-transfer-project-template-rust
```

## Prerequisites

* [protobuf](https://protobuf.dev/). The Temporal Rust SDK generates its protocol code at build
  time, so `protoc` must be on your `PATH` before compiling.

  ```bash
  # macOS
  brew install protobuf

  # Debian/Ubuntu
  sudo apt-get install -y protobuf-compiler
  ```

Cargo installs the Temporal SDK the first time you build:

```bash
cargo build
```

## Configure the Temporal Client

By default, this template connects to a local Temporal server at
`localhost:7233` in the `default` namespace. To connect elsewhere, configure
the Temporal SDK with a `temporal.toml` profile or the supported environment
variables, such as `TEMPORAL_ADDRESS`, `TEMPORAL_NAMESPACE` and `TEMPORAL_API_KEY`.
See the [environment configuration docs](https://docs.temporal.io/develop/environment-configuration)
for available settings, including Temporal Cloud API keys.

When `TEMPORAL_PROFILE` is unset, the SDK uses `profile.default` from the
configuration file when present. An explicitly named profile must exist.

## Run the application

Run the workflow:

```bash
cargo run --bin starter
```

In another window, run the worker:

```bash
cargo run --bin worker
```

## Run the tests

The tests start their own local Temporal dev server. They use the `temporal`
CLI on your `PATH` when it is installed, and otherwise download it.

```bash
cargo test
```

## Project layout

| File | Purpose |
| --- | --- |
| `src/lib.rs` | Task queue name and the `PaymentDetails` input |
| `src/banking_service.rs` | Mock banking API with withdraw, deposit and a deposit that always fails, plus its error types |
| `src/activities.rs` | `withdraw`, `deposit` and `refund` activities |
| `src/workflows.rs` | `MoneyTransfer` workflow, which refunds the withdrawal if the deposit fails |
| `src/bin/worker.rs` | Runs the worker |
| `src/bin/starter.rs` | Starts the workflow and prints its result |
| `tests/money_transfer.rs` | Integration tests |

Please [read the tutorial](https://learn.temporal.io/getting_started/rust/first_program_in_rust/) for more details.
