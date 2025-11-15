use std::collections::HashMap;

use amimono::{Component, Rpc, RpcClient, Runtime};
use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AdServiceRequest {
    context_keys: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct AdServiceResponse {
    ads: Vec<Ad>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Ad {
    redirect_url: String,
    text: String,
}

impl Ad {
    fn new<S: ToString, T: ToString>(redirect_url: S, text: T) -> Ad {
        Ad {
            redirect_url: redirect_url.to_string(),
            text: text.to_string(),
        }
    }
}

const MAX_ADS_TO_SERVE: usize = 2;

pub struct AdService {
    ads_map: HashMap<String, Vec<Ad>>,
}

impl AdService {
    fn new() -> AdService {
        let ads_map = {
            let hairdryer = Ad::new("todo", "Hairdryer for sale. 50% off.");
            let tank_top = Ad::new("todo", "Tank top for sale. 20% off.");
            let candle_holder = Ad::new("todo", "Candle holder for sale. 30% off.");
            let bamboo_glass_jar = Ad::new("todo", "Bamboo glass jar for sale. 10% off.");
            let watch = Ad::new("todo", "Watch for sale. Buy one, get second kit for free.");
            let mug = Ad::new("todo", "Mug for sale. Buy two, get third one for free.");
            let loafers = Ad::new(
                "todo",
                "Loafers for sale. Buy one, get second one for free.",
            );

            let mut ads_map = HashMap::new();
            ads_map.insert("clothing".to_owned(), vec![tank_top]);
            ads_map.insert("accessories".to_owned(), vec![watch]);
            ads_map.insert("footwear".to_owned(), vec![loafers]);
            ads_map.insert("hair".to_owned(), vec![hairdryer]);
            ads_map.insert("decor".to_owned(), vec![candle_holder]);
            ads_map.insert("kitchen".to_owned(), vec![bamboo_glass_jar, mug]);
            ads_map
        };

        AdService { ads_map }
    }

    fn get_ads_by_category(&self, category: &str) -> Vec<Ad> {
        match self.ads_map.get(category) {
            Some(x) => x.clone(),
            None => Vec::new(),
        }
    }

    fn get_random_ads(&self) -> Vec<Ad> {
        let all_ads: Vec<&Ad> = self.ads_map.values().flat_map(|x| x.iter()).collect();
        (0..MAX_ADS_TO_SERVE)
            .map(|_| *all_ads.choose(&mut rand::rng()).unwrap())
            .cloned()
            .collect()
    }
}

impl Rpc for AdService {
    const LABEL: amimono::Label = "adservice";

    type Request = AdServiceRequest;

    type Response = AdServiceResponse;

    async fn start(_rt: &Runtime) -> Self {
        log::info!("ad service started");
        AdService::new()
    }

    async fn handle(&self, _rt: &Runtime, q: &Self::Request) -> Self::Response {
        log::info!("received ad request (context_words={:?})", q.context_keys);
        let ads = if q.context_keys.len() > 0 {
            q.context_keys
                .iter()
                .flat_map(|k| self.get_ads_by_category(k).into_iter())
                .collect()
        } else {
            self.get_random_ads()
        };
        let ads = if ads.len() == 0 {
            self.get_random_ads()
        } else {
            ads
        };
        AdServiceResponse { ads }
    }
}

pub async fn client(rt: &Runtime) -> RpcClient<AdService> {
    AdService::client(rt).await
}

pub fn component() -> Component {
    AdService::component()
}
