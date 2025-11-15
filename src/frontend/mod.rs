use amimono::{Binding, BindingType, Component, Runtime};
use axum::{Router, routing::get};

use crate::currencyservice::CurrencyClient;

struct FrontendServer {
    data: FrontendServerData,
}

#[derive(Clone)]
struct FrontendServerData {
    rt: Runtime,
    currency: CurrencyClient,
}

impl FrontendServer {
    async fn new(rt: &Runtime) -> FrontendServer {
        FrontendServer {
            data: FrontendServerData {
                rt: rt.clone(),
                currency: CurrencyClient::new(rt).await,
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

        let app = Router::new().route(
            "/",
            get({
                let data = self.data.clone();
                async move || {
                    let cs = data.currency.get_supported_currencies(&data.rt).await;
                    cs.unwrap().join(", ")
                }
            }),
        );

        let listener = tokio::net::TcpListener::bind(sock).await.unwrap();
        log::info!("frontend listening on {}", sock);
        axum::serve(listener, app).await.unwrap();
    }
}

async fn frontend_main(rt: Runtime) {
    let server = FrontendServer::new(&rt).await;
    server.start(&rt).await
}

pub fn component() -> Component {
    Component::from_async_fn("frontend", BindingType::Http, frontend_main)
}
