use amimono::{Component, Rpc, RpcClient, Runtime};
use serde::{Deserialize, Serialize};

use crate::{cartservice::CartItem, currencyservice::Money, shippingservice::Address};

#[derive(Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub item: CartItem,
    pub cost: Money,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OrderResult {
    pub order_id: String,
    pub shipping_tracking_id: String,
    pub shipping_cost: Money,
    pub shipping_address: Address,
    pub items: Vec<OrderItem>,
}

#[derive(Serialize, Deserialize)]
pub enum EmailServiceRequest {
    SendOrderConfirmation { email: String, order: OrderResult },
}

pub struct EmailService;

impl Rpc for EmailService {
    const LABEL: amimono::Label = "emailservice";

    type Request = EmailServiceRequest;
    type Response = ();

    async fn start(_rt: &Runtime) -> Self {
        EmailService
    }

    async fn handle(&self, _rt: &Runtime, _q: &Self::Request) -> Self::Response {
        todo!()
    }
}

pub async fn client(rt: &Runtime) -> RpcClient<EmailService> {
    EmailService::client(rt).await
}

pub fn component() -> Component {
    EmailService::component()
}
