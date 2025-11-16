use amimono::{AppBuilder, AppConfig, JobBuilder};

pub mod frontend;
pub mod service;
pub mod shared;

#[allow(unused)]
fn configure_strict_monolith() -> AppConfig {
    AppBuilder::new()
        .add_job(
            JobBuilder::new()
                .with_label("boutique")
                .add_component(frontend::component())
                .add_component(service::ad::component())
                .add_component(service::cart::component())
                .add_component(service::checkout::component())
                .add_component(service::currency::component())
                .add_component(service::email::component())
                .add_component(service::payment::component())
                .add_component(service::productcatalog::component())
                .add_component(service::recommendation::component())
                .add_component(service::shipping::component())
                .build(),
        )
        .build()
}

#[allow(unused)]
fn configure_strict_microservices() -> AppConfig {
    AppBuilder::new()
        .add_job(JobBuilder::new().add_component(frontend::component()))
        .add_job(JobBuilder::new().add_component(service::ad::component()))
        .add_job(JobBuilder::new().add_component(service::cart::component()))
        .add_job(JobBuilder::new().add_component(service::checkout::component()))
        .add_job(JobBuilder::new().add_component(service::currency::component()))
        .add_job(JobBuilder::new().add_component(service::email::component()))
        .add_job(JobBuilder::new().add_component(service::payment::component()))
        .add_job(JobBuilder::new().add_component(service::productcatalog::component()))
        .add_job(JobBuilder::new().add_component(service::recommendation::component()))
        .add_job(JobBuilder::new().add_component(service::shipping::component()))
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
