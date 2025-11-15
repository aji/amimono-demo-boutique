use amimono::{Component, Rpc, RpcClient, RpcHandler, Runtime};
use serde::{Deserialize, Serialize};

use crate::{
    cartservice::{CartClient, CartItem},
    currencyservice::{CurrencyClient, Money},
    emailservice::{EmailService, EmailServiceRequest, OrderItem, OrderResult},
    paymentservice::{
        CreditCardInfo, PaymentService, PaymentServiceRequest, PaymentServiceResponse,
    },
    productcatalogservice::ProductCatalogClient,
    shippingservice::{Address, ShippingClient},
};

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

pub struct CheckoutService {
    productcatalog: ProductCatalogClient,
    cart: CartClient,
    currency: CurrencyClient,
    shipping: ShippingClient,
    email: <EmailService as Rpc>::Client,
    payment: <PaymentService as Rpc>::Client,
}

struct OrderPrep {
    order_items: Vec<OrderItem>,
    cart_items: Vec<CartItem>,
    shipping_cost_localized: Money,
}

impl CheckoutService {
    async fn prepare_order_items_and_shipping_quote_from_cart(
        &self,
        rt: &Runtime,
        user_id: &str,
        user_currency: &str,
        address: &Address,
    ) -> OrderPrep {
        let cart_items = self.get_user_cart(rt, user_id).await;
        let order_items = self
            .prep_order_items(rt, cart_items.as_slice(), user_currency)
            .await;
        let shipping_usd = self
            .quote_shipping(rt, address, cart_items.as_slice())
            .await;
        let shipping_price = self
            .convert_currency(rt, &shipping_usd, user_currency)
            .await;

        OrderPrep {
            order_items,
            cart_items,
            shipping_cost_localized: shipping_price,
        }
    }

    async fn quote_shipping(
        &self,
        rt: &Runtime,
        address: &Address,
        cart_items: &[CartItem],
    ) -> Money {
        self.shipping
            .get_quote(rt, address, cart_items)
            .await
            .unwrap()
    }

    async fn get_user_cart(&self, rt: &Runtime, user_id: &str) -> Vec<CartItem> {
        self.cart.get_cart(rt, user_id).await.unwrap().items
    }

    async fn empty_user_cart(&self, rt: &Runtime, user_id: &str) {
        self.cart.empty_cart(rt, user_id).await.unwrap()
    }

    async fn prep_order_items(
        &self,
        rt: &Runtime,
        items: &[CartItem],
        user_currency: &str,
    ) -> Vec<OrderItem> {
        let mut res: Vec<OrderItem> = Vec::new();
        for item in items.iter() {
            let product = self
                .productcatalog
                .get_product(rt, item.product_id.as_str())
                .await
                .unwrap();
            let price = self
                .currency
                .convert(rt, &product.price_usd, user_currency)
                .await
                .unwrap();
            res.push(OrderItem {
                item: item.clone(),
                cost: price,
            });
        }
        res
    }

    async fn convert_currency(&self, rt: &Runtime, from: &Money, to: &str) -> Money {
        self.currency.convert(rt, from, to).await.unwrap()
    }

    async fn charge_card(
        &self,
        rt: &Runtime,
        amount: &Money,
        payment_info: &CreditCardInfo,
    ) -> String {
        let q = PaymentServiceRequest::Charge {
            amount: amount.clone(),
            credit_card: payment_info.clone(),
        };
        let a = self.payment.handle(rt, q).await.unwrap();
        let PaymentServiceResponse::Charge { transaction_id } = a;
        transaction_id
    }

    async fn send_order_confirmation(
        &self,
        rt: &Runtime,
        email: &str,
        order: &OrderResult,
    ) -> Result<(), ()> {
        let q = EmailServiceRequest::SendOrderConfirmation {
            email: email.to_string(),
            order: order.clone(),
        };
        self.email.handle(rt, q).await
    }

    async fn ship_order(&self, rt: &Runtime, address: &Address, items: &[CartItem]) -> String {
        self.shipping.ship_order(rt, address, items).await.unwrap()
    }
}

impl Rpc for CheckoutService {
    const LABEL: amimono::Label = "checkoutservice";

    type Handler = Self;

    type Client = RpcClient<Self>;

    async fn start(rt: &Runtime) -> Self {
        CheckoutService {
            productcatalog: ProductCatalogClient::new(rt).await,
            cart: CartClient::new(rt).await,
            currency: CurrencyClient::new(rt).await,
            shipping: ShippingClient::new(rt).await,
            email: EmailService::client(rt).await,
            payment: PaymentService::client(rt).await,
        }
    }
}

impl RpcHandler for CheckoutService {
    type Request = CheckoutServiceRequest;

    type Response = CheckoutServiceResponse;

    async fn handle(&self, rt: &Runtime, q: Self::Request) -> Self::Response {
        log::info!(
            "[PlaceOrder] user_id={} user_currency={}",
            q.user_id,
            q.user_currency
        );

        let order_id = uuid::Uuid::new_v4().to_string();
        let prep = self
            .prepare_order_items_and_shipping_quote_from_cart(
                rt,
                &q.user_id,
                &q.user_currency,
                &q.address,
            )
            .await;

        let total = prep.shipping_cost_localized.clone()
            + prep
                .order_items
                .iter()
                .map(|x| x.item.quantity * x.cost.clone())
                .sum();

        let tx_id = self.charge_card(rt, &total, &q.credit_card).await;
        log::info!("payment went through (transaction_id: {})", tx_id);

        let shipping_tracking_id = self.ship_order(rt, &q.address, &prep.cart_items[..]).await;

        self.empty_user_cart(rt, &q.user_id).await;

        let order = OrderResult {
            order_id,
            shipping_tracking_id,
            shipping_cost: prep.shipping_cost_localized,
            shipping_address: q.address.clone(),
            items: prep.order_items,
        };

        match self
            .send_order_confirmation(rt, q.email.as_str(), &order)
            .await
        {
            Ok(_) => log::info!("order confirmation email sent to {}", q.email),
            Err(_) => log::warn!("failed to send order confirmation to {}", q.email),
        }

        CheckoutServiceResponse { order }
    }
}

pub fn component() -> Component {
    CheckoutService::component()
}
