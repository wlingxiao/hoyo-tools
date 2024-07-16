use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub zzz: Option<GameConfig>,
    pub hsr: Option<GameConfig>,
    pub genshin: Option<GameConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameConfig {
    pub enable: bool,
    pub uid: String,
    pub cdkeys: Option<Vec<String>>,
}

pub async fn parse_config() -> Config {
    let config_txt = tokio::fs::read_to_string("config.json").await.unwrap();
    serde_json::from_str(config_txt.as_str()).unwrap()
}