use std::{collections::HashMap, sync::Arc};

use amimono::{Component, Runtime};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::shared::CartItem;

#[derive(Serialize, Deserialize)]
pub struct Cart {
    pub user_id: String,
    pub items: Vec<CartItem>,
}

mod ops {
    use crate::{cartservice::Cart, shared::CartItem};

    amimono::rpc_ops! {
        fn add_item(user_id: String, item: CartItem) -> ();
        fn get_cart(user_id: String) -> Cart;
        fn empty_cart(user_id: String) -> ();
    }
}

pub struct CartService {
    carts: Arc<Mutex<HashMap<String, Vec<CartItem>>>>,
}

impl ops::Handler for CartService {
    const LABEL: amimono::Label = "cartservice";

    async fn new(_rt: &Runtime) -> CartService {
        CartService {
            carts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn add_item(&self, _rt: &Runtime, user_id: String, item: CartItem) -> () {
        self.carts
            .lock()
            .await
            .entry(user_id)
            .or_insert(Vec::new())
            .push(item.clone());
    }

    async fn get_cart(&self, _rt: &Runtime, user_id: String) -> Cart {
        let items = self
            .carts
            .lock()
            .await
            .get(&user_id)
            .cloned()
            .unwrap_or(Vec::new());
        Cart {
            user_id: user_id.to_string(),
            items,
        }
    }

    async fn empty_cart(&self, _rt: &Runtime, user_id: String) -> () {
        self.carts.lock().await.remove(&user_id);
    }
}

pub type CartClient = ops::RpcClient<CartService>;

pub fn component() -> Component {
    ops::component::<CartService>()
}
