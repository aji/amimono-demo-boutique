use amimono::{config::ComponentConfig, rpc::RpcError};

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
        user_id: &str,
        user_currency: &str,
        address: &Address,
    ) -> OrderPrep {
        let cart_items = self.get_user_cart(user_id).await;
        let order_items = self
            .prep_order_items(cart_items.as_slice(), user_currency)
            .await;
        let shipping_usd = self.quote_shipping(address, cart_items.as_slice()).await;
        let shipping_price = self.convert_currency(&shipping_usd, user_currency).await;

        OrderPrep {
            order_items,
            cart_items,
            shipping_cost_localized: shipping_price,
        }
    }

    async fn quote_shipping(&self, address: &Address, cart_items: &[CartItem]) -> Money {
        self.shipping
            .get_quote(address.clone(), cart_items.to_vec())
            .await
            .unwrap()
    }

    async fn get_user_cart(&self, user_id: &str) -> Vec<CartItem> {
        self.cart.get_cart(user_id.to_owned()).await.unwrap().items
    }

    async fn empty_user_cart(&self, user_id: &str) {
        self.cart.empty_cart(user_id.to_owned()).await.unwrap()
    }

    async fn prep_order_items(&self, items: &[CartItem], user_currency: &str) -> Vec<OrderItem> {
        let mut res: Vec<OrderItem> = Vec::new();
        for item in items.iter() {
            let product = self
                .productcatalog
                .get_product(item.product_id.to_string())
                .await
                .unwrap();
            let price = self
                .currency
                .convert(product.price_usd.clone(), user_currency.to_owned())
                .await
                .unwrap();
            res.push(OrderItem {
                item: item.clone(),
                cost: price,
            });
        }
        res
    }

    async fn convert_currency(&self, from: &Money, to: &str) -> Money {
        self.currency
            .convert(from.clone(), to.to_owned())
            .await
            .unwrap()
    }

    async fn charge_card(&self, amount: &Money, payment_info: &CreditCardInfo) -> String {
        self.payment
            .charge(amount.clone(), payment_info.clone())
            .await
            .unwrap()
    }

    async fn send_order_confirmation(
        &self,
        email: &str,
        order: &OrderResult,
    ) -> Result<(), RpcError> {
        self.email
            .send_order_confirmation(email.to_string(), order.clone())
            .await
    }

    async fn ship_order(&self, address: &Address, items: &[CartItem]) -> String {
        self.shipping
            .ship_order(address.clone(), items.to_vec())
            .await
            .unwrap()
    }
}

impl ops::Handler for CheckoutService {
    async fn new() -> Self {
        CheckoutService {
            productcatalog: ProductCatalogClient::new(),
            cart: CartClient::new(),
            currency: CurrencyClient::new(),
            shipping: ShippingClient::new(),
            email: EmailClient::new(),
            payment: PaymentClient::new(),
        }
    }

    async fn checkout(
        &self,
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
            .prepare_order_items_and_shipping_quote_from_cart(&user_id, &user_currency, &address)
            .await;

        let total = prep.shipping_cost_localized.clone()
            + prep
                .order_items
                .iter()
                .map(|x| x.item.quantity * x.cost.clone())
                .sum();

        let tx_id = self.charge_card(&total, &credit_card).await;
        log::info!("payment went through (transaction_id: {})", tx_id);

        let shipping_tracking_id = self.ship_order(&address, &prep.cart_items[..]).await;

        self.empty_user_cart(&user_id).await;

        let order = OrderResult {
            order_id,
            shipping_tracking_id,
            shipping_cost: prep.shipping_cost_localized,
            shipping_address: address.clone(),
            items: prep.order_items,
        };

        match self.send_order_confirmation(&email, &order).await {
            Ok(_) => log::info!("order confirmation email sent to {}", email),
            Err(_) => log::warn!("failed to send order confirmation to {}", email),
        }

        order
    }
}

pub type CheckoutClient = ops::Client<CheckoutService>;

pub fn component() -> ComponentConfig {
    ops::component::<CheckoutService>("checkoutservice".to_string())
}
