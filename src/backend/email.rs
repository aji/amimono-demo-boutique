use amimono::config::ComponentConfig;

use crate::shared::OrderResult;

mod ops {
    use crate::shared::OrderResult;

    amimono::rpc_ops! {
        fn send_order_confirmation(email: String, order: OrderResult) -> ();
    }
}

pub struct EmailService;

impl ops::Handler for EmailService {
    fn new() -> Self {
        EmailService
    }

    async fn send_order_confirmation(&self, _email: String, _order: OrderResult) -> () {
        todo!()
    }
}

pub type EmailClient = ops::Client<EmailService>;

pub fn component() -> ComponentConfig {
    ops::component::<EmailService>("emailservice".to_string())
}
