use rusty_injector::{LazyTransient, Registry, Transient};
use rusty_injector_macros::Inject;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
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

pub trait CreditCardProcessor: Send + Sync {
    fn charge(&self, creditcard: &CreditCard, amount: i32) -> Result<i32, Error>;
}

#[derive(Debug, Default)]
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

#[derive(Debug, Default)]
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

pub fn test() {
    let mut registry = Registry::new();
    registry.transient::<Box<dyn CreditCardProcessor>>(|| {
        Box::new(PaypalCreditCardProcessor::default())
    });
    registry.transient::<Box<dyn TransactionLog>>(|| Box::new(RealTransactionLog::default()));

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
