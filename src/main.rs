use amimono::config::{AppBuilder, AppConfig, JobBuilder};

pub mod backend;
pub mod frontend;
pub mod shared;

#[allow(unused)]
fn configure_strict_monolith() -> AppConfig {
    AppBuilder::new()
        .add_job(
            JobBuilder::new()
                .with_label("boutique")
                .add_component(frontend::component())
                .add_component(backend::ad::component())
                .add_component(backend::cart::component())
                .add_component(backend::checkout::component())
                .add_component(backend::currency::component())
                .add_component(backend::email::component())
                .add_component(backend::payment::component())
                .add_component(backend::productcatalog::component())
                .add_component(backend::recommendation::component())
                .add_component(backend::shipping::component())
                .build(),
        )
        .build()
}

#[allow(unused)]
fn configure_strict_microservices() -> AppConfig {
    AppBuilder::new()
        .add_job(JobBuilder::new().add_component(frontend::component()))
        .add_job(JobBuilder::new().add_component(backend::ad::component()))
        .add_job(JobBuilder::new().add_component(backend::cart::component()))
        .add_job(JobBuilder::new().add_component(backend::checkout::component()))
        .add_job(JobBuilder::new().add_component(backend::currency::component()))
        .add_job(JobBuilder::new().add_component(backend::email::component()))
        .add_job(JobBuilder::new().add_component(backend::payment::component()))
        .add_job(JobBuilder::new().add_component(backend::productcatalog::component()))
        .add_job(JobBuilder::new().add_component(backend::recommendation::component()))
        .add_job(JobBuilder::new().add_component(backend::shipping::component()))
        .build()
}

fn configure() -> AppConfig {
    configure_strict_monolith()
}

fn main() {
    env_logger::init();
    amimono::entry(configure());
}
