pub mod activities;
pub mod banking_service;
pub mod workflows;

// @@@SNIPSTART rust-money-transfer-project-template-shared
use serde::{Deserialize, Serialize};

pub const MONEY_TRANSFER_TASK_QUEUE_NAME: &str = "TRANSFER_MONEY_TASK_QUEUE";

/// Input to the money transfer workflow and its activities.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaymentDetails {
    pub source_account: String,
    pub target_account: String,
    /// Signed to match the Python template's `int`, so workflows started from
    /// either SDK can send the same values.
    pub amount: i64,
    pub reference_id: String,
}
// @@@SNIPEND
