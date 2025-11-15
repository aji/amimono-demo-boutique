use amimono::{Component, Rpc, RpcClient, RpcHandler, Runtime};
use serde::{Deserialize, Serialize};

pub struct RecommendationService;

#[derive(Serialize, Deserialize)]
pub struct ListRecommendationsRequest {
    pub user_id: String,
    pub product_ids: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ListRecommendationsResponse {
    pub product_ids: Vec<String>,
}

impl Rpc for RecommendationService {
    const LABEL: amimono::Label = "recommendationservice";

    type Handler = Self;

    type Client = RpcClient<Self>;

    async fn start(_rt: &Runtime) -> Self {
        RecommendationService
    }
}

impl RpcHandler for RecommendationService {
    type Request = ListRecommendationsRequest;

    type Response = ListRecommendationsResponse;

    async fn handle(&self, _rt: &Runtime, _q: Self::Request) -> Self::Response {
        todo!()
    }
}

pub async fn client(rt: &Runtime) -> RpcClient<RecommendationService> {
    RecommendationService::client(rt).await
}

pub fn component() -> Component {
    RecommendationService::component()
}
