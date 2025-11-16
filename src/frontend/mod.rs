use std::{net::SocketAddr, time::Instant};

use amimono::{Binding, BindingType, Component, Runtime};
use axum::{
    Form, Router,
    extract::Path,
    response::{Html, Redirect},
    routing::{get, post},
};
use axum_extra::extract::{CookieJar, cookie::Cookie};

use crate::{
    backend::{
        AdClient, CartClient, CheckoutClient, CurrencyClient, ProductCatalogClient,
        RecommendationClient, ShippingClient,
    },
    shared::CartItem,
};

mod templates;

struct FrontendServer {
    data: FrontendServerData,
}

#[derive(Clone)]
#[allow(unused)]
struct FrontendServerData {
    rt: Runtime,
    sock_addr: SocketAddr,
    base_url: String,
    ad: AdClient,
    cart: CartClient,
    checkout: CheckoutClient,
    currency: CurrencyClient,
    productcatalog: ProductCatalogClient,
    shipping: ShippingClient,
    recommendation: RecommendationClient,
}

impl FrontendServer {
    async fn new(rt: &Runtime) -> FrontendServer {
        let (sock_addr, base_url) = match rt.binding() {
            Binding::Http(sock_addr, base_url) => (sock_addr.clone(), base_url.clone()),
            binding => {
                log::error!("got {:?} instead of an Http binding, cannot start", binding);
                panic!();
            }
        };

        FrontendServer {
            data: FrontendServerData {
                rt: rt.clone(),
                sock_addr,
                base_url,
                ad: AdClient::new(rt).await,
                cart: CartClient::new(rt).await,
                checkout: CheckoutClient::new(rt).await,
                currency: CurrencyClient::new(rt).await,
                productcatalog: ProductCatalogClient::new(rt).await,
                shipping: ShippingClient::new(rt).await,
                recommendation: RecommendationClient::new(rt).await,
            },
        }
    }

    async fn start(&self, _rt: &Runtime) {
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
            .nest_service("/static", tower_http::services::ServeDir::new("static"))
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
                    let data = self.data.clone();
                    async move |jar: CookieJar| {
                        let (jar, ctx) = data.cart_ctx(jar).await;
                        (jar, Html(templates::init().render("cart", &ctx).unwrap()))
                    }
                })
                .post({
                    let data = self.data.clone();
                    async move |jar: CookieJar, Form(form): Form<templates::CartForm>| {
                        let jar = data.cart_form(jar, form).await;
                        (jar, Redirect::to("/cart"))
                    }
                })
            })
            .route("/cart/empty", {
                post({
                    let data = self.data.clone();
                    async move |jar: CookieJar| {
                        let (jar, user_id) = data.get_or_set_user_id(jar);
                        data.cart.empty_cart(&data.rt, user_id).await.unwrap();
                        (jar, Redirect::to("/cart"))
                    }
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

        let listener = tokio::net::TcpListener::bind(self.data.sock_addr)
            .await
            .unwrap();
        log::info!("frontend listening on {}", self.data.base_url);
        axum::serve(listener, app).await.unwrap();
    }
}

impl FrontendServerData {
    fn get_or_set_user_id(&self, jar: CookieJar) -> (CookieJar, String) {
        let key = "BOUTIQUE_USER_ID";
        let value = jar.get(key).map(|x| x.value().to_string());
        match value {
            Some(x) => (jar, x),
            None => {
                let id = uuid::Uuid::new_v4().to_string();
                (jar.add(Cookie::new(key, id.clone())), id)
            }
        }
    }

    async fn header_ctx(&'_ self) -> templates::HeaderContext<'_> {
        templates::HeaderContext {
            base_url: self.base_url.as_str(),
        }
    }

    async fn footer_ctx(&'_ self) -> templates::FooterContext<'_> {
        templates::FooterContext {
            base_url: self.base_url.as_str(),
        }
    }

    async fn home_ctx(&'_ self) -> templates::HomeContext<'_> {
        let products = self.productcatalog.list_products(&self.rt).await.unwrap();
        templates::HomeContext {
            header: self.header_ctx().await,
            footer: self.footer_ctx().await,
            base_url: self.base_url.as_str(),
            products,
        }
    }

    async fn product_ctx(&'_ self, id: &str) -> templates::ProductContext<'_> {
        let product = self
            .productcatalog
            .get_product(&self.rt, id.to_string())
            .await
            .unwrap();
        templates::ProductContext {
            header: self.header_ctx().await,
            footer: self.footer_ctx().await,
            base_url: self.base_url.as_str(),
            product,
        }
    }

    async fn cart_ctx(&'_ self, jar: CookieJar) -> (CookieJar, templates::CartContext<'_>) {
        let (jar, user_id) = self.get_or_set_user_id(jar);
        log::info!("loading cart for {}", user_id);
        let cart = self.cart.get_cart(&self.rt, user_id).await.unwrap();
        let ctx = templates::CartContext {
            header: self.header_ctx().await,
            footer: self.footer_ctx().await,
            base_url: self.base_url.as_str(),
            items: cart.items,
        };
        (jar, ctx)
    }

    async fn cart_form(&self, jar: CookieJar, form: templates::CartForm) -> CookieJar {
        let (jar, user_id) = self.get_or_set_user_id(jar);
        let item = CartItem {
            product_id: form.product_id,
            quantity: form.quantity,
        };
        self.cart.add_item(&self.rt, user_id, item).await.unwrap();
        jar
    }
}

async fn frontend_main(rt: Runtime) {
    let server = FrontendServer::new(&rt).await;
    server.start(&rt).await
}

pub fn component() -> Component {
    Component::from_async_fn("frontend", BindingType::Http, frontend_main)
}
