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

#[allow(unused)]
fn configure_strict_monolith() -> AppConfig {
    AppBuilder::new()
        .add_job(
            JobBuilder::new()
                .with_label("boutique")
                .add_component(frontend::component())
                .add_component(adservice::component())
                .add_component(cartservice::component())
                .add_component(checkoutservice::component())
                .add_component(currencyservice::component())
                .add_component(emailservice::component())
                .add_component(paymentservice::component())
                .add_component(productcatalogservice::component())
                .add_component(recommendationservice::component())
                .add_component(shippingservice::component())
                .build(),
        )
        .build()
}

#[allow(unused)]
fn configure_strict_microservices() -> AppConfig {
    AppBuilder::new()
        .add_job(JobBuilder::new().add_component(frontend::component()))
        .add_job(JobBuilder::new().add_component(adservice::component()))
        .add_job(JobBuilder::new().add_component(cartservice::component()))
        .add_job(JobBuilder::new().add_component(checkoutservice::component()))
        .add_job(JobBuilder::new().add_component(currencyservice::component()))
        .add_job(JobBuilder::new().add_component(emailservice::component()))
        .add_job(JobBuilder::new().add_component(paymentservice::component()))
        .add_job(JobBuilder::new().add_component(productcatalogservice::component()))
        .add_job(JobBuilder::new().add_component(recommendationservice::component()))
        .add_job(JobBuilder::new().add_component(shippingservice::component()))
        .build()
}

fn configure() -> AppConfig {
    match std::env::var("AMIMONO_JOB").as_ref().map(String::as_str) {
        Ok("_local") => configure_strict_monolith(),
        _ => configure_strict_microservices(),
    }
}

fn main() {
    env_logger::init();
    amimono::entry(configure());
}
