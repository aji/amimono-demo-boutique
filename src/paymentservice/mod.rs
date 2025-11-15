use amimono::{Component, Rpc, RpcClient, Runtime};
use serde::{Deserialize, Serialize};

use crate::currencyservice::Money;

#[derive(Serialize, Deserialize)]
pub struct CreditCardInfo {
    credit_card_number: String,
    credit_card_ccv: i32,
    credit_card_expiration_year: i32,
    credit_card_expiration_month: i32,
}

#[derive(Serialize, Deserialize)]
pub enum PaymentServiceRequest {
    Charge {
        amount: Money,
        credit_card: CreditCardInfo,
    },
}

#[derive(Serialize, Deserialize)]
pub enum PaymentServiceResponse {
    Charge { transaction_id: String },
}

pub struct PaymentService;

impl Rpc for PaymentService {
    const LABEL: amimono::Label = "paymentservice";

    type Request = PaymentServiceRequest;
    type Response = PaymentServiceResponse;

    async fn start(_rt: &Runtime) -> Self {
        PaymentService
    }

    async fn handle(&self, _rt: &Runtime, _q: &Self::Request) -> Self::Response {
        todo!()
    }
}

pub async fn client(rt: &Runtime) -> RpcClient<PaymentService> {
    PaymentService::client(rt).await
}

pub fn component() -> Component {
    PaymentService::component()
}
