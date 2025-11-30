use std::collections::HashMap;

use amimono::{config::ComponentConfig, rpc::RpcResult};
use amimono_haze::dashboard::tree;

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

impl CurrencyService {
    fn get_per_euro(&self, currency_code: &str) -> RpcResult<f64> {
        self.conversion
            .get(currency_code)
            .cloned()
            .ok_or_else(|| format!("unsupported currency: {}", currency_code).into())
    }
}

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
        let from_per_euro = self.get_per_euro(&from.currency_code)?;
        let to_per_euro = self.get_per_euro(&to)?;

        let from_nanos = from.units as f64 * 1_000_000_000.0 + from.nanos as f64;

        let to_units = 0i64;
        let to_nanos = (from_nanos * to_per_euro / from_per_euro) as i64;

        Ok(Money {
            currency_code: to.to_owned(),
            units: to_units + (to_nanos / 1_000_000_000),
            nanos: (to_nanos % 1_000_000_000) as i32,
        })
    }
}

pub type CurrencyClient = ops::Client<CurrencyService>;

pub fn component() -> ComponentConfig {
    ops::component::<CurrencyService>("currencyservice".to_string())
}

pub struct DashboardDirectory;

impl tree::Directory for DashboardDirectory {
    async fn list(&self) -> tree::TreeResult<Vec<tree::DirEntry>> {
        let res = CurrencyClient::new()
            .get_supported_currencies()
            .await?
            .into_iter()
            .map(tree::DirEntry::item)
            .collect();
        Ok(res)
    }

    async fn open_dir(&self, _name: &str) -> tree::TreeResult<tree::BoxDirectory> {
        Err(tree::TreeError::NotFound)
    }

    async fn open_item(&self, name: &str) -> tree::TreeResult<tree::Item> {
        let client = CurrencyClient::new();

        let one_in = Money {
            currency_code: name.to_owned(),
            units: 1,
            nanos: 0,
        };
        let one_eur = Money {
            currency_code: "EUR".to_owned(),
            units: 1,
            nanos: 0,
        };

        let as_in = client.convert(one_eur.clone(), name.to_owned()).await?;
        let as_eur = client.convert(one_in.clone(), "EUR".to_owned()).await?;

        let msg = format!("{one_eur:?} = {as_in:?}\n{one_in:?} = {as_eur:?}");
        Ok(tree::Item::new(msg))
    }
}
