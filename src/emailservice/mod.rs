use amimono::{Component, Runtime};

use crate::shared::OrderResult;

mod ops {
    use crate::shared::OrderResult;

    amimono::rpc_ops! {
        fn send_order_confirmation(email: String, order: OrderResult) -> ();
    }
}

pub struct EmailService;

impl ops::Handler for EmailService {
    const LABEL: amimono::Label = "emailservice";

    async fn new(_rt: &Runtime) -> Self {
        EmailService
    }

    async fn send_order_confirmation(
        &self,
        _rt: &amimono::Runtime,
        _email: String,
        _order: OrderResult,
    ) -> () {
        todo!()
    }
}

pub type EmailClient = ops::RpcClient<EmailService>;

pub fn component() -> Component {
    ops::component::<EmailService>()
}
