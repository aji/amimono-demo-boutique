use serde::Serialize;
use std::fmt::Write;
use tinytemplate::TinyTemplate;

use crate::shared::Product;

const FOOTER_TEMPLATE: &'static str = include_str!("footer.html");
const HEADER_TEMPLATE: &'static str = include_str!("header.html");
const HOME_TEMPLATE: &'static str = include_str!("home.html");
const PRODUCT_TEMPLATE: &'static str = include_str!("product.html");

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeaderContext {
    pub base_url: String,
    pub frontend_message: Option<String>,
    pub cart_size: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FooterContext {}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HomeContext {
    pub base_url: String,
    pub header: HeaderContext,
    pub footer: FooterContext,
    pub products: Vec<Product>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductContext {
    pub header: HeaderContext,
    pub footer: FooterContext,
    pub product: Product,
}

pub fn init() -> TinyTemplate<'static> {
    let mut tt = TinyTemplate::new();

    tt.add_template("footer", FOOTER_TEMPLATE).unwrap();
    tt.add_template("header", HEADER_TEMPLATE).unwrap();
    tt.add_template("home", HOME_TEMPLATE).unwrap();
    tt.add_template("product", PRODUCT_TEMPLATE).unwrap();

    tt.add_formatter("money", |val, s| {
        let money = val.as_object().unwrap();
        let currency_code = money.get("currencyCode").unwrap().as_str().unwrap();
        let units = money.get("units").unwrap().as_i64().unwrap();
        let nanos = money.get("nanos").unwrap().as_i64().unwrap();
        write!(s, "{} {}.{:09}", currency_code, units, nanos)?;
        Ok(())
    });

    tt
}
