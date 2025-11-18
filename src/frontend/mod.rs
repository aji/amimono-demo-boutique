use std::{
    net::{Ipv4Addr, SocketAddr},
    time::Instant,
};

use amimono::config::{BindingType, ComponentConfig};
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
    async fn new() -> FrontendServer {
        // TODO: this
        let (sock_addr, base_url) = (
            (Ipv4Addr::LOCALHOST, 9000).into(),
            "http://localhost:9000".to_string(),
        );

        FrontendServer {
            data: FrontendServerData {
                sock_addr,
                base_url,
                ad: AdClient::new(),
                cart: CartClient::new(),
                checkout: CheckoutClient::new(),
                currency: CurrencyClient::new(),
                productcatalog: ProductCatalogClient::new(),
                shipping: ShippingClient::new(),
                recommendation: RecommendationClient::new(),
            },
        }
    }

    async fn start(&self) {
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
                        data.cart.empty_cart(user_id).await.unwrap();
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
        let products = self.productcatalog.list_products().await.unwrap();
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
            .get_product(id.to_string())
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
        let cart = self.cart.get_cart(user_id).await.unwrap();
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
        self.cart.add_item(user_id, item).await.unwrap();
        jar
    }
}

#[tokio::main]
async fn frontend_main() {
    let server = FrontendServer::new().await;
    server.start().await
}

pub fn component() -> ComponentConfig {
    ComponentConfig {
        label: "frontend".to_string(),
        binding: BindingType::Http,
        register: |_, _| (),
        entry: frontend_main,
    }
}
