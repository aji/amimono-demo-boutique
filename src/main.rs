use amimono::{AppBuilder, AppConfig, JobBuilder};

pub mod adservice;
pub mod cartservice;
pub mod checkoutservice;
pub mod currencyservice;
pub mod emailservice;
pub mod frontend;
pub mod paymentservice;
pub mod productcatalogservice;
pub mod recommendationservice;
pub mod shippingservice;

pub mod shared;

fn configure() -> AppConfig {
    AppBuilder::new()
        .add_job(
            JobBuilder::new()
                .with_label("boutique")
                .add_component(adservice::component())
                .add_component(cartservice::component())
                .add_component(checkoutservice::component())
                .add_component(currencyservice::component())
                .add_component(emailservice::component())
                .add_component(frontend::component())
                .add_component(paymentservice::component())
                .add_component(productcatalogservice::component())
                .add_component(recommendationservice::component())
                .add_component(shippingservice::component())
                .build(),
        )
        .build()
}

fn main() {
    env_logger::init();
    amimono::entry(configure());
}
