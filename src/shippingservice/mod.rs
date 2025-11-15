use amimono::{Component, Runtime};

use crate::shared::{Address, CartItem, Money};

mod rpc;
pub use rpc::ShippingClient;

pub(in crate::shippingservice) struct ShippingService;

impl ShippingService {
    async fn start(_rt: &Runtime) -> Self {
        ShippingService
    }

    async fn get_quote(&self, _rt: &Runtime, _address: &Address, _items: &[CartItem]) -> Money {
        todo!()
    }

    async fn ship_order(&self, _rt: &Runtime, _address: &Address, _items: &[CartItem]) -> String {
        todo!()
    }
}

pub async fn client(rt: &Runtime) -> ShippingClient {
    rpc::ShippingClient::new(rt).await
}

pub fn component() -> Component {
    rpc::component()
}
