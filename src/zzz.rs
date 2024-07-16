use std::error::Error;
use std::sync::Arc;

use crate::hoyo::{Gift, HoyoClient, Name, now_timestamp};

pub struct ZzzClient {
    client: Arc<HoyoClient>,
}

impl Name for ZzzClient {
    fn name(&self) -> &str { "zzz" }
}

impl Gift for ZzzClient {
    async fn gift(&self, uid: &str, cdkey: &str) -> Result<(), Box<dyn Error>> {
        let now = now_timestamp().as_millis();
        let url = format!("https://public-operation-nap.hoyoverse.com/common/apicdkey/api/webExchangeCdkey?t={now}&lang=en&game_biz=nap_global&uid={uid}&region=prod_gf_jp&cdkey={cdkey}");
        let b = self.client.get::<()>(url.as_str()).await?;
        Ok(b.unwrap())
    }
}

impl ZzzClient {
    pub fn new(client: Arc<HoyoClient>) -> Self {
        Self { client }
    }
}