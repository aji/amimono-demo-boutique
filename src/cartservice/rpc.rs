use amimono::{Component, Rpc, RpcClient, RpcHandler, Runtime};
use serde::{Deserialize, Serialize};

use super::{Cart, CartItem, CartService};

#[derive(Serialize, Deserialize)]
enum CartRequest {
    AddItem(String, CartItem),
    GetCart(String),
    EmptyCart(String),
}

#[derive(Serialize, Deserialize)]
enum CartResponse {
    Empty,
    Cart(Cart),
}

struct CartServiceRpc(CartService);

impl Rpc for CartServiceRpc {
    const LABEL: amimono::Label = "cartservice";

    type Handler = Self;
    type Client = RpcClient<Self>;

    async fn start(rt: &Runtime) -> CartServiceRpc {
        CartServiceRpc(CartService::start(rt).await)
    }
}

impl RpcHandler for CartServiceRpc {
    type Request = CartRequest;
    type Response = CartResponse;

    async fn handle(&self, rt: &Runtime, q: Self::Request) -> Self::Response {
        match q {
            CartRequest::AddItem(user_id, item) => {
                self.0.add_item(rt, user_id.as_str(), &item).await;
                CartResponse::Empty
            }
            CartRequest::GetCart(user_id) => {
                let cart = self.0.get_cart(rt, user_id.as_str()).await;
                CartResponse::Cart(cart)
            }
            CartRequest::EmptyCart(user_id) => {
                self.0.empty_cart(rt, user_id.as_str()).await;
                CartResponse::Empty
            }
        }
    }
}

pub struct CartClient(RpcClient<CartServiceRpc>);

impl CartClient {
    pub async fn new(rt: &Runtime) -> CartClient {
        CartClient(CartServiceRpc::client(rt).await)
    }

    pub async fn add_item(&self, rt: &Runtime, user_id: &str, item: &CartItem) -> Result<(), ()> {
        let req = CartRequest::AddItem(user_id.to_owned(), item.clone());
        match self.0.handle(rt, req).await {
            Ok(CartResponse::Empty) => Ok(()),
            _ => Err(()),
        }
    }

    pub async fn get_cart(&self, rt: &Runtime, user_id: &str) -> Result<Cart, ()> {
        let req = CartRequest::GetCart(user_id.to_owned());
        match self.0.handle(rt, req).await {
            Ok(CartResponse::Cart(cart)) => Ok(cart),
            _ => Err(()),
        }
    }

    pub async fn empty_cart(&self, rt: &Runtime, user_id: &str) -> Result<(), ()> {
        let req = CartRequest::EmptyCart(user_id.to_owned());
        match self.0.handle(rt, req).await {
            Ok(CartResponse::Empty) => Ok(()),
            _ => Err(()),
        }
    }
}

pub fn component() -> Component {
    CartServiceRpc::component()
}
