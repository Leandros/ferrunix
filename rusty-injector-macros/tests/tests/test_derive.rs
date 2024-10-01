use super::common::*;
use rusty_injector::Registry;
use rusty_injector_macros::Inject;

pub trait CreditCardProcessor: Send + Sync {
    fn charge(&self, creditcard: &CreditCard, amount: i32) -> Result<i32, Error>;
}

#[derive(Debug, Default, Inject)]
#[provides(dyn TransactionLog)]
pub struct PaypalCreditCardProcessor {}

impl CreditCardProcessor for PaypalCreditCardProcessor {
    fn charge(&self, creditcard: &CreditCard, amount: i32) -> Result<i32, Error> {
        println!("charging {creditcard:?} for {amount} via PayPal");
        Ok(amount)
    }
}

pub trait TransactionLog: Send + Sync {
    fn log_charge(&self, amount: i32);
    fn log_error(&self, err: &Error);
}

#[derive(Debug, Default, Inject)]
#[provides(dyn TransactionLog)]
pub struct RealTransactionLog {}

impl TransactionLog for RealTransactionLog {
    fn log_charge(&self, amount: i32) {
        println!("charged {amount}");
    }

    fn log_error(&self, err: &Error) {
        eprintln!("error: charging creditcard: {err:?}");
    }
}

pub trait BillingService: Send + Sync {
    fn charge_order(&self, order: PizzaOrder, creditcard: &CreditCard) -> Result<Receipt, Error>;
}

#[derive(Inject)]
#[provides(dyn BillingService)]
pub struct RealBillingService {
    creditcard_processor: Box<dyn CreditCardProcessor>,
    transactionlog: Box<dyn TransactionLog>,
}

impl BillingService for RealBillingService {
    fn charge_order(&self, order: PizzaOrder, creditcard: &CreditCard) -> Result<Receipt, Error> {
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

/* --------- GENERATED --------- */

mod __inner_register_creditcardprocessor {
    #![allow(unused_imports)]
    use super::*;
    use rusty_injector::{inventory_submit, RegistrationFunc, Registry, Singleton, Transient};

    impl PaypalCreditCardProcessor {
        pub(crate) fn register(registry: &mut Registry) {
            registry.transient::<Box<dyn CreditCardProcessor>>(|| {
                Box::new(PaypalCreditCardProcessor::default())
            });
        }
    }

    inventory_submit!(RegistrationFunc(|registry| {
        PaypalCreditCardProcessor::register(registry);
    }));
}

mod __inner_register_transactionlog {
    #![allow(unused_imports)]
    use super::*;
    use rusty_injector::{inventory_submit, RegistrationFunc, Registry, Singleton, Transient};

    impl RealTransactionLog {
        pub(crate) fn register(registry: &mut Registry) {
            registry
                .transient::<Box<dyn TransactionLog>>(|| Box::new(RealTransactionLog::default()));
        }
    }

    inventory_submit!(RegistrationFunc(|registry| {
        RealTransactionLog::register(registry);
    }));
}

mod __inner_register_realbillingservice {
    #![allow(unused_imports)]
    use super::*;
    use rusty_injector::{inventory_submit, RegistrationFunc, Registry, Singleton, Transient};

    impl RealBillingService {
        pub(crate) fn register(registry: &mut Registry) {
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
        }
    }

    inventory_submit!(RegistrationFunc(|registry| {
        RealBillingService::register(registry);
    }));
}

pub fn test() {
    let registry = Registry::new();

    let billing_service = registry.get_transient::<Box<dyn BillingService>>().unwrap();

    let order = PizzaOrder(100);
    let creditcard = CreditCard {
        crc: "1234".to_owned(),
        expiry_year: 25,
        expiry_month: 11,
    };
    let result = billing_service.charge_order(order, &creditcard);

    assert!(result.is_ok());
}
