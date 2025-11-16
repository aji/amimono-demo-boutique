use amimono::{Component, Runtime};

use crate::shared::{CreditCardInfo, Money};

mod ops {
    use crate::shared::{CreditCardInfo, Money};

    amimono::rpc_ops! {
        fn charge(amount: Money, credit_card: CreditCardInfo) -> String;
    }
}

pub struct PaymentService;

impl ops::Handler for PaymentService {
    const LABEL: amimono::Label = "paymentservice";

    async fn new(_rt: &Runtime) -> Self {
        PaymentService
    }

    async fn charge(
        &self,
        _rt: &amimono::Runtime,
        amount: Money,
        credit_card: CreditCardInfo,
    ) -> String {
        log::info!("charge {:?} with {:?}", credit_card, amount);
        // TODO, leave this stubbed for now
        uuid::Uuid::new_v4().to_string()
    }
}

pub type PaymentClient = ops::RpcClient<PaymentService>;

pub fn component() -> Component {
    ops::component::<PaymentService>()
}
