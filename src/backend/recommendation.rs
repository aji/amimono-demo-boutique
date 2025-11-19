use amimono::{config::ComponentConfig, rpc::RpcResult};

mod ops {
    amimono::rpc_ops! {
        fn list_recommendations(user_id: String, product_ids: Vec<String>) -> Vec<String>;
    }
}

pub struct RecommendationService;

impl ops::Handler for RecommendationService {
    async fn new() -> Self {
        RecommendationService
    }

    async fn list_recommendations(
        &self,
        _user_id: String,
        _product_ids: Vec<String>,
    ) -> RpcResult<Vec<String>> {
        // TODO
        Ok(vec![])
    }
}

pub type RecommendationClient = ops::Client<RecommendationService>;

pub fn component() -> ComponentConfig {
    ops::component::<RecommendationService>("recommendationservice".to_string())
}
