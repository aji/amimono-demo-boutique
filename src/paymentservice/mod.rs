use amimono::{Component, Rpc, RpcClient, RpcHandler, Runtime};
use serde::{Deserialize, Serialize};

use crate::currencyservice::Money;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditCardInfo {
    credit_card_number: String,
    credit_card_ccv: i32,
    credit_card_expiration_year: i32,
    credit_card_expiration_month: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PaymentServiceRequest {
    Charge {
        amount: Money,
        credit_card: CreditCardInfo,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PaymentServiceResponse {
    Charge { transaction_id: String },
}

pub struct PaymentService;

impl Rpc for PaymentService {
    const LABEL: amimono::Label = "paymentservice";

    type Handler = Self;
    type Client = RpcClient<Self>;

    async fn start(_rt: &Runtime) -> Self {
        PaymentService
    }
}

impl RpcHandler for PaymentService {
    type Request = PaymentServiceRequest;
    type Response = PaymentServiceResponse;

    async fn handle(&self, _rt: &Runtime, q: Self::Request) -> Self::Response {
        log::info!("invoked: {:?}", q);
        // TODO, leave this stubbed for now
        PaymentServiceResponse::Charge {
            transaction_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

pub async fn client(rt: &Runtime) -> RpcClient<PaymentService> {
    PaymentService::client(rt).await
}

pub fn component() -> Component {
    PaymentService::component()
}
