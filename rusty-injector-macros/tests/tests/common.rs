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
