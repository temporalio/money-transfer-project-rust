// @@@SNIPSTART rust-project-template-run-workflow
use std::error::Error;

use money_transfer::workflows::MoneyTransfer;
use money_transfer::{MONEY_TRANSFER_TASK_QUEUE_NAME, PaymentDetails};
use temporalio_client::{
    Client, ClientOptions, WorkflowGetResultOptions, WorkflowStartOptions,
    errors::WorkflowGetResultError,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Connection settings come from temporal.toml or TEMPORAL_* environment
    // variables, defaulting to localhost:7233 and the "default" namespace.
    let (mut conn_opts, client_opts) = ClientOptions::load_from_config(Default::default())?;
    // Record who started the workflow, as `{pid}@{hostname}` like the worker.
    if conn_opts.identity.is_empty() {
        conn_opts.identity = format!(
            "{}@{}",
            std::process::id(),
            gethostname::gethostname().to_string_lossy()
        );
    }
    let client = Client::connect(conn_opts, client_opts).await?;

    let data = PaymentDetails {
        source_account: "85-150".to_string(),
        target_account: "43-812".to_string(),
        amount: 250,
        reference_id: "12345".to_string(),
    };

    let handle = client
        .start_workflow(
            MoneyTransfer::run,
            data,
            WorkflowStartOptions::new(MONEY_TRANSFER_TASK_QUEUE_NAME, "pay-invoice-701").build(),
        )
        .await?;

    match handle.get_result(WorkflowGetResultOptions::default()).await {
        Ok(result) => println!("Result: {result}"),
        Err(err @ WorkflowGetResultError::Failed(_)) => report_failure(&err),
        Err(err) => return Err(err.into()),
    }

    Ok(())
}

/// Prints a failed workflow's error and the chain of errors that caused it.
fn report_failure(err: &dyn Error) {
    eprintln!("Transfer failed: {err}");
    let mut last_message = err.to_string();
    let mut source = err.source();
    while let Some(cause) = source {
        // The SDK wraps some errors in a layer that repeats the inner message.
        let message = cause.to_string();
        if !last_message.ends_with(&message) {
            eprintln!("  caused by: {message}");
            last_message = message;
        }
        source = cause.source();
    }
}
// @@@SNIPEND
