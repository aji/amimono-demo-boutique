use std::time::Instant;

use amimono::{Binding, BindingType, Component, Runtime};
use axum::{
    Router,
    extract::Path,
    response::Html,
    routing::{get, post},
};

use crate::service::{CurrencyClient, ProductCatalogClient};

mod templates;

struct FrontendServer {
    data: FrontendServerData,
}

#[derive(Clone)]
struct FrontendServerData {
    rt: Runtime,
    #[allow(unused)]
    currency: CurrencyClient,
    productcatalog: ProductCatalogClient,
}

impl FrontendServer {
    async fn new(rt: &Runtime) -> FrontendServer {
        FrontendServer {
            data: FrontendServerData {
                rt: rt.clone(),
                currency: CurrencyClient::new(rt).await,
                productcatalog: ProductCatalogClient::new(rt).await,
            },
        }
    }

    async fn start(&self, rt: &Runtime) {
        let sock = match rt.binding() {
            Binding::Http(addr, _) => addr,
            binding => {
                log::error!("got {:?} instead of an Http binding, cannot start", binding);
                return;
            }
        };

        let app = Router::new()
            .route("/", {
                get({
                    let data = self.data.clone();
                    async move || {
                        let ctx = data.home_ctx().await;
                        Html(templates::init().render("home", &ctx).unwrap())
                    }
                })
            })
            .route("/product/{id}", {
                get({
                    let data = self.data.clone();
                    async move |Path(id): Path<String>| {
                        let ctx = data.product_ctx(&id).await;
                        Html(templates::init().render("product", &ctx).unwrap())
                    }
                })
            })
            .route("/cart", {
                get({
                    let _data = self.data.clone();
                    async move || Html("<h1>TODO</h1>")
                })
                .post({
                    let _data = self.data.clone();
                    async move || Html("<h1>TODO</h1>")
                })
            })
            .route("/cart/empty", {
                post({
                    let _data = self.data.clone();
                    async move || Html("<h1>TODO</h1>")
                })
            })
            .route("/set_currency", {
                post({
                    let _data = self.data.clone();
                    async move || Html("<h1>TODO</h1>")
                })
            })
            .route("/logout", {
                post({
                    let _data = self.data.clone();
                    async move || Html("<h1>TODO</h1>")
                })
            })
            .route("/cart/checkout", {
                post({
                    let _data = self.data.clone();
                    async move || Html("<h1>TODO</h1>")
                })
            })
            .layer({
                use axum::extract::Request;
                use axum::middleware::{self, Next};
                middleware::from_fn(async |req: Request, next: Next| {
                    let start = Instant::now();
                    let prefix = format!("{} {:?}", req.method(), req.uri());
                    let res = next.run(req).await;
                    log::info!(
                        "{} - {} - {}ms",
                        prefix,
                        res.status(),
                        start.elapsed().as_millis()
                    );
                    res
                })
            });

        let listener = tokio::net::TcpListener::bind(sock).await.unwrap();
        log::info!("frontend listening on {}", sock);
        axum::serve(listener, app).await.unwrap();
    }
}

impl FrontendServerData {
    fn base_url(&self) -> String {
        "".to_string()
    }

    fn header_ctx(&self) -> templates::HeaderContext {
        templates::HeaderContext {
            base_url: self.base_url(),
            frontend_message: None,
            cart_size: 0,
        }
    }

    fn footer_ctx(&self) -> templates::FooterContext {
        templates::FooterContext {}
    }

    async fn home_ctx(&self) -> templates::HomeContext {
        let products = self.productcatalog.list_products(&self.rt).await.unwrap();
        templates::HomeContext {
            base_url: self.base_url(),
            header: self.header_ctx(),
            footer: self.footer_ctx(),
            products,
        }
    }

    async fn product_ctx(&self, id: &str) -> templates::ProductContext {
        let product = self
            .productcatalog
            .get_product(&self.rt, id.to_string())
            .await
            .unwrap();
        templates::ProductContext {
            header: self.header_ctx(),
            footer: self.footer_ctx(),
            product,
        }
    }
}

async fn frontend_main(rt: Runtime) {
    let server = FrontendServer::new(&rt).await;
    server.start(&rt).await
}

pub fn component() -> Component {
    Component::from_async_fn("frontend", BindingType::Http, frontend_main)
}
