use std::time::Instant;

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
        FrontendServer {
            data: FrontendServerData {
                rt: rt.clone(),
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
        log::info!("frontend listening on http://{}", sock);
        axum::serve(listener, app).await.unwrap();
    }
}

impl FrontendServerData {
    fn base_url(&self) -> String {
        "".to_string()
    }

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
            base_url: self.base_url(),
            product,
        }
    }

    async fn cart_ctx(&self, jar: CookieJar) -> (CookieJar, templates::CartContext) {
        let (jar, user_id) = self.get_or_set_user_id(jar);
        log::info!("loading cart for {}", user_id);
        let cart = self.cart.get_cart(&self.rt, user_id).await.unwrap();
        let ctx = templates::CartContext {
            header: self.header_ctx(),
            footer: self.footer_ctx(),
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
