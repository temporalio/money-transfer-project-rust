// @@@SNIPSTART rust-money-transfer-project-template-run-worker
use money_transfer::MONEY_TRANSFER_TASK_QUEUE_NAME;
use money_transfer::activities::BankingActivities;
use money_transfer::workflows::MoneyTransfer;
use temporalio_client::envconfig::LoadClientConfigProfileOptions;
use temporalio_client::{Client, ClientOptions};
use temporalio_sdk::{Runtime, Worker, WorkerOptions};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    let runtime = Runtime::from_current_tokio(Default::default())?;
    // Connect to Temporal Cloud by loading the "cloud-setup" profile from the
    // shared Temporal client config (temporal.toml), which supplies the Cloud
    // address, namespace, TLS settings, and API key.
    let (conn_opts, client_opts) = ClientOptions::load_from_config(
        LoadClientConfigProfileOptions::builder()
            .config_file_profile("cloud-setup".to_string())
            .build(),
    )?;
    let client = Client::connect(conn_opts, client_opts).await?;
    let worker_options = WorkerOptions::new(MONEY_TRANSFER_TASK_QUEUE_NAME)
        .register_workflow::<MoneyTransfer>()?
        .register_activities(BankingActivities::new())
        .build();
    let mut worker = Worker::new(&runtime, client, worker_options)?;
    worker.run().await?;

    Ok(())
}
// @@@SNIPEND
