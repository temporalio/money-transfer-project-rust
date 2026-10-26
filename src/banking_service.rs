//! This code simulates a client for a hypothetical banking service.
//! It supports both withdrawals and deposits, and generates a random transaction ID for each request.
//!
//! Tip: You can modify these functions to introduce delays or errors, allowing
//! you to experiment with failures and timeouts.

use std::collections::HashMap;
use std::fmt;

use uuid::Uuid;

/// Errors returned by the mock banking service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BankingError {
    /// The account does not have enough funds to complete the transaction.
    InsufficientFunds { account_number: String },
    /// The account number is invalid.
    InvalidAccount { account_number: String },
    /// An unexpected error, such as the one returned by
    /// [`BankingService::deposit_that_fails`].
    Unknown(String),
}

impl BankingError {
    pub fn kind(&self) -> BankingErrorKind {
        match self {
            BankingError::InsufficientFunds { .. } => BankingErrorKind::InsufficientFunds,
            BankingError::InvalidAccount { .. } => BankingErrorKind::InvalidAccount,
            BankingError::Unknown(_) => BankingErrorKind::Unknown,
        }
    }
}

impl fmt::Display for BankingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BankingError::InsufficientFunds { account_number } => write!(
                f,
                "The account {account_number} has insufficient funds to complete this transaction."
            ),
            BankingError::InvalidAccount { account_number } => {
                write!(f, "The account number {account_number} is invalid.")
            }
            BankingError::Unknown(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for BankingError {}

/// The kind of a [`BankingError`], without its details.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BankingErrorKind {
    InsufficientFunds,
    InvalidAccount,
    Unknown,
}

impl BankingErrorKind {
    /// Errors that retrying can't fix.
    pub const NON_RETRYABLE: [Self; 2] = [Self::InvalidAccount, Self::InsufficientFunds];

    /// The error type name reported to Temporal, which retry policies match
    /// against.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InsufficientFunds => "InsufficientFundsError",
            Self::InvalidAccount => "InvalidAccountError",
            // The Python template's deposit_that_fails raises a ValueError.
            Self::Unknown => "ValueError",
        }
    }
}

/// A mock implementation of a banking API.
///
/// Provides methods for simulating deposits and withdrawals from bank
/// accounts, as well as a method for simulating a deposit that always fails.
#[derive(Debug, Clone)]
pub struct BankingService {
    hostname: String,
    /// Account balances, keyed by account number.
    accounts: HashMap<String, i64>,
}

impl BankingService {
    pub fn new(hostname: &str) -> Self {
        Self {
            hostname: hostname.to_string(),
            accounts: HashMap::from([("85-150".to_string(), 2000), ("43-812".to_string(), 0)]),
        }
    }

    /// The hostname of the banking API service.
    pub fn hostname(&self) -> &str {
        &self.hostname
    }

    /// Simulates a withdrawal from a bank account and returns a transaction ID.
    ///
    /// `reference_id` identifies the transaction, for idempotency.
    pub fn withdraw(
        &self,
        account_number: &str,
        amount: i64,
        _reference_id: &str,
    ) -> Result<String, BankingError> {
        if amount > self.balance(account_number)? {
            return Err(BankingError::InsufficientFunds {
                account_number: account_number.to_string(),
            });
        }

        Ok(transaction_id("W"))
    }

    /// Simulates a deposit to a bank account and returns a transaction ID.
    ///
    /// `reference_id` identifies the transaction, for idempotency.
    pub fn deposit(
        &self,
        account_number: &str,
        _amount: i64,
        _reference_id: &str,
    ) -> Result<String, BankingError> {
        self.balance(account_number)?;

        Ok(transaction_id("D"))
    }

    /// Simulates a deposit to a bank account that always fails with an
    /// unknown error.
    pub fn deposit_that_fails(
        &self,
        _account_number: &str,
        _amount: i64,
        _reference_id: &str,
    ) -> Result<String, BankingError> {
        Err(BankingError::Unknown(
            "This deposit has failed.".to_string(),
        ))
    }

    fn balance(&self, account_number: &str) -> Result<i64, BankingError> {
        self.accounts
            .get(account_number)
            .copied()
            .ok_or_else(|| BankingError::InvalidAccount {
                account_number: account_number.to_string(),
            })
    }
}

/// Generates a transaction ID; `prefix` identifies the type of transaction.
fn transaction_id(prefix: &str) -> String {
    format!("{prefix}-{}", Uuid::new_v4())
}
