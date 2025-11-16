use serde::{Deserialize, Serialize};
use std::fmt::Write;
use tinytemplate::TinyTemplate;

use crate::shared::{CartItem, Product};

const CART_TEMPLATE: &'static str = include_str!("cart.html");
const FOOTER_TEMPLATE: &'static str = include_str!("footer.html");
const HEADER_TEMPLATE: &'static str = include_str!("header.html");
const HOME_TEMPLATE: &'static str = include_str!("home.html");
const PRODUCT_TEMPLATE: &'static str = include_str!("product.html");

#[derive(Serialize)]
pub struct HeaderContext {
    pub base_url: String,
    pub frontend_message: Option<String>,
    pub cart_size: usize,
}

#[derive(Serialize)]
pub struct FooterContext {}

#[derive(Serialize)]
pub struct HomeContext {
    pub base_url: String,
    pub header: HeaderContext,
    pub footer: FooterContext,
    pub products: Vec<Product>,
}

#[derive(Serialize)]
pub struct ProductContext {
    pub header: HeaderContext,
    pub footer: FooterContext,
    pub base_url: String,
    pub product: Product,
}

#[derive(Serialize)]
pub struct CartContext {
    pub header: HeaderContext,
    pub footer: FooterContext,
    pub items: Vec<CartItem>,
}

#[derive(Deserialize)]
pub struct CartForm {
    pub product_id: String,
    pub quantity: u32,
}

pub fn init() -> TinyTemplate<'static> {
    let mut tt = TinyTemplate::new();

    tt.add_template("cart", CART_TEMPLATE).unwrap();
    tt.add_template("footer", FOOTER_TEMPLATE).unwrap();
    tt.add_template("header", HEADER_TEMPLATE).unwrap();
    tt.add_template("home", HOME_TEMPLATE).unwrap();
    tt.add_template("product", PRODUCT_TEMPLATE).unwrap();

    tt.add_formatter("money", |val, s| {
        let money = val.as_object().unwrap();
        let currency_code = money.get("currency_code").unwrap().as_str().unwrap();
        let units = money.get("units").unwrap().as_i64().unwrap();
        let nanos = money.get("nanos").unwrap().as_i64().unwrap();
        write!(s, "{} {}.{:09}", currency_code, units, nanos)?;
        Ok(())
    });

    tt
}
