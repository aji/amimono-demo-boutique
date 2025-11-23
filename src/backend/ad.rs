use std::collections::HashMap;

use amimono::{config::ComponentConfig, rpc::RpcResult};
use rand::seq::IndexedRandom;

use crate::shared::Ad;

mod ops {
    use crate::shared::Ad;

    amimono::rpc_ops! {
        fn get_ads(context_keys: Vec<String>) -> Vec<Ad>;
    }
}

const MAX_ADS_TO_SERVE: usize = 2;

pub struct AdService {
    ads_map: HashMap<String, Vec<Ad>>,
}

impl AdService {
    fn new() -> AdService {
        let ads_map = {
            let hairdryer = Ad::new("2ZYFJ3GM2N", "Hairdryer for sale. 50% off.");
            let tank_top = Ad::new("66VCHSJNUP", "Tank top for sale. 20% off.");
            let candle_holder = Ad::new("0PUK6V6EV0", "Candle holder for sale. 30% off.");
            let bamboo_glass_jar = Ad::new("9SIQT8TOJO", "Bamboo glass jar for sale. 10% off.");
            let watch = Ad::new(
                "1YMWWN1N4O",
                "Watch for sale. Buy one, get second kit for free.",
            );
            let mug = Ad::new(
                "6E92ZMYYFZ",
                "Mug for sale. Buy two, get third one for free.",
            );
            let loafers = Ad::new(
                "L9ECAV7KIM",
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

impl ops::Handler for AdService {
    async fn new() -> Self {
        AdService::new()
    }

    async fn get_ads(&self, context_keys: Vec<String>) -> RpcResult<Vec<Ad>> {
        log::info!("received ad request (context_words={:?})", context_keys);
        let ads = if context_keys.len() > 0 {
            context_keys
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
        Ok(ads)
    }
}

pub type AdClient = ops::Client<AdService>;

pub fn component() -> ComponentConfig {
    ops::component::<AdService>("adservice".to_owned())
}
