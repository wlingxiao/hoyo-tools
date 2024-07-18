use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use reqwest::{Client, Method, Request, Url};
use reqwest::cookie::CookieStore;
use reqwest::header::HeaderValue;
use rookie::enums::CookieToString;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

#[derive(Serialize, Deserialize)]
pub struct HoyoResult<T> {
    retcode: i32,
    message: String,
    data: Option<T>,
}

#[derive(Debug)]
pub struct HoyoError {
    pub retcode: i32,
    pub message: String,
}

impl Display for HoyoError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}-{}", self.retcode, self.message)
    }
}

impl Error for HoyoError {}

pub trait DailyInfoData {
    fn is_sign(&self) -> bool;
}

pub trait DailyInfo<T>
where
    T: DailyInfoData,
{
    async fn info(&self) -> Result<Option<T>, Box<dyn Error>>;
}

pub trait Gift {
    async fn gift(&self, uid: &str, cdkey: &str) -> Result<(), Box<dyn Error>>;
}

pub trait Name {
    fn name(&self) -> &str;
}

pub trait DailyCheckIn {
    async fn check_in(&self) -> Result<(), Box<dyn Error>>;
}

pub struct HoyoClient {
    client: Client,
}

impl HoyoClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .read_timeout(Duration::from_secs(10))
            .cookie_provider(Arc::new(BrowserCookieStore { 0: Mutex::new(0) }))
            .build().unwrap();
        Self { client }
    }

    pub async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<Option<T>, Box<dyn Error>> {
        let request = Request::new(Method::GET, Url::parse(url)?);
        self.send(request).await
    }

    pub async fn post<T: DeserializeOwned>(&self, url: &str) -> Result<Option<T>, Box<dyn Error>> {
        let request = Request::new(Method::POST, Url::parse(url)?);
        self.send(request).await
    }

    pub async fn send<T: DeserializeOwned>(&self, mut request: Request) -> Result<Option<T>, Box<dyn Error>> {
        let headers = request.headers_mut();
        headers.insert("Referer", "https://www.hoyoverse.com/en-us".parse().unwrap());
        headers.insert("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36".parse().unwrap());

        let url = request.url().to_owned();
        log::debug!("请求的 url 是: {}", url);

        let resp_ret = self.client
            .execute(request)
            .await;

        match resp_ret.and_then(|resp| resp.error_for_status()) {
            Ok(resp) => {
                match resp.json::<HoyoResult<T>>().await {
                    Ok(ret) => {
                        if ret.retcode != 0 {
                            log::warn!("接口返回的 retcode 不为 0: {} - {} {}", url, ret.retcode, ret.message);
                            return Err(Box::new(HoyoError { retcode: ret.retcode, message: ret.message }));
                        }
                        Ok(ret.data)
                    }
                    Err(e) => {
                        return Err(Box::new(e));
                    }
                }
            }
            Err(e) if e.is_timeout() => {
                log::error!("请求超时了: {}", &url);
                Err(Box::new(e))
            }
            Err(e) if e.is_status() => {
                log::warn!("响应的状态码不是 200: {} {}", url, e.status().unwrap());
                Err(Box::new(e))
            }
            Err(e) => {
                Err(Box::new(e))
            }
        }
    }
}

struct BrowserCookieStore(Mutex<i32>);
impl CookieStore for BrowserCookieStore {
    fn set_cookies(&self, _cookie_headers: &mut dyn Iterator<Item=&HeaderValue>, _url: &Url) {
        // ignore
    }

    fn cookies(&self, url: &Url) -> Option<HeaderValue> {
        let _lock = self.0.lock().unwrap();
        let cs = get_cookie_str(url);
        HeaderValue::from_maybe_shared(bytes::Bytes::from(cs)).ok()
    }
}

fn get_cookie_str(url: &Url) -> String {
    let host = url.host_str().unwrap();
    let split = host
        .split(".")
        .collect::<Vec<&str>>();

    let mut segments = split.iter()
        .rev()
        .take(2);
    let mut parent_domain = String::new();
    let root = segments.next();
    let sub = segments.next();
    if sub.is_some() {
        parent_domain.push_str(sub.unwrap())
    }
    if root.is_some() {
        parent_domain.push('.');
        parent_domain.push_str(root.unwrap())
    }
    let cookies = rookie::chrome(Some(vec![parent_domain.to_string(), host.to_string()]))
        .unwrap();
    let mut new_cookies = Vec::new();
    for c in cookies {
        if is_valid_cookie(&c) {
            new_cookies.push(c);
        }
    }
    let cookie_str = new_cookies.to_string();
    log::debug!("{}/{}: {}", parent_domain, host, &cookie_str);
    cookie_str
}

fn is_valid_cookie(c: &rookie::enums::Cookie) -> bool {
    match c.expires {
        None => { false }
        Some(expires) => {
            Duration::from_secs(expires) > now_timestamp()
        }
    }
}

pub fn now_timestamp() -> Duration {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
}