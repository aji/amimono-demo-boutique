use amimono::{Component, RpcError, Runtime};

use crate::{
    backend::{
        CartClient, CurrencyClient, EmailClient, PaymentClient, ProductCatalogClient,
        ShippingClient,
    },
    shared::{Address, CartItem, CreditCardInfo, Money, OrderItem, OrderResult},
};

mod ops {
    use crate::shared::{Address, CreditCardInfo, OrderResult};

    amimono::rpc_ops! {
        fn checkout(
            user_id: String,
            user_currency: String,
            address: Address,
            email: String,
            credit_card: CreditCardInfo
        ) -> OrderResult;
    }
}

pub struct CheckoutService {
    productcatalog: ProductCatalogClient,
    cart: CartClient,
    currency: CurrencyClient,
    shipping: ShippingClient,
    email: EmailClient,
    payment: PaymentClient,
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
            .get_quote(rt, address.clone(), cart_items.to_vec())
            .await
            .unwrap()
    }

    async fn get_user_cart(&self, rt: &Runtime, user_id: &str) -> Vec<CartItem> {
        self.cart
            .get_cart(rt, user_id.to_owned())
            .await
            .unwrap()
            .items
    }

    async fn empty_user_cart(&self, rt: &Runtime, user_id: &str) {
        self.cart.empty_cart(rt, user_id.to_owned()).await.unwrap()
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
                .get_product(rt, item.product_id.to_string())
                .await
                .unwrap();
            let price = self
                .currency
                .convert(rt, product.price_usd.clone(), user_currency.to_owned())
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
        self.currency
            .convert(rt, from.clone(), to.to_owned())
            .await
            .unwrap()
    }

    async fn charge_card(
        &self,
        rt: &Runtime,
        amount: &Money,
        payment_info: &CreditCardInfo,
    ) -> String {
        self.payment
            .charge(rt, amount.clone(), payment_info.clone())
            .await
            .unwrap()
    }

    async fn send_order_confirmation(
        &self,
        rt: &Runtime,
        email: &str,
        order: &OrderResult,
    ) -> Result<(), RpcError> {
        self.email
            .send_order_confirmation(rt, email.to_string(), order.clone())
            .await
    }

    async fn ship_order(&self, rt: &Runtime, address: &Address, items: &[CartItem]) -> String {
        self.shipping
            .ship_order(rt, address.clone(), items.to_vec())
            .await
            .unwrap()
    }
}

impl ops::Handler for CheckoutService {
    const LABEL: amimono::Label = "checkoutservice";

    async fn new(rt: &Runtime) -> Self {
        CheckoutService {
            productcatalog: ProductCatalogClient::new(rt).await,
            cart: CartClient::new(rt).await,
            currency: CurrencyClient::new(rt).await,
            shipping: ShippingClient::new(rt).await,
            email: EmailClient::new(rt).await,
            payment: PaymentClient::new(rt).await,
        }
    }

    async fn checkout(
        &self,
        rt: &Runtime,
        user_id: String,
        user_currency: String,
        address: Address,
        email: String,
        credit_card: CreditCardInfo,
    ) -> OrderResult {
        log::info!(
            "[PlaceOrder] user_id={} user_currency={}",
            user_id,
            user_currency
        );

        let order_id = uuid::Uuid::new_v4().to_string();
        let prep = self
            .prepare_order_items_and_shipping_quote_from_cart(
                rt,
                &user_id,
                &user_currency,
                &address,
            )
            .await;

        let total = prep.shipping_cost_localized.clone()
            + prep
                .order_items
                .iter()
                .map(|x| x.item.quantity * x.cost.clone())
                .sum();

        let tx_id = self.charge_card(rt, &total, &credit_card).await;
        log::info!("payment went through (transaction_id: {})", tx_id);

        let shipping_tracking_id = self.ship_order(rt, &address, &prep.cart_items[..]).await;

        self.empty_user_cart(rt, &user_id).await;

        let order = OrderResult {
            order_id,
            shipping_tracking_id,
            shipping_cost: prep.shipping_cost_localized,
            shipping_address: address.clone(),
            items: prep.order_items,
        };

        match self.send_order_confirmation(rt, &email, &order).await {
            Ok(_) => log::info!("order confirmation email sent to {}", email),
            Err(_) => log::warn!("failed to send order confirmation to {}", email),
        }

        order
    }
}

pub type CheckoutClient = ops::RpcClient<CheckoutService>;

pub fn component() -> Component {
    ops::component::<CheckoutService>()
}
