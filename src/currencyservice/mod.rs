use std::{
    collections::HashMap,
    fmt,
    iter::Sum,
    ops::{Add, Mul},
};

use amimono::{Component, Runtime};
use serde::{Deserialize, Serialize};

mod rpc;

pub use rpc::CurrencyClient;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Money {
    pub currency_code: String,
    pub units: i64,
    pub nanos: i32,
}

impl Money {
    fn normalize(self) -> Money {
        Money {
            currency_code: self.currency_code,
            units: self.units + (self.nanos / 1_000_000_000) as i64,
            nanos: self.nanos % 1_000_000_000,
        }
    }

    pub fn from_usd(dollars: i64, cents: i32) -> Money {
        let res = Money {
            currency_code: "USD".to_owned(),
            units: dollars,
            nanos: cents * 10_000_000,
        };
        res.normalize()
    }
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

impl fmt::Debug for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Money({} {}.{:09})",
            self.currency_code, self.units, self.nanos
        )
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
        let res = Money {
            currency_code: self.currency_code,
            units: self.units + rhs.units,
            nanos: self.nanos + rhs.nanos,
        };
        res.normalize()
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
        let res = Money {
            currency_code: rhs.currency_code,
            units: (self as i64) * rhs.units,
            nanos: (self as i32) * rhs.nanos,
        };
        res.normalize()
    }
}

pub(in crate::currencyservice) struct CurrencyService {
    conversion: HashMap<String, f64>,
}

const CURRENCY_CONVERSION_DATA: &'static str = include_str!("conversion.json");

impl CurrencyService {
    async fn start(_rt: &Runtime) -> CurrencyService {
        let service = CurrencyService {
            conversion: serde_json::from_str(CURRENCY_CONVERSION_DATA).unwrap(),
        };
        log::debug!("loaded conversion data: {:?}", service.conversion);
        service
    }

    async fn get_supported_currencies(&self, _rt: &Runtime) -> Vec<String> {
        self.conversion.keys().cloned().collect()
    }

    async fn convert(&self, _rt: &Runtime, from: &Money, to: &str) -> Money {
        let from_per_euro = self.conversion.get(&from.currency_code).unwrap();
        let to_per_euro = self.conversion.get(to).unwrap();

        let to_per_from = to_per_euro / from_per_euro;

        let to_units = (from.units as f64 * to_per_from) as i64;
        let to_nanos = (from.nanos as f64 * to_per_from) as i32;

        Money {
            currency_code: to.to_owned(),
            units: to_units + (to_nanos / 1_000_000_000) as i64,
            nanos: to_nanos % 1_000_000_000,
        }
    }
}

pub async fn client(rt: &Runtime) -> CurrencyClient {
    CurrencyClient::new(rt).await
}

pub fn component() -> Component {
    rpc::component()
}
