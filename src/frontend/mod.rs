use std::{fmt, net::SocketAddr, time::Instant};

use amimono::{
    config::{Binding, BindingType, ComponentConfig},
    rpc::RpcError,
    runtime::{self, Component},
};
use axum::{
    Form, Router,
    extract::Path,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
};
use axum_extra::extract::{CookieJar, cookie::Cookie};

use crate::backend::{
    AdClient, CartClient, CheckoutClient, CurrencyClient, ProductCatalogClient,
    RecommendationClient, ShippingClient,
};
use crate::shared::{Address, CartItem, CreditCardInfo, OrderResult};

mod templates;

type Res<T> = Result<T, FrontendError>;

type Page = Result<(CookieJar, Html<String>), FrontendError>;
type Post = Result<(CookieJar, Redirect), FrontendError>;

#[derive(Debug)]
enum FrontendError {
    Rpc(RpcError),
    Template(tinytemplate::error::Error),
}

impl From<RpcError> for FrontendError {
    fn from(err: RpcError) -> Self {
        FrontendError::Rpc(err)
    }
}
impl From<tinytemplate::error::Error> for FrontendError {
    fn from(err: tinytemplate::error::Error) -> Self {
        FrontendError::Template(err)
    }
}

impl fmt::Display for FrontendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrontendError::Rpc(e) => write!(f, "RPC error: {:?}", e),
            FrontendError::Template(e) => write!(f, "Template error: {}", e),
        }
    }
}

impl IntoResponse for FrontendError {
    fn into_response(self) -> axum::response::Response {
        let res = (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("{}", self),
        );
        res.into_response()
    }
}

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
        let sock_addr = match runtime::binding::<Self>() {
            Binding::Http(port) => ([0, 0, 0, 0], port).into(),
            _ => panic!("FrontendServer does not have a binding"),
        };
        let base_url = match std::env::var("BOUTIQUE_BASE_URL") {
            Ok(url) => url,
            Err(_) => "".to_owned(),
        };

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
                    async move |jar: CookieJar| -> Page {
                        let (jar, ctx) = data.home_ctx(jar).await?;
                        let html = templates::init().render("home", &ctx)?;
                        Ok((jar, Html(html)))
                    }
                })
            })
            .nest_service("/static", tower_http::services::ServeDir::new("static"))
            .route("/product/{id}", {
                get({
                    let data = self.data.clone();
                    async move |jar: CookieJar, Path(id): Path<String>| -> Page {
                        let ctx = data.product_ctx(&id).await?;
                        Ok((jar, Html(templates::init().render("product", &ctx)?)))
                    }
                })
            })
            .route("/cart", {
                get({
                    let data = self.data.clone();
                    async move |jar: CookieJar| -> Page {
                        let (jar, ctx) = data.cart_ctx(jar).await?;
                        Ok((jar, Html(templates::init().render("cart", &ctx)?)))
                    }
                })
                .post({
                    let data = self.data.clone();
                    async move |jar: CookieJar, Form(form): Form<templates::CartForm>| -> Post {
                        let jar = data.cart_form(jar, form).await?;
                        Ok((jar, Redirect::to("/cart")))
                    }
                })
            })
            .route("/cart/empty", {
                post({
                    let data = self.data.clone();
                    async move |jar: CookieJar| -> Post {
                        let (jar, user_id) = data.get_or_set_user_id(jar);
                        data.cart.empty_cart(user_id).await?;
                        Ok((jar, Redirect::to("/cart")))
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
                    let data = self.data.clone();
                    async move |jar: CookieJar, Form(form): Form<templates::CheckoutForm>| -> Page {
                        let (jar, order) = data.checkout_form(jar, form).await?;
                        let ctx = data.checkout_ctx(order).await?;
                        Ok((jar, Html(templates::init().render("checkout", &ctx)?)))
                    }
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
        log::info!(
            "frontend listening on {} (base_url={:?})",
            self.data.sock_addr,
            self.data.base_url
        );
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

    async fn header_ctx(&'_ self) -> Res<templates::HeaderContext<'_>> {
        Ok(templates::HeaderContext {
            base_url: self.base_url.as_str(),
        })
    }

    async fn footer_ctx(&'_ self) -> Res<templates::FooterContext<'_>> {
        Ok(templates::FooterContext {
            base_url: self.base_url.as_str(),
        })
    }

    async fn home_ctx(&'_ self, jar: CookieJar) -> Res<(CookieJar, templates::HomeContext<'_>)> {
        let products = self.productcatalog.list_products().await?;
        // Get user_id from cookie jar
        let (jar, user_id) = self.get_or_set_user_id(jar);
        // Get recommended product ids from recommendation service
        let recommended_ids = self
            .recommendation
            .list_recommendations(
                user_id.clone(),
                products.iter().map(|p| p.id.clone()).collect(),
            )
            .await?;
        // Join recommended ids with products
        let recommended: Vec<_> = recommended_ids
            .into_iter()
            .filter_map(|id| products.iter().find(|p| p.id == id).cloned())
            .collect();
        Ok((
            jar,
            templates::HomeContext {
                header: self.header_ctx().await?,
                footer: self.footer_ctx().await?,
                base_url: self.base_url.as_str(),
                products,
                recommended,
            },
        ))
    }

    async fn product_ctx(&'_ self, id: &str) -> Res<templates::ProductContext<'_>> {
        let product = self.productcatalog.get_product(id.to_string()).await?;
        // Fetch ads using product categories
        let mut ads = self
            .ad
            .get_ads(product.categories.clone())
            .await
            .unwrap_or_default();
        // Filter out ads that match the current product id
        ads.retain(|ad| !ad.redirect_url.contains(&product.id));
        Ok(templates::ProductContext {
            header: self.header_ctx().await?,
            footer: self.footer_ctx().await?,
            base_url: self.base_url.as_str(),
            product,
            ads,
        })
    }

    async fn cart_ctx(&'_ self, jar: CookieJar) -> Res<(CookieJar, templates::CartContext<'_>)> {
        let (jar, user_id) = self.get_or_set_user_id(jar);
        log::info!("loading cart for {}", user_id);
        let cart = self.cart.get_cart(user_id).await?;
        let ctx = templates::CartContext {
            header: self.header_ctx().await?,
            footer: self.footer_ctx().await?,
            base_url: self.base_url.as_str(),
            items: cart.items,
        };
        Ok((jar, ctx))
    }

    async fn cart_form(&self, jar: CookieJar, form: templates::CartForm) -> Res<CookieJar> {
        let (jar, user_id) = self.get_or_set_user_id(jar);
        let item = CartItem {
            product_id: form.product_id,
            quantity: form.quantity,
        };
        self.cart.add_item(user_id, item).await?;
        Ok(jar)
    }

    async fn checkout_ctx<'svc>(
        &'svc self,
        order: OrderResult,
    ) -> Res<templates::CheckoutContext<'svc>> {
        Ok(templates::CheckoutContext {
            header: self.header_ctx().await?,
            footer: self.footer_ctx().await?,
            base_url: self.base_url.as_str(),
            order_id: order.order_id,
            shipping_tracking_id: order.shipping_tracking_id,
            shipping_cost: order.shipping_cost,
            shipping_address: order.shipping_address,
            items: order.items,
        })
    }

    async fn checkout_form(
        &self,
        jar: CookieJar,
        form: templates::CheckoutForm,
    ) -> Res<(CookieJar, OrderResult)> {
        let (jar, user_id) = self.get_or_set_user_id(jar);
        let user_currency = "USD".to_string(); // TODO: support user currency selection
        let address = Address {
            street_address: form.street_address,
            city: form.city,
            state: form.state,
            country: form.country,
            zip_code: form.zip_code,
        };
        let credit_card = CreditCardInfo {
            credit_card_number: form.credit_card_number,
            credit_card_ccv: form.credit_card_ccv,
            credit_card_expiration_year: form.credit_card_expiration_year,
            credit_card_expiration_month: form.credit_card_expiration_month,
        };
        let order = self
            .checkout
            .checkout(user_id, user_currency, address, form.email, credit_card)
            .await?;
        Ok((jar, order))
    }
}

impl runtime::Component for FrontendServer {
    type Instance = ();
}

pub fn component() -> ComponentConfig {
    ComponentConfig {
        label: "frontend".to_string(),
        id: FrontendServer::id(),
        binding: BindingType::HttpFixed(8123),
        is_stateful: false,
        entry: || {
            Box::pin(async {
                let server = FrontendServer::new().await;
                server.start().await;
            })
        },
    }
}
