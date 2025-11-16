use amimono::{Component, Runtime};

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
    const LABEL: amimono::Label = "shippingservice";

    async fn new(_rt: &Runtime) -> Self {
        ShippingService
    }

    async fn get_quote(&self, _rt: &Runtime, _address: Address, _items: Vec<CartItem>) -> Money {
        todo!()
    }

    async fn ship_order(&self, _rt: &Runtime, _address: Address, _items: Vec<CartItem>) -> String {
        todo!()
    }
}

pub type ShippingClient = ops::RpcClient<ShippingService>;

pub fn component() -> Component {
    ops::component::<ShippingService>()
}
