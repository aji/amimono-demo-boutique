use amimono::{Component, Rpc, RpcClient, RpcHandler, Runtime};
use serde::{Deserialize, Serialize};

use crate::shared::OrderResult;

#[derive(Serialize, Deserialize)]
pub enum EmailServiceRequest {
    SendOrderConfirmation { email: String, order: OrderResult },
}

pub struct EmailService;

impl Rpc for EmailService {
    const LABEL: amimono::Label = "emailservice";

    type Handler = Self;
    type Client = RpcClient<Self>;

    async fn start(_rt: &Runtime) -> Self {
        EmailService
    }
}

impl RpcHandler for EmailService {
    type Request = EmailServiceRequest;
    type Response = ();

    async fn handle(&self, _rt: &Runtime, _q: Self::Request) -> Self::Response {
        todo!()
    }
}

pub type EmailClient = <EmailService as Rpc>::Client;

pub async fn client(rt: &Runtime) -> EmailClient {
    EmailService::client(rt).await
}

pub fn component() -> Component {
    EmailService::component()
}
