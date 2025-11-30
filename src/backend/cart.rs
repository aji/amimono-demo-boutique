use std::collections::HashMap;

use amimono::{config::ComponentConfig, rpc::RpcResult};
use amimono_haze::crdt::{
    Crdt, CrdtClient, StoredCrdt,
    crdt::{Max, Version},
};
use serde::{Deserialize, Serialize};

use crate::shared::CartItem;

#[derive(Serialize, Deserialize)]
pub struct Cart {
    pub user_id: String,
    pub items: Vec<CartItem>,
}

#[derive(Serialize, Deserialize)]
struct CartData {
    items: Version<u32, HashMap<String, Max<u32>>>,
}

impl Crdt for CartData {
    fn merge_from(&mut self, other: Self) {
        self.items.merge_from(other.items);
    }
}

impl StoredCrdt for CartData {}

impl Default for CartData {
    fn default() -> Self {
        Self {
            items: Version(0, HashMap::new()),
        }
    }
}

impl CartData {
    fn to_cart(self, user_id: String) -> Cart {
        let items = self
            .items
            .1
            .into_iter()
            .map(|(k, v)| CartItem {
                product_id: k,
                quantity: v.0,
            })
            .collect();
        Cart { user_id, items }
    }
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
    crdt: CrdtClient<CartData>,
}

impl ops::Handler for CartService {
    async fn new() -> CartService {
        CartService {
            crdt: CrdtClient::new("cart".to_owned()),
        }
    }

    async fn add_item(&self, user_id: String, item: CartItem) -> RpcResult<()> {
        log::info!("add_item({}, {})", user_id, item.product_id);
        let cart = {
            let mut cart = self.crdt.get_or_default(&user_id).await?;
            let qty = cart.items.1.entry(item.product_id).or_insert(Max(0));
            qty.0 += item.quantity;
            cart
        };
        self.crdt.put(&user_id, cart).await?;
        Ok(())
    }

    async fn get_cart(&self, user_id: String) -> RpcResult<Cart> {
        let cart = self.crdt.get_or_default(&user_id).await?;
        Ok(cart.to_cart(user_id))
    }

    async fn empty_cart(&self, user_id: String) -> RpcResult<()> {
        log::info!("empty_cart({})", user_id);
        let cart = {
            let mut cart = self.crdt.get_or_default(&user_id).await?;
            cart.items.0 += 1;
            cart.items.1.clear();
            cart
        };
        self.crdt.put(&user_id, cart).await?;
        Ok(())
    }
}

pub type CartClient = ops::Client<CartService>;

pub fn component() -> ComponentConfig {
    CartData::bind("cart");
    ops::component::<CartService>("cartservice".to_owned())
}
