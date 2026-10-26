// @@@SNIPSTART rust-money-transfer-project-template-withdraw
use std::sync::Arc;

use temporalio_macros::activities;
use temporalio_sdk::{
    ApplicationFailure,
    activities::{ActivityContext, ActivityError},
};
use tracing::error;

use crate::PaymentDetails;
use crate::banking_service::{BankingError, BankingErrorKind, BankingService};

pub struct BankingActivities {
    bank: BankingService,
}

impl BankingActivities {
    pub fn new() -> Self {
        Self {
            bank: BankingService::new("bank-api.example.com"),
        }
    }
}

impl Default for BankingActivities {
    fn default() -> Self {
        Self::new()
    }
}

// Activity names match the Python template's, so workers and workflows
// written with either SDK can share the task queue.
#[activities]
impl BankingActivities {
    #[activity(name = "withdraw")]
    pub async fn withdraw(
        self: Arc<Self>,
        _ctx: ActivityContext,
        data: PaymentDetails,
    ) -> Result<String, ActivityError> {
        let reference_id = format!("{}-withdrawal", data.reference_id);
        self.bank
            .withdraw(&data.source_account, data.amount, &reference_id)
            .map_err(|err| to_activity_error(err, "Withdrawal failed"))
    }

    // @@@SNIPEND
    // @@@SNIPSTART rust-money-transfer-project-template-deposit
    #[activity(name = "deposit")]
    pub async fn deposit(
        self: Arc<Self>,
        ctx: ActivityContext,
        data: PaymentDetails,
    ) -> Result<String, ActivityError> {
        let reference_id = format!("{}-deposit", data.reference_id);

        // Demo-only failure injection, driven by the DEMO_FAILURE env var on the
        // Worker. Unset/"off" leaves behavior unchanged.
        match std::env::var("DEMO_FAILURE")
            .unwrap_or_default()
            .to_lowercase()
            .as_str()
        {
            // Reuse the always-failing banking path for the first two attempts; the
            // error is retryable, so Temporal retries and the activity succeeds on
            // attempt 3 -> the Workflow recovers and completes.
            "transient" if ctx.info().attempt < 3 => {
                return self
                    .bank
                    .deposit_that_fails(&data.target_account, data.amount, &reference_id)
                    .map_err(|err| to_activity_error(err, "Deposit failed"));
            }
            // Reuse the always-failing banking path, but make it non-retryable so the
            // Workflow's refund compensation (saga rollback) runs instead of retrying.
            "permanent" => {
                let err = self
                    .bank
                    .deposit_that_fails(&data.target_account, data.amount, &reference_id)
                    .unwrap_err();
                error!("Deposit failed: {err}");
                return Err(ActivityError::application(
                    ApplicationFailure::builder(err)
                        .type_name("DepositFailure".to_string())
                        .non_retryable(true)
                        .build(),
                ));
            }
            _ => {}
        }

        let confirmation = self
            .bank
            .deposit(&data.target_account, data.amount, &reference_id);
        confirmation.map_err(|err| to_activity_error(err, "Deposit failed"))
    }

    // @@@SNIPEND

    // @@@SNIPSTART rust-money-transfer-project-template-refund
    #[activity(name = "refund")]
    pub async fn refund(
        self: Arc<Self>,
        _ctx: ActivityContext,
        data: PaymentDetails,
    ) -> Result<String, ActivityError> {
        let reference_id = format!("{}-refund", data.reference_id);
        self.bank
            .deposit(&data.source_account, data.amount, &reference_id)
            .map_err(|err| to_activity_error(err, "Refund failed"))
    }

    // @@@SNIPEND
}

/// Converts a banking error into an activity failure whose type name matches
/// the error kind, so the workflow's retry policy can mark it non-retryable.
/// Every error except an invalid account is logged before it is returned.
fn to_activity_error(err: BankingError, message: &str) -> ActivityError {
    let kind = err.kind();
    if kind != BankingErrorKind::InvalidAccount {
        error!("{message}: {err}");
    }
    ActivityError::application(
        ApplicationFailure::builder(err)
            .type_name(kind.as_str().to_string())
            .build(),
    )
}
