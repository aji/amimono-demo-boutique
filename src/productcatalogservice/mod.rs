use amimono::{Component, Rpc, RpcClient, Runtime};
use serde::{Deserialize, Serialize};

use crate::currencyservice::Money;

#[derive(Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub description: String,
    pub picture: String,
    pub price_usd: Money,
    pub categories: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub enum ProductCatalogRequest {
    ListProducts,
    GetProduct { id: String },
    SearchProducts { query: String },
}

#[derive(Serialize, Deserialize)]
pub enum ProductCatalogResponse {
    ListProducts { products: Vec<Product> },
    GetProduct { product: Product },
    SearchProducts { results: Vec<Product> },
}

pub struct ProductCatalogService;

impl Rpc for ProductCatalogService {
    const LABEL: amimono::Label = "productcatalogservice";

    type Request = ProductCatalogRequest;

    type Response = ProductCatalogResponse;

    async fn start(_rt: &Runtime) -> Self {
        ProductCatalogService
    }

    async fn handle(&self, _rt: &Runtime, _q: &Self::Request) -> Self::Response {
        todo!()
    }
}

pub async fn client(rt: &Runtime) -> RpcClient<ProductCatalogService> {
    ProductCatalogService::client(rt).await
}

pub fn component() -> Component {
    ProductCatalogService::component()
}
