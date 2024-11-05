//! All traits for the simple example.
#![allow(unused)]

use thiserror::Error;

/// All errors that might happen.
#[derive(Debug, Error)]
pub enum ExampleError {
    /// An error during a transaction happened.
    #[error("charge error")]
    ChargeError,
}

/// Receipt of a successful charge.
#[derive(Debug, Default)]
pub struct Receipt(pub i32);

/// Order for a pizza, the inner `i32` defines the charged amount.
#[derive(Debug, Default)]
pub struct PizzaOrder(pub i32);

/// Representation of a credit card.
#[derive(Debug, Default)]
pub struct CreditCard {
    /// Credit card cyclic redunancy check.
    pub crc: String,
    /// Expiry month of the credit card.
    pub expiry_month: u16,
    /// Expiry year of the credit card.
    pub expiry_year: u16,
}

/// A billing service.
pub trait BillingService: Send + Sync {
    /// Charge `creditcard` with `order`.
    fn charge_order(
        &self,
        order: PizzaOrder,
        creditcard: &CreditCard,
    ) -> Result<Receipt, ExampleError>;
}

/// A credit card processor.
pub trait CreditCardProcessor: Send + Sync {
    /// Charge `amount` with `amount` and return the deducted amount, or an error.
    fn charge(
        &self,
        creditcard: &CreditCard,
        amount: i32,
    ) -> Result<i32, ExampleError>;
}

/// Transaction logging, with no defined output.
pub trait TransactionLog: Send + Sync {
    /// Log a successful charge.
    fn log_charge(&self, amount: i32);
    /// Log an error.
    fn log_error(&self, err: &ExampleError);
}
