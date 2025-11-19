use std::collections::HashMap;

use amimono::{config::ComponentConfig, rpc::RpcResult};

use crate::shared::Money;

mod ops {
    use crate::shared::Money;

    amimono::rpc_ops! {
        fn get_supported_currencies() -> Vec<String>;
        fn convert(from: Money, to: String) -> Money;
    }
}

pub struct CurrencyService {
    conversion: HashMap<String, f64>,
}

const CURRENCY_CONVERSION_DATA: &'static str = include_str!("conversion.json");

impl ops::Handler for CurrencyService {
    async fn new() -> CurrencyService {
        let service = CurrencyService {
            conversion: serde_json::from_str(CURRENCY_CONVERSION_DATA).unwrap(),
        };
        log::debug!("loaded conversion data: {:?}", service.conversion);
        service
    }

    async fn get_supported_currencies(&self) -> RpcResult<Vec<String>> {
        Ok(self.conversion.keys().cloned().collect())
    }

    async fn convert(&self, from: Money, to: String) -> RpcResult<Money> {
        let from_per_euro = self.conversion.get(&from.currency_code).unwrap();
        let to_per_euro = self.conversion.get(&to).unwrap();

        let to_per_from = to_per_euro / from_per_euro;

        let to_units = (from.units as f64 * to_per_from) as i64;
        let to_nanos = (from.nanos as f64 * to_per_from) as i32;

        Ok(Money {
            currency_code: to.to_owned(),
            units: to_units + (to_nanos / 1_000_000_000) as i64,
            nanos: to_nanos % 1_000_000_000,
        })
    }
}

pub type CurrencyClient = ops::Client<CurrencyService>;

pub fn component() -> ComponentConfig {
    ops::component::<CurrencyService>("currencyservice".to_string())
}
