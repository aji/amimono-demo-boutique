use amimono::{Component, Rpc, Runtime};
use serde::{Deserialize, Serialize};

use crate::{emailservice::OrderResult, paymentservice::CreditCardInfo, shippingservice::Address};

#[derive(Serialize, Deserialize)]
pub struct CheckoutServiceRequest {
    user_id: String,
    user_currency: String,

    address: Address,
    email: String,
    credit_card: CreditCardInfo,
}

#[derive(Serialize, Deserialize)]
pub struct CheckoutServiceResponse {
    order: OrderResult,
}

pub struct CheckoutService;

impl Rpc for CheckoutService {
    const LABEL: amimono::Label = "checkoutservice";

    type Request = CheckoutServiceRequest;

    type Response = CheckoutServiceResponse;

    async fn start(_rt: &Runtime) -> Self {
        CheckoutService
    }

    async fn handle(&self, _rt: &Runtime, _q: &Self::Request) -> Self::Response {
        todo!()
    }
}

pub fn component() -> Component {
    CheckoutService::component()
}
