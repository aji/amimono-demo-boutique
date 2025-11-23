use crate::backend::ProductCatalogClient;
use amimono::{config::ComponentConfig, rpc::RpcResult};
use rand::seq::SliceRandom;

mod ops {
    amimono::rpc_ops! {
        fn list_recommendations(user_id: String, product_ids: Vec<String>) -> Vec<String>;
    }
}

pub struct RecommendationService {
    productcatalog: ProductCatalogClient,
}

const NUM_RECOMMENDATIONS: usize = 3;

impl ops::Handler for RecommendationService {
    async fn new() -> Self {
        RecommendationService {
            productcatalog: ProductCatalogClient::new(),
        }
    }

    async fn list_recommendations(
        &self,
        _user_id: String,
        _product_ids: Vec<String>,
    ) -> RpcResult<Vec<String>> {
        let mut products = self.productcatalog.list_products().await?;
        products.shuffle(&mut rand::rng());
        let ids = products
            .into_iter()
            .take(NUM_RECOMMENDATIONS)
            .map(|p| p.id)
            .collect::<Vec<_>>();
        Ok(ids)
    }
}

pub type RecommendationClient = ops::Client<RecommendationService>;

pub fn component() -> ComponentConfig {
    ops::component::<RecommendationService>("recommendationservice".to_string())
}
