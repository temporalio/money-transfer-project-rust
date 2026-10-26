// @@@SNIPSTART rust-project-template-run-workflow
use std::error::Error;

use money_transfer::workflows::MoneyTransfer;
use money_transfer::{MONEY_TRANSFER_TASK_QUEUE_NAME, PaymentDetails};
use temporalio_client::envconfig::LoadClientConfigProfileOptions;
use temporalio_client::{
    Client, ClientOptions, WorkflowGetResultOptions, WorkflowStartOptions,
    errors::WorkflowGetResultError,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Connect to Temporal Cloud by loading the "cloud-setup" profile from the
    // shared Temporal client config (temporal.toml), which supplies the Cloud
    // address, namespace, TLS settings, and API key.
    let (mut conn_opts, client_opts) = ClientOptions::load_from_config(
        LoadClientConfigProfileOptions::builder()
            .config_file_profile("cloud-setup".to_string())
            .build(),
    )?;
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

    let workflow_id =
        std::env::var("WORKFLOW_ID").unwrap_or_else(|_| "money-transfer-demo".to_string());
    let handle = client
        .start_workflow(
            MoneyTransfer::run,
            data,
            WorkflowStartOptions::new(MONEY_TRANSFER_TASK_QUEUE_NAME, workflow_id).build(),
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
