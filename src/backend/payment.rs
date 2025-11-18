use amimono::config::ComponentConfig;

use crate::shared::{CreditCardInfo, Money};

mod ops {
    use crate::shared::{CreditCardInfo, Money};

    amimono::rpc_ops! {
        fn charge(amount: Money, credit_card: CreditCardInfo) -> String;
    }
}

pub struct PaymentService;

impl ops::Handler for PaymentService {
    async fn new() -> Self {
        PaymentService
    }

    async fn charge(&self, amount: Money, credit_card: CreditCardInfo) -> String {
        log::info!("charge {:?} with {:?}", credit_card, amount);
        // TODO, leave this stubbed for now
        uuid::Uuid::new_v4().to_string()
    }
}

pub type PaymentClient = ops::Client<PaymentService>;

pub fn component() -> ComponentConfig {
    ops::component::<PaymentService>("paymentservice".to_string())
}
