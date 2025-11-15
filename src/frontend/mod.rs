use amimono::{Binding, BindingType, Component, Runtime};
use axum::{
    Router,
    extract::Path,
    response::Html,
    routing::{MethodRouter, get},
};

use crate::{
    currencyservice::{self, CurrencyClient},
    productcatalogservice::{self, ProductCatalogClient},
};

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
                currency: currencyservice::client(rt).await,
                productcatalog: productcatalogservice::client(rt).await,
            },
        }
    }

    fn on_home(&self) -> MethodRouter<()> {
        let data = self.data.clone();
        get(async move || {
            let ctx = data.home_ctx();
            Html(templates::init().render("home", &ctx).unwrap())
        })
    }

    fn on_product(&self) -> MethodRouter<()> {
        let data = self.data.clone();
        get(async move |Path(id): Path<String>| {
            let ctx = data.product_ctx(&id).await;
            Html(templates::init().render("product", &ctx).unwrap())
        })
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
            .route("/", self.on_home())
            .route("/product/{id}", self.on_product());

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

    fn home_ctx(&self) -> templates::HomeContext {
        templates::HomeContext {
            base_url: self.base_url(),
            header: self.header_ctx(),
            footer: self.footer_ctx(),
            products: Vec::new(),
        }
    }

    async fn product_ctx(&self, id: &str) -> templates::ProductContext {
        let product = self.productcatalog.get_product(&self.rt, id).await.unwrap();
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
