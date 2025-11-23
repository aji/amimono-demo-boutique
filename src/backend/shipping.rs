use amimono::{config::ComponentConfig, rpc::RpcResult};

use crate::shared::{Address, CartItem, Money};

mod ops {
    use crate::shared::{Address, CartItem, Money};

    amimono::rpc_ops! {
        fn get_quote(address: Address, items: Vec<CartItem>) -> Money;
        fn ship_order(address: Address, items: Vec<CartItem>) -> String;
    }
}

pub struct ShippingService;

impl ops::Handler for ShippingService {
    async fn new() -> Self {
        ShippingService
    }

    async fn get_quote(&self, _address: Address, _items: Vec<CartItem>) -> RpcResult<Money> {
        Ok(Money::from_usd(3, 50))
    }

    async fn ship_order(&self, _address: Address, _items: Vec<CartItem>) -> RpcResult<String> {
        Ok(uuid::Uuid::new_v4().to_string())
    }
}

pub type ShippingClient = ops::Client<ShippingService>;

pub fn component() -> ComponentConfig {
    ops::component::<ShippingService>("shippingservice".to_string())
}
