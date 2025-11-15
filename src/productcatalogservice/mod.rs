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

pub struct ProductCatalogClient(RpcClient<ProductCatalogService>);

impl ProductCatalogClient {
    pub async fn new(rt: &Runtime) -> ProductCatalogClient {
        ProductCatalogClient(ProductCatalogService::client(rt).await)
    }

    pub async fn list_products(&self, rt: &Runtime) -> Result<Vec<Product>, ()> {
        let q = ProductCatalogRequest::ListProducts;
        match self.0.call(rt, &q).await {
            Ok(ProductCatalogResponse::ListProducts { products }) => Ok(products),
            _ => Err(()),
        }
    }

    pub async fn get_product(&self, rt: &Runtime, id: &str) -> Result<Product, ()> {
        let q = ProductCatalogRequest::GetProduct { id: id.to_string() };
        match self.0.call(rt, &q).await {
            Ok(ProductCatalogResponse::GetProduct { product }) => Ok(product),
            _ => Err(()),
        }
    }

    pub async fn search_products(&self, rt: &Runtime, query: &str) -> Result<Vec<Product>, ()> {
        let q = ProductCatalogRequest::SearchProducts {
            query: query.to_string(),
        };
        match self.0.call(rt, &q).await {
            Ok(ProductCatalogResponse::SearchProducts { results }) => Ok(results),
            _ => Err(()),
        }
    }
}

pub async fn client(rt: &Runtime) -> RpcClient<ProductCatalogService> {
    ProductCatalogService::client(rt).await
}

pub fn component() -> Component {
    ProductCatalogService::component()
}
