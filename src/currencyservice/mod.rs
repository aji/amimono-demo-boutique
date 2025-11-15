use std::{
    iter::Sum,
    ops::{Add, Mul},
};

use amimono::{Component, Runtime};
use serde::{Deserialize, Serialize};

mod rpc;

pub use rpc::CurrencyClient;

#[derive(Clone, Serialize, Deserialize)]
pub struct Money {
    pub currency_code: String,
    pub units: i64,
    pub nanos: i32,
}

impl Default for Money {
    fn default() -> Self {
        Self {
            currency_code: "USD".to_owned(),
            units: 0,
            nanos: 0,
        }
    }
}

impl Add<Money> for Money {
    type Output = Money;
    fn add(self, rhs: Money) -> Self::Output {
        if self.currency_code != rhs.currency_code {
            panic!(
                "attempted to add currencies of different types ({} and {})",
                self.currency_code, rhs.currency_code
            );
        }
        let units = self.units + rhs.units;
        let nanos = self.nanos + rhs.nanos;
        Money {
            currency_code: self.currency_code,
            units: units + (nanos / 1_000_000_000) as i64,
            nanos: nanos % 1_000_000_000,
        }
    }
}

impl Sum<Money> for Money {
    fn sum<I: Iterator<Item = Money>>(mut it: I) -> Self {
        let mut tot = match it.next() {
            Some(x) => x,
            None => return Default::default(),
        };
        for x in it {
            tot = tot + x
        }
        tot
    }
}

impl Mul<Money> for u32 {
    type Output = Money;
    fn mul(self, rhs: Money) -> Self::Output {
        let nanos = (self as i32) * rhs.nanos;
        let units = (self as i64) * rhs.units;
        Money {
            currency_code: rhs.currency_code,
            units: units + (nanos / 1_000_000_000) as i64,
            nanos: nanos % 1_000_000_000,
        }
    }
}

pub(in crate::currencyservice) struct CurrencyService {
    supported_currencies: Vec<String>,
}

impl CurrencyService {
    async fn start(_rt: &Runtime) -> CurrencyService {
        CurrencyService {
            supported_currencies: vec!["USD".to_owned(), "JPY".to_owned(), "EUR".to_owned()],
        }
    }

    async fn get_supported_currencies(&self, _rt: &Runtime) -> Vec<String> {
        self.supported_currencies.clone()
    }

    async fn convert(&self, _rt: &Runtime, _from: &Money, _to: &str) -> Money {
        todo!()
    }
}

pub async fn client(rt: &Runtime) -> CurrencyClient {
    CurrencyClient::new(rt).await
}

pub fn component() -> Component {
    rpc::component()
}
