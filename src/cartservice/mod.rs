use amimono::{Component, Runtime};
use serde::{Deserialize, Serialize};

mod rpc;

pub use rpc::CartClient;

#[derive(Clone, Serialize, Deserialize)]
pub struct CartItem {
    pub product_id: String,
    pub quantity: u32,
}

#[derive(Serialize, Deserialize)]
pub struct Cart {
    pub user_id: String,
    pub items: Vec<CartItem>,
}

pub(in crate::cartservice) struct CartService;

impl CartService {
    async fn start(_rt: &Runtime) -> CartService {
        CartService
    }

    async fn add_item(&self, _rt: &Runtime, _user_id: &str, _item: &CartItem) -> () {
        todo!()
    }

    async fn get_cart(&self, _rt: &Runtime, _user_id: &str) -> Cart {
        todo!()
    }

    async fn empty_cart(&self, _rt: &Runtime, _user_id: &str) -> () {
        todo!()
    }
}

pub async fn client(rt: &Runtime) -> CartClient {
    rpc::CartClient::new(rt).await
}

pub fn component() -> Component {
    rpc::component()
}
