use amimono::{
    config::ComponentConfig,
    rpc::{RpcError, RpcResult},
};
use amimono_haze::dashboard::tree;
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

    async fn list_products(&self) -> RpcResult<Vec<Product>> {
        log::debug!("list_products()");
        Ok(self.data.products.clone())
    }

    async fn get_product(&self, id: String) -> RpcResult<Product> {
        log::debug!("get_product({id:?})");
        let res = self
            .data
            .products
            .iter()
            .filter(|x| x.id == id)
            .next()
            .ok_or(RpcError::Misc(format!("no such product with ID: {id}")))?
            .clone();
        Ok(res)
    }

    async fn search_products(&self, query: String) -> RpcResult<Vec<Product>> {
        log::debug!("search_products({query:?})");
        let query = query.to_lowercase();
        let res = self
            .data
            .products
            .iter()
            .filter(|x| {
                x.name.to_lowercase().contains(&query[..])
                    || x.description.to_lowercase().contains(&query[..])
            })
            .cloned()
            .collect();
        Ok(res)
    }
}

pub type ProductCatalogClient = ops::Client<ProductCatalogService>;

pub fn component() -> ComponentConfig {
    ops::component::<ProductCatalogService>("productcatalogservice".to_owned())
}

pub struct DashboardDirectory;

impl tree::Directory for DashboardDirectory {
    async fn list(&self) -> tree::TreeResult<Vec<tree::DirEntry>> {
        let its = ProductCatalogClient::new()
            .list_products()
            .await?
            .into_iter()
            .map(|it| tree::DirEntry::item(it.id))
            .collect();
        Ok(its)
    }

    async fn open_dir(&self, _name: &str) -> tree::TreeResult<tree::BoxDirectory> {
        Err(tree::TreeError::NotFound)
    }

    async fn open_item(&self, name: &str) -> tree::TreeResult<tree::Item> {
        let it = ProductCatalogClient::new()
            .get_product(name.to_owned())
            .await?;
        match serde_json::to_string_pretty(&it) {
            Ok(s) => Ok(tree::Item::new(s)),
            Err(e) => Err(tree::TreeError::Other(e.to_string())),
        }
    }
}
