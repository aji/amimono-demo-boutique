use amimono::{Component, Runtime};
use serde::{Deserialize, Serialize};

use crate::{cartservice::CartItem, currencyservice::Money};

mod rpc;

pub use rpc::ShippingClient;

#[derive(Clone, Serialize, Deserialize)]
pub struct Address {
    street_address: String,
    city: String,
    state: String,
    country: String,
    zip_code: i32,
}

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
