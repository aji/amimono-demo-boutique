use amimono::{Component, Rpc, RpcClient, Runtime};
use serde::{Deserialize, Serialize};

use super::{CurrencyService, Money};

#[derive(Serialize, Deserialize)]
pub enum CurrencyServiceRequest {
    GetSupportedCurrencies,
    Convert(Money, String),
}

#[derive(Serialize, Deserialize)]
pub enum CurrencyServiceResponse {
    GetSupportedCurrencies(Vec<String>),
    Convert(Money),
}

pub struct CurrencyServiceRpc(CurrencyService);

impl Rpc for CurrencyServiceRpc {
    const LABEL: amimono::Label = "currencyservice";

    type Request = CurrencyServiceRequest;

    type Response = CurrencyServiceResponse;

    async fn start(rt: &amimono::Runtime) -> Self {
        CurrencyServiceRpc(CurrencyService::start(rt).await)
    }

    async fn handle(&self, rt: &amimono::Runtime, q: &Self::Request) -> Self::Response {
        match q {
            CurrencyServiceRequest::GetSupportedCurrencies => {
                let a = self.0.get_supported_currencies(rt).await;
                CurrencyServiceResponse::GetSupportedCurrencies(a)
            }
            CurrencyServiceRequest::Convert(from, to) => {
                let a = self.0.convert(rt, from, to).await;
                CurrencyServiceResponse::Convert(a)
            }
        }
    }
}

#[derive(Clone)]
pub struct CurrencyClient(RpcClient<CurrencyServiceRpc>);

impl CurrencyClient {
    pub async fn new(rt: &Runtime) -> CurrencyClient {
        CurrencyClient(CurrencyServiceRpc::client(rt).await)
    }

    pub async fn get_supported_currencies(&self, rt: &Runtime) -> Result<Vec<String>, ()> {
        let q = CurrencyServiceRequest::GetSupportedCurrencies;
        match self.0.call(rt, &q).await {
            Ok(CurrencyServiceResponse::GetSupportedCurrencies(a)) => Ok(a),
            _ => Err(()),
        }
    }

    pub async fn convert(&self, rt: &Runtime, from: &Money, to: &str) -> Result<Money, ()> {
        let q = CurrencyServiceRequest::Convert(from.clone(), to.to_owned());
        match self.0.call(rt, &q).await {
            Ok(CurrencyServiceResponse::Convert(a)) => Ok(a),
            _ => Err(()),
        }
    }
}

pub fn component() -> Component {
    CurrencyServiceRpc::component()
}
