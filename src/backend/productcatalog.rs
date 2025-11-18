use amimono::config::ComponentConfig;
use serde::{Deserialize, Serialize};

use crate::shared::Product;

#[derive(Serialize, Deserialize)]
struct ProductCatalogData {
    products: Vec<Product>,
}

const PRODUCT_CATALOG_DATA: &'static str = include_str!("products.json");

mod ops {
    use crate::shared::Product;

    amimono::rpc_ops! {
        fn list_products() -> Vec<Product>;
        fn get_product(id: String) -> Product;
        fn search_products(query: String) -> Vec<Product>;
    }
}

pub struct ProductCatalogService {
    data: ProductCatalogData,
}

impl ops::Handler for ProductCatalogService {
    async fn new() -> ProductCatalogService {
        let data: ProductCatalogData = serde_json::from_str(PRODUCT_CATALOG_DATA).unwrap();
        log::debug!("catalog loaded: {:?}", data.products);
        ProductCatalogService { data }
    }

    async fn list_products(&self) -> Vec<Product> {
        log::debug!("list_products()");
        self.data.products.clone()
    }

    async fn get_product(&self, id: String) -> Product {
        log::debug!("get_product({id:?})");
        self.data
            .products
            .iter()
            .filter(|x| x.id == id)
            .next()
            .expect("no such product with ID")
            .clone()
    }

    async fn search_products(&self, query: String) -> Vec<Product> {
        log::debug!("search_products({query:?})");
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

pub type ProductCatalogClient = ops::Client<ProductCatalogService>;

pub fn component() -> ComponentConfig {
    ops::component::<ProductCatalogService>("productcatalogservice".to_owned())
}
