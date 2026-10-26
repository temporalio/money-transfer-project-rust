// @@@SNIPSTART rust-money-transfer-project-template-workflows
use std::time::Duration;

use temporalio_common::RetryPolicy;
use temporalio_macros::{workflow, workflow_methods};
use temporalio_sdk::{ActivityOptions, WorkflowContext, WorkflowResult};
use tracing::{error, info};

use crate::PaymentDetails;
use crate::activities::BankingActivities;
use crate::banking_service::BankingErrorKind;

#[workflow]
#[derive(Default)]
pub struct MoneyTransfer;

#[workflow_methods]
impl MoneyTransfer {
    #[run]
    pub async fn run(
        ctx: &mut WorkflowContext<Self>,
        payment_details: PaymentDetails,
    ) -> WorkflowResult<String> {
        let retry_policy = RetryPolicy::builder()
            .maximum_interval(Duration::from_secs(2))
            .non_retryable_error_types(
                BankingErrorKind::NON_RETRYABLE.map(BankingErrorKind::as_str),
            )
            .build();
        let activity_options = ActivityOptions::with_start_to_close_timeout(Duration::from_secs(5))
            .retry_policy(retry_policy)
            .build();

        let withdraw_id = ctx
            .execute_activity(
                BankingActivities::withdraw,
                payment_details.clone(),
                activity_options.clone(),
            )
            .await?;

        let deposit = ctx
            .execute_activity(
                BankingActivities::deposit,
                payment_details.clone(),
                activity_options.clone(),
            )
            .await;
        let deposit_err = match deposit {
            Ok(deposit_id) => {
                return Ok(format!(
                    "Transfer complete (transaction IDs: {withdraw_id}, {deposit_id})"
                ));
            }
            Err(err) => err,
        };
        // Like Python's workflow.logger, skip logging while replaying history,
        // so each message is written once rather than on every replay.
        if !ctx.is_replaying_history_events() {
            error!("Deposit failed: {deposit_err}");
        }

        // Put the withdrawn money back, then fail with the deposit error.
        let refund = ctx
            .execute_activity(BankingActivities::refund, payment_details, activity_options)
            .await;
        if !ctx.is_replaying_history_events() {
            match &refund {
                Ok(refund_id) => info!("Refund successful. Confirmation ID: {refund_id}"),
                Err(err) => error!("Refund failed: {err}"),
            }
        }
        refund?;

        Err(deposit_err.into())
    }
}
// @@@SNIPEND
