#![allow(clippy::unwrap_used, dead_code)]

use ferrunix::{Registry, Transient};

use crate::common::*;

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

#[test]
fn registry_dyn_traits() {
    let registry = Registry::empty();
    registry.transient::<Box<dyn CreditCardProcessor>, _>(|| {
        Box::new(PaypalCreditCardProcessor::default())
    });
    registry.transient::<Box<dyn TransactionLog>, _>(|| {
        Box::new(RealTransactionLog::default())
    });
    registry.validate_all().unwrap();

    registry
        .with_deps::<Box<dyn BillingService>, (
            Transient<Box<dyn TransactionLog>>,
            Transient<Box<dyn CreditCardProcessor>>,
        )>()
        .transient(|(transaction, processor)| {
            Box::new(RealBillingService {
                transactionlog: transaction.get(),
                creditcard_processor: processor.get(),
            })
        });

    registry.validate_all().unwrap();

    let billing_service =
        registry.get_transient::<Box<dyn BillingService>>().unwrap();

    let order = PizzaOrder(100);
    let creditcard = CreditCard {
        crc: "1234".to_owned(),
        expiry_year: 25,
        expiry_month: 11,
    };
    let result = billing_service.charge_order(order, &creditcard);

    result.unwrap();
}
