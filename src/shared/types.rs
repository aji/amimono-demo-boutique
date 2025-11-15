use serde::{Deserialize, Serialize};

use crate::shared::Money;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ad {
    pub redirect_url: String,
    pub text: String,
}

impl Ad {
    pub fn new<S: ToString, T: ToString>(redirect_url: S, text: T) -> Ad {
        Ad {
            redirect_url: redirect_url.to_string(),
            text: text.to_string(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Address {
    pub street_address: String,
    pub city: String,
    pub state: String,
    pub country: String,
    pub zip_code: i32,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CartItem {
    pub product_id: String,
    pub quantity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditCardInfo {
    pub credit_card_number: String,
    pub credit_card_ccv: i32,
    pub credit_card_expiration_year: i32,
    pub credit_card_expiration_month: i32,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderItem {
    pub item: CartItem,
    pub cost: Money,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderResult {
    pub order_id: String,
    pub shipping_tracking_id: String,
    pub shipping_cost: Money,
    pub shipping_address: Address,
    pub items: Vec<OrderItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Product {
    pub id: String,
    pub name: String,
    pub description: String,
    pub picture: String,
    pub price_usd: Money,
    pub categories: Vec<String>,
}
