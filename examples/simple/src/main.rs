//! Simple example for ferrunix, using manual registration.
//!
//! This example is inspired by the Guice example.

use std::error::Error;

use ferrunix::{Registry, Transient};

use self::traits::{
    BillingService, CreditCard, CreditCardProcessor, ExampleError, PizzaOrder,
    Receipt, TransactionLog,
};

mod traits;

/// An implementation of a credit card processr for PayPal.
#[derive(Debug, Default)]
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
#[derive(Debug, Default)]
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
pub struct RealBillingService {
    creditcard_processor: Box<dyn CreditCardProcessor>,
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
    let registry = Registry::empty();
    registry.register_transient::<Box<dyn CreditCardProcessor>, _>(|| {
        Box::new(PaypalCreditCardProcessor::default())
    });
    registry.register_transient::<Box<dyn TransactionLog>, _>(|| {
        Box::new(RealTransactionLog::default())
    });

    registry
        .with_deps::<Box<dyn BillingService>, (
            Transient<Box<dyn TransactionLog>>,
            Transient<Box<dyn CreditCardProcessor>>,
        )>()
        .register_transient(|(transaction, processor)| {
            Box::new(RealBillingService {
                transactionlog: transaction.get(),
                creditcard_processor: processor.get(),
            })
        });

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
