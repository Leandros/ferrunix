use ferrunix::{Registry, Transient};

use crate::common::*;

#[tokio::test]
async fn test_simple() {
    let registry = Registry::empty();
    registry.transient(|| Box::pin(async move { 1_u32 })).await;
    registry
        .with_deps::<_, (Transient<u32>,)>()
        .transient(|(x,)| {
            Box::pin(async move {
                let x = x.get();
                u64::from(x) + 2
            })
        })
        .await;
    registry.singleton(|| Box::pin(async move { 1_u64 })).await;
    // registry.singleton(|| async_ctor!(async move { 1_u64 })).await;

    let val = registry.get_transient::<u32>().await.unwrap();
    assert_eq!(val, 1);

    let val1 = registry.get_singleton::<u64>().await.unwrap();
    assert_eq!(*val1, 1);
}

#[derive(Debug, Default)]
pub struct PaypalCreditCardProcessor {}

#[async_trait::async_trait]
impl AsyncCreditCardProcessor for PaypalCreditCardProcessor {
    async fn charge(
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

#[async_trait::async_trait]
impl AsyncTransactionLog for RealTransactionLog {
    async fn log_charge(&self, amount: i32) {
        println!("charged {amount}");
    }

    async fn log_error(&self, err: &ExampleError) {
        eprintln!("error: charging creditcard: {err}");
    }
}

pub struct RealBillingService {
    creditcard_processor: Box<dyn AsyncCreditCardProcessor>,
    transactionlog: Box<dyn AsyncTransactionLog>,
}

#[async_trait::async_trait]
impl AsyncBillingService for RealBillingService {
    async fn charge_order(
        &self,
        order: PizzaOrder,
        creditcard: &CreditCard,
    ) -> Result<Receipt, ExampleError> {
        match self.creditcard_processor.charge(creditcard, order.0).await {
            Ok(charged_amount) => {
                self.transactionlog.log_charge(charged_amount).await;
                Ok(Receipt(charged_amount))
            }
            Err(err) => {
                self.transactionlog.log_error(&err).await;
                Err(err)
            }
        }
    }
}

#[tokio::test]
async fn test_more_complex() {
    let registry = Registry::empty();
    registry
        .transient::<Box<dyn AsyncCreditCardProcessor>>(|| {
            Box::pin(async move {
                Box::new(PaypalCreditCardProcessor::default())
                    as Box<dyn AsyncCreditCardProcessor>
            })
        })
        .await;
    registry
        .transient::<Box<dyn AsyncTransactionLog>>(|| {
            Box::pin(async move {
                Box::new(RealTransactionLog::default())
                    as Box<dyn AsyncTransactionLog>
            })
        })
        .await;
    registry.validate_all().unwrap();

    registry
        .with_deps::<Box<dyn AsyncBillingService>, (
            Transient<Box<dyn AsyncTransactionLog>>,
            Transient<Box<dyn AsyncCreditCardProcessor>>,
        )>()
        .transient(|(transaction, processor)| {
            Box::pin(async move {
                Box::new(RealBillingService {
                    transactionlog: transaction.get(),
                    creditcard_processor: processor.get(),
                }) as Box<dyn AsyncBillingService>
            })
        })
        .await;

    registry.validate_all().unwrap();

    let billing_service = registry
        .get_transient::<Box<dyn AsyncBillingService>>()
        .await
        .unwrap();

    let order = PizzaOrder(100);
    let creditcard = CreditCard {
        crc: "1234".to_owned(),
        expiry_year: 25,
        expiry_month: 11,
    };
    let result = billing_service.charge_order(order, &creditcard).await;

    result.unwrap();
}
