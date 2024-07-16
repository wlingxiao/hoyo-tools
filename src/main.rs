use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use reqwest::Url;
use crate::config::GameConfig;
use crate::genshin::GenshinClient;
use crate::hoyo::{DailyCheckIn, DailyInfo, DailyInfoData, Gift, HoyoClient, HoyoError, Name};
use crate::hsr::HsrClient;
use crate::zzz::ZzzClient;

mod config;
mod hoyo;
mod zzz;
mod hsr;
mod genshin;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();
    let hoyo_client = Arc::new(HoyoClient::new());

    let config = config::parse_config().await;
    if let Some(hsr_config) = &config.hsr {
        let client = HsrClient::new(hoyo_client.clone());
        run_daily_info(&client).await;
        run_gift(&client, hsr_config).await;
        run_daily_check_in(&client).await;
    }

    if let Some(zzz_config) = &config.zzz {
        let client = ZzzClient::new(hoyo_client.clone());
        run_gift(&client, zzz_config).await;
    }

    if let Some(genshin_config) = &config.genshin {
        let client = GenshinClient::new(hoyo_client.clone());
        run_gift(&client, genshin_config).await;
        run_daily_check_in(&client).await;
    }

    Ok(())
}

async fn run_gift<T>(client: &T, config: &GameConfig)
where
    T: Gift + Name,
{
    let name = client.name();
    if !config.enable {
        log::info!("{name} 兑换码: 未启用");
        return;
    }
    log::info!("{name} 兑换码: 已启用");
    if let Some(cdkeys) = &config.cdkeys {
        let uid = &config.uid;
        for cdkey in cdkeys {
            let parse_result = Url::parse(cdkey);
            let cdkey: &str = match parse_result {
                Ok(url) => {
                    let mut query_pairs = url.query_pairs();
                    let code_opt = query_pairs.find(|(c, _)| c == "code");
                    match code_opt {
                        Some((_, code)) => { &*code.to_string() }
                        None => cdkey
                    }
                }
                Err(_) => { cdkey }
            };
            let _ = client.gift(uid, cdkey).await;
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
}

async fn run_daily_info<A, T>(client: &T)
where
    A: DailyInfoData,
    T: DailyInfo<A> + Name,
{
    if let Ok(Some(info)) = client.info().await {
        let sign_msg = if info.is_sign() { "已签到" } else { "未签到" };
        let name = client.name();
        log::info!("{name}: 今天{sign_msg}")
    }
}

async fn run_daily_check_in<T>(client: &T)
where
    T: DailyCheckIn + Name,
{
    let name = client.name();
    match client.check_in().await {
        Ok(_) => {
            log::info!("{name} 签到成功");
        }
        Err(e) => {
            if let Ok(e) = e.downcast::<HoyoError>() {
                if e.retcode == -2003 || e.retcode == -5003 {
                    log::info!("{name} 已签到");
                    return;
                }
            }
            log::info!("{name} 签到失败")
        }
    }
}