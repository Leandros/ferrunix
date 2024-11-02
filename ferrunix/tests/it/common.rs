#![allow(dead_code)]

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExampleError {
    #[error("charge error")]
    ChargeError,
}

#[derive(Debug, Default)]
pub struct Receipt(pub i32);

#[derive(Debug, Default)]
pub struct PizzaOrder(pub i32);

#[derive(Debug, Default)]
pub struct CreditCard {
    pub crc: String,
    pub expiry_month: u16,
    pub expiry_year: u16,
}

//          ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
//          ┃                         Traits                          ┃
//          ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
// These traits are going to be implemented as part of the tests.
pub trait BillingService: Send + Sync {
    fn charge_order(
        &self,
        order: PizzaOrder,
        creditcard: &CreditCard,
    ) -> Result<Receipt, ExampleError>;
}

pub trait CreditCardProcessor: Send + Sync {
    fn charge(
        &self,
        creditcard: &CreditCard,
        amount: i32,
    ) -> Result<i32, ExampleError>;
}

pub trait TransactionLog: Send + Sync {
    fn log_charge(&self, amount: i32);
    fn log_error(&self, err: &ExampleError);
}

//          ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
//          ┃                      Async Traits                       ┃
//          ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

#[async_trait::async_trait]
pub trait AsyncBillingService: Send + Sync {
    async fn charge_order(
        &self,
        order: PizzaOrder,
        creditcard: &CreditCard,
    ) -> Result<Receipt, ExampleError>;
}

#[async_trait::async_trait]
pub trait AsyncCreditCardProcessor: Send + Sync {
    async fn charge(
        &self,
        creditcard: &CreditCard,
        amount: i32,
    ) -> Result<i32, ExampleError>;
}

#[async_trait::async_trait]
pub trait AsyncTransactionLog: Send + Sync {
    async fn log_charge(&self, amount: i32);
    async fn log_error(&self, err: &ExampleError);
}
