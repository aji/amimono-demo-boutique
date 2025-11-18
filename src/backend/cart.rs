use std::{collections::HashMap, sync::Arc};

use amimono::config::ComponentConfig;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::shared::CartItem;

#[derive(Serialize, Deserialize)]
pub struct Cart {
    pub user_id: String,
    pub items: Vec<CartItem>,
}

mod ops {
    use super::Cart;
    use crate::shared::CartItem;

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
    fn new() -> CartService {
        CartService {
            carts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn add_item(&self, user_id: String, item: CartItem) -> () {
        log::info!("add_item({}, {})", user_id, item.product_id);
        self.carts
            .lock()
            .await
            .entry(user_id)
            .or_insert(Vec::new())
            .push(item.clone());
    }

    async fn get_cart(&self, user_id: String) -> Cart {
        log::info!("get_cart({})", user_id);
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

    async fn empty_cart(&self, user_id: String) -> () {
        log::info!("empty_cart({})", user_id);
        self.carts.lock().await.remove(&user_id);
    }
}

pub type CartClient = ops::Client<CartService>;

pub fn component() -> ComponentConfig {
    ops::component::<CartService>("cartservice".to_owned())
}
