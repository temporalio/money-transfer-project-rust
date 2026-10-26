// @@@SNIPSTART money-transfer-project-template-rust-tests
use money_transfer::PaymentDetails;
use money_transfer::activities::BankingActivities;
use money_transfer::banking_service::BankingErrorKind;
use money_transfer::workflows::MoneyTransfer;
use temporalio_client::{
    WorkflowGetResultOptions, WorkflowStartOptions, errors::WorkflowGetResultError,
};
use temporalio_common::error::IncomingError;
use temporalio_sdk::{
    Runtime, Worker, WorkerOptions,
    testing::{EphemeralExe, LocalWorkflowEnvironmentOptions, WorkflowEnvironment},
};
use uuid::Uuid;

/// Uses the `temporal` CLI on the PATH to run the local dev server when one is
/// installed; otherwise the SDK downloads it.
fn local_environment_options() -> LocalWorkflowEnvironmentOptions {
    let temporal_cli = std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|dir| dir.join(format!("temporal{}", std::env::consts::EXE_SUFFIX)))
            .find(|path| path.is_file())
    });
    match temporal_cli {
        Some(path) => LocalWorkflowEnvironmentOptions::builder()
            .server_executable(EphemeralExe::ExistingPath(
                path.to_string_lossy().into_owned(),
            ))
            .build(),
        None => LocalWorkflowEnvironmentOptions::default(),
    }
}

/// Starts a local dev server and a worker, runs the MoneyTransfer workflow
/// with `data`, then shuts the worker and server down.
async fn execute_money_transfer(data: PaymentDetails) -> Result<String, WorkflowGetResultError> {
    let task_queue_name = Uuid::new_v4().to_string();
    let env = WorkflowEnvironment::start_local(local_environment_options())
        .await
        .unwrap();
    let runtime = Runtime::from_current_tokio(Default::default()).unwrap();
    let worker_options = WorkerOptions::new(&task_queue_name)
        .register_workflow::<MoneyTransfer>()
        .unwrap()
        .register_activities(BankingActivities::new())
        .build();
    let mut worker = Worker::new(&runtime, env.client().clone(), worker_options).unwrap();
    let shutdown_worker = worker.shutdown_handle();

    let client = env.client().clone();
    let (worker_result, result) = tokio::join!(worker.run(), async move {
        let handle = client
            .start_workflow(
                MoneyTransfer::run,
                data,
                WorkflowStartOptions::new(task_queue_name, Uuid::new_v4().to_string()).build(),
            )
            .await
            .unwrap();
        let result = handle.get_result(WorkflowGetResultOptions::default()).await;
        shutdown_worker();
        result
    });
    worker_result.unwrap();
    env.shutdown().await.unwrap();
    result
}

/// Returns the type name of the application error that caused the activity
/// failure that failed the workflow.
fn failure_cause_type(err: &WorkflowGetResultError) -> Option<&str> {
    let WorkflowGetResultError::Failed(activity_err) = err else {
        return None;
    };
    match activity_err.cause()? {
        IncomingError::Application(app_err) => app_err.type_name(),
        _ => None,
    }
}

#[tokio::test]
async fn transfer_succeeds() {
    let data = PaymentDetails {
        source_account: "85-150".to_string(),
        target_account: "43-812".to_string(),
        amount: 250,
        reference_id: "12345".to_string(),
    };

    let result = execute_money_transfer(data).await.unwrap();
    assert!(result.starts_with("Transfer complete"));
}

#[tokio::test]
async fn withdraw_with_insufficient_funds_fails() {
    let data = PaymentDetails {
        source_account: "85-150".to_string(),
        target_account: "43-812".to_string(),
        amount: 25000,
        reference_id: "12345".to_string(),
    };

    let err = execute_money_transfer(data).await.unwrap_err();
    assert_eq!(
        failure_cause_type(&err),
        Some(BankingErrorKind::InsufficientFunds.as_str())
    );
}

// @@@SNIPEND
#[tokio::test]
async fn withdraw_from_invalid_account_fails() {
    let data = PaymentDetails {
        source_account: "85-151".to_string(),
        target_account: "43-812".to_string(),
        amount: 250,
        reference_id: "12345".to_string(),
    };

    let err = execute_money_transfer(data).await.unwrap_err();
    assert_eq!(
        failure_cause_type(&err),
        Some(BankingErrorKind::InvalidAccount.as_str())
    );
}

/// Workers and workflows from other SDKs, such as the Python template, find
/// these activities by name.
#[test]
fn activity_names_match_other_sdks() {
    assert_eq!(BankingActivities::withdraw.name(), "withdraw");
    assert_eq!(BankingActivities::deposit.name(), "deposit");
    assert_eq!(BankingActivities::refund.name(), "refund");
}
