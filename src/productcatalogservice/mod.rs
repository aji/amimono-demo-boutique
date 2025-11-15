use amimono::{Component, Rpc, RpcClient, RpcHandler, Runtime};
use serde::{Deserialize, Serialize};

use crate::currencyservice::Money;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Serialize, Deserialize)]
struct ProductCatalogData {
    products: Vec<Product>,
}

const PRODUCT_CATALOG_DATA: &'static str = include_str!("products.json");

pub struct ProductCatalogService {
    data: ProductCatalogData,
}

impl ProductCatalogService {
    async fn new() -> ProductCatalogService {
        let data: ProductCatalogData = serde_json::from_str(PRODUCT_CATALOG_DATA).unwrap();
        log::debug!("catalog loaded: {:?}", data.products);
        ProductCatalogService { data }
    }

    async fn list_products(&self) -> Vec<Product> {
        self.data.products.clone()
    }

    async fn get_product(&self, id: &str) -> Product {
        self.data
            .products
            .iter()
            .filter(|x| x.id == id)
            .next()
            .expect("no such product with ID")
            .clone()
    }

    async fn search_products(&self, query: &str) -> Vec<Product> {
        let query = query.to_lowercase();
        self.data
            .products
            .iter()
            .filter(|x| {
                x.name.to_lowercase().contains(&query[..])
                    || x.description.to_lowercase().contains(&query[..])
            })
            .cloned()
            .collect()
    }
}

impl Rpc for ProductCatalogService {
    const LABEL: amimono::Label = "productcatalogservice";

    type Handler = Self;
    type Client = RpcClient<Self>;

    async fn start(_rt: &Runtime) -> Self {
        ProductCatalogService::new().await
    }
}

impl RpcHandler for ProductCatalogService {
    type Request = ProductCatalogRequest;
    type Response = ProductCatalogResponse;

    async fn handle(&self, _rt: &Runtime, q: Self::Request) -> Self::Response {
        match q {
            ProductCatalogRequest::ListProducts => {
                let products = self.list_products().await;
                ProductCatalogResponse::ListProducts { products }
            }
            ProductCatalogRequest::GetProduct { id } => {
                let product = self.get_product(&id).await;
                ProductCatalogResponse::GetProduct { product }
            }
            ProductCatalogRequest::SearchProducts { query } => {
                let results = self.search_products(&query).await;
                ProductCatalogResponse::SearchProducts { results }
            }
        }
    }
}

pub struct ProductCatalogClient(RpcClient<ProductCatalogService>);

impl ProductCatalogClient {
    pub async fn new(rt: &Runtime) -> ProductCatalogClient {
        ProductCatalogClient(ProductCatalogService::client(rt).await)
    }

    pub async fn list_products(&self, rt: &Runtime) -> Result<Vec<Product>, ()> {
        let q = ProductCatalogRequest::ListProducts;
        match self.0.handle(rt, q).await {
            Ok(ProductCatalogResponse::ListProducts { products }) => Ok(products),
            _ => Err(()),
        }
    }

    pub async fn get_product(&self, rt: &Runtime, id: &str) -> Result<Product, ()> {
        let q = ProductCatalogRequest::GetProduct { id: id.to_string() };
        match self.0.handle(rt, q).await {
            Ok(ProductCatalogResponse::GetProduct { product }) => Ok(product),
            _ => Err(()),
        }
    }

    pub async fn search_products(&self, rt: &Runtime, query: &str) -> Result<Vec<Product>, ()> {
        let q = ProductCatalogRequest::SearchProducts {
            query: query.to_string(),
        };
        match self.0.handle(rt, q).await {
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
