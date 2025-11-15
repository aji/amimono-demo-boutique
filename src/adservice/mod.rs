use amimono::{Component, Rpc, RpcClient, Runtime};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AdServiceRequest {
    context_keys: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct AdServiceResponse {
    ads: Vec<Ad>,
}

#[derive(Serialize, Deserialize)]
pub struct Ad {
    redirect_url: String,
    text: String,
}

pub struct AdService;

impl Rpc for AdService {
    const LABEL: amimono::Label = "adservice";

    type Request = AdServiceRequest;

    type Response = AdServiceResponse;

    async fn start(_rt: &Runtime) -> Self {
        AdService
    }

    async fn handle(&self, _rt: &Runtime, _q: &Self::Request) -> Self::Response {
        todo!()
    }
}

pub async fn client(rt: &Runtime) -> RpcClient<AdService> {
    AdService::client(rt).await
}

pub fn component() -> Component {
    AdService::component()
}
