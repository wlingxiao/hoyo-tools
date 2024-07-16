use std::error::Error;
use std::sync::Arc;

use crate::hoyo::{DailyCheckIn, Gift, HoyoClient, Name};

pub struct GenshinClient {
    client: Arc<HoyoClient>,
}

impl Name for GenshinClient {
    fn name(&self) -> &str { "genshin" }
}

impl Gift for GenshinClient {
    async fn gift(&self, uid: &str, cdkey: &str) -> Result<(), Box<dyn Error>> {
        let url = format!("https://sg-hk4e-api.hoyoverse.com/common/apicdkey/api/webExchangeCdkey?uid={uid}&region=os_asia&lang=zh-tw&cdkey={cdkey}&game_biz=hk4e_global&sLangKey=en-us");
        let ret = self.client.get::<()>(url.as_str()).await?;
        Ok(ret.unwrap())
    }
}

impl DailyCheckIn for GenshinClient {
    async fn check_in(&self) -> Result<(), Box<dyn Error>> {
        let url = "https://sg-hk4e-api.hoyolab.com/event/sol/sign?act_id=e202102251931481";
        self.client.post::<()>(url).await?;
        Ok(())
    }
}

impl GenshinClient {
    pub fn new(client: Arc<HoyoClient>) -> Self {
        Self { client }
    }
}