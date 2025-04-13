//! Simple example for ferrunix, using the `derive` macro.
//!
//! This example is inspired by the Guice example.

use std::error::Error;

use ferrunix::{Inject, Registry};

use self::traits::{
    BillingService, CreditCard, CreditCardProcessor, ExampleError, PizzaOrder,
    Receipt, TransactionLog,
};

mod traits;

/// An implementation of a credit card processr for PayPal.
#[derive(Debug, Default, Inject)]
#[provides(transient = "dyn CreditCardProcessor")]
pub struct PaypalCreditCardProcessor {}

impl CreditCardProcessor for PaypalCreditCardProcessor {
    fn charge(
        &self,
        _creditcard: &CreditCard,
        amount: i32,
    ) -> Result<i32, ExampleError> {
        println!("charging {amount} via PayPal");
        Ok(amount)
    }
}

/// An implementation of a transaction log for stdout/stderr.
#[derive(Debug, Default, Inject)]
#[provides(transient = "dyn TransactionLog")]
pub struct RealTransactionLog {}

impl TransactionLog for RealTransactionLog {
    fn log_charge(&self, amount: i32) {
        println!("charged {amount}");
    }

    fn log_error(&self, err: &ExampleError) {
        eprintln!("error: charging creditcard: {err}");
    }
}

/// An implementation of a concrete billing service.
#[derive(Inject)]
#[provides(transient = "dyn BillingService")]
pub struct RealBillingService {
    #[inject(transient)]
    creditcard_processor: Box<dyn CreditCardProcessor>,
    #[inject(transient)]
    transactionlog: Box<dyn TransactionLog>,
}

impl BillingService for RealBillingService {
    fn charge_order(
        &self,
        order: PizzaOrder,
        creditcard: &CreditCard,
    ) -> Result<Receipt, ExampleError> {
        match self.creditcard_processor.charge(creditcard, order.0) {
            Ok(charged_amount) => {
                self.transactionlog.log_charge(charged_amount);
                Ok(Receipt(charged_amount))
            }
            Err(err) => {
                self.transactionlog.log_error(&err);
                Err(err)
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let registry = Registry::global();
    registry.validate_all_full()?;

    let billing_service =
        registry.transient::<Box<dyn BillingService>>().unwrap();

    let order = PizzaOrder(100);
    let creditcard = CreditCard {
        crc: "1234".to_owned(),
        expiry_year: 25,
        expiry_month: 11,
    };
    let receipt = billing_service.charge_order(order, &creditcard)?;

    println!("Receipt: {receipt:?}");

    Ok(())
}
