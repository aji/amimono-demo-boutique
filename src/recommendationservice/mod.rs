use amimono::{Component, Runtime};

mod ops {
    amimono::rpc_ops! {
        fn list_recommendations(user_id: String, product_ids: Vec<String>) -> Vec<String>;
    }
}

pub struct RecommendationService;

impl ops::Handler for RecommendationService {
    const LABEL: amimono::Label = "recommendationservice";

    async fn new(_rt: &Runtime) -> Self {
        RecommendationService
    }

    async fn list_recommendations(
        &self,
        _rt: &amimono::Runtime,
        _user_id: String,
        _product_ids: Vec<String>,
    ) -> Vec<String> {
        // TODO
        vec![]
    }
}

pub type RecommendationClient = ops::RpcClient<RecommendationService>;

pub fn component() -> Component {
    ops::component::<RecommendationService>()
}
