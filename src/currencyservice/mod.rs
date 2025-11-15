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
