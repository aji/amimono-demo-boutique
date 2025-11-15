use amimono::{Component, Rpc, RpcClient, RpcHandler, Runtime};
use serde::{Deserialize, Serialize};

use crate::{
    shared::{Address, CartItem, Money},
    shippingservice::ShippingService,
};

#[derive(Serialize, Deserialize)]
pub enum ShippingRequest {
    GetQuote {
        address: Address,
        items: Vec<CartItem>,
    },
    ShipOrder {
        address: Address,
        items: Vec<CartItem>,
    },
}

#[derive(Serialize, Deserialize)]
pub enum ShippingResponse {
    GetQuote(Money),
    ShipOrder(String),
}

struct ShippingServiceRpc(ShippingService);

impl Rpc for ShippingServiceRpc {
    const LABEL: amimono::Label = "shippingservice";

    type Handler = Self;

    type Client = ShippingClient;

    async fn start(rt: &Runtime) -> Self {
        ShippingServiceRpc(ShippingService::start(rt).await)
    }
}

impl RpcHandler for ShippingServiceRpc {
    type Request = ShippingRequest;

    type Response = ShippingResponse;

    async fn handle(&self, rt: &amimono::Runtime, q: Self::Request) -> Self::Response {
        match q {
            ShippingRequest::GetQuote { address, items } => {
                let a = self.0.get_quote(rt, &address, items.as_slice()).await;
                ShippingResponse::GetQuote(a)
            }
            ShippingRequest::ShipOrder { address, items } => {
                let a = self.0.ship_order(rt, &address, items.as_slice()).await;
                ShippingResponse::ShipOrder(a)
            }
        }
    }
}

pub struct ShippingClient(RpcClient<ShippingServiceRpc>);

impl ShippingClient {
    pub async fn get_quote(
        &self,
        rt: &Runtime,
        address: &Address,
        items: &[CartItem],
    ) -> Result<Money, ()> {
        let q = ShippingRequest::GetQuote {
            address: address.clone(),
            items: items.to_vec(),
        };
        match self.0.handle(rt, q).await {
            Ok(ShippingResponse::GetQuote(cost)) => Ok(cost),
            _ => Err(()),
        }
    }

    pub async fn ship_order(
        &self,
        rt: &Runtime,
        address: &Address,
        items: &[CartItem],
    ) -> Result<String, ()> {
        let q = ShippingRequest::ShipOrder {
            address: address.clone(),
            items: items.to_vec(),
        };
        match self.0.handle(rt, q).await {
            Ok(ShippingResponse::ShipOrder(tracking_id)) => Ok(tracking_id),
            _ => Err(()),
        }
    }
}

impl From<RpcClient<ShippingServiceRpc>> for ShippingClient {
    fn from(value: RpcClient<ShippingServiceRpc>) -> Self {
        ShippingClient(value)
    }
}

pub async fn client(rt: &Runtime) -> ShippingClient {
    ShippingServiceRpc::client(rt).await
}

pub fn component() -> Component {
    ShippingServiceRpc::component()
}
