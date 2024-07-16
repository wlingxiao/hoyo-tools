use std::error::Error;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::hoyo::{DailyCheckIn, DailyInfo, DailyInfoData, Gift, HoyoClient, Name, now_timestamp};

#[derive(Serialize, Deserialize, Debug)]
pub struct HsrDailyInfoData {
    total_sign_day: i32,
    today: String,
    is_sign: bool,
    is_sub: bool,
    region: String,
    sign_cnt_missed: i32,
    short_sign_day: i32,
    send_first: bool,
}

impl DailyInfoData for HsrDailyInfoData {
    fn is_sign(&self) -> bool {
        self.is_sign
    }
}

pub struct HsrClient {
    client: Arc<HoyoClient>,
}

impl DailyInfo<HsrDailyInfoData> for HsrClient {
    async fn info(&self) -> Result<Option<HsrDailyInfoData>, Box<dyn Error>> {
        self.client
            .get::<HsrDailyInfoData>("https://sg-public-api.hoyolab.com/event/luna/os/info?act_id=e202303301540311")
            .await
    }
}

impl Gift for HsrClient {
    async fn gift(&self, uid: &str, cdkey: &str) -> Result<(), Box<dyn Error>> {
        let now = now_timestamp().as_millis();
        let url = format!("https://sg-hkrpg-api.hoyoverse.com/common/apicdkey/api/webExchangeCdkey?t={now}&lang=ja&game_biz=hkrpg_global&uid={uid}&region=prod_official_asia&cdkey={cdkey}");
        let b = self.client.get::<()>(url.as_str()).await?;
        Ok(b.unwrap())
    }
}

impl Name for HsrClient {
    fn name(&self) -> &str { "hsr" }
}

impl DailyCheckIn for HsrClient {
    async fn check_in(&self) -> Result<(), Box<dyn Error>> {
        let url = "https://sg-public-api.hoyolab.com/event/luna/os/sign?act_id=e202303301540311";
        self.client.post::<()>(url).await?;
        Ok(())
    }
}

impl HsrClient {
    pub fn new(client: Arc<HoyoClient>) -> Self {
        Self { client }
    }
}