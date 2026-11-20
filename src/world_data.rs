use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherData {
    pub city: String,
    pub temperature_c: f64,
    pub wind_kmh: f64,
    pub condition: String,
    pub is_live: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub usd_try: f64,
    pub eur_try: f64,
    pub gbp_try: f64,
    pub btc_try: f64,
    pub btc_24h_change: f64,
    pub gold_gram_try: f64,
    pub is_fx_live: bool,
    pub is_crypto_live: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsHeadline {
    pub title: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub fetched_at: String,
    pub read_only_sandbox: bool,
    pub weather: WeatherData,
    pub market: MarketData,
    pub headlines: Vec<NewsHeadline>,
    pub is_news_live: bool,
}

impl Default for WorldSnapshot {
    fn default() -> Self {
        Self {
            fetched_at: Utc::now().to_rfc3339(),
            read_only_sandbox: true,
            weather: WeatherData {
                city: "İstanbul".to_string(),
                temperature_c: 16.5,
                wind_kmh: 14.0,
                condition: "Parçalı Bulutlu".to_string(),
                is_live: false,
            },
            market: MarketData {
                usd_try: 36.20,
                eur_try: 39.10,
                gbp_try: 46.00,
                btc_try: 3_150_000.0,
                btc_24h_change: 1.4,
                gold_gram_try: 3_350.0,
                is_fx_live: false,
                is_crypto_live: false,
            },
            headlines: vec![
                NewsHeadline {
                    title: "İstanbul'da kira ve ulaşım maliyetleri genç çalışanları yeni gelir modelleri aramaya itiyor".to_string(),
                    source: "Sandbox Önbellek".to_string(),
                },
                NewsHeadline {
                    title: "Uzaktan yazılım ve dijital hizmet ihracatında giriş seviyesi talebi artıyor".to_string(),
                    source: "Sandbox Önbellek".to_string(),
                },
                NewsHeadline {
                    title: "Küresel piyasalarda teknoloji hisseleri ve döviz kurları yakından izleniyor".to_string(),
                    source: "Sandbox Önbellek".to_string(),
                },
            ],
            is_news_live: false,
        }
    }
}

pub struct WorldDataFetcher {
    client: Client,
}

impl WorldDataFetcher {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(6))
            .user_agent("zero-to-life-readonly-sandbox/0.1")
            .build()
            .unwrap_or_else(|_| Client::new());
        Self { client }
    }

    /// Strictly READ-ONLY HTTP GET calls to public real-world APIs.
    /// Falls back gracefully to previous/default snapshot if offline or rate-limited.
    pub async fn fetch_snapshot(&self, previous: &WorldSnapshot) -> WorldSnapshot {
        let (weather_res, fx_res, crypto_res, news_res) = tokio::join!(
            self.fetch_weather(),
            self.fetch_fx(),
            self.fetch_crypto(),
            self.fetch_rss_headlines(),
        );

        let weather = weather_res.unwrap_or_else(|| {
            let mut w = previous.weather.clone();
            w.is_live = false;
            w
        });

        let mut market = previous.market.clone();
        if let Some((usd, eur, gbp)) = fx_res {
            market.usd_try = usd;
            market.eur_try = eur;
            market.gbp_try = gbp;
            market.is_fx_live = true;
        } else {
            market.is_fx_live = false;
        }

        if let Some((btc_try, btc_change, gold_gram_try)) = crypto_res {
            market.btc_try = btc_try;
            market.btc_24h_change = btc_change;
            if gold_gram_try > 100.0 {
                market.gold_gram_try = gold_gram_try;
            }
            market.is_crypto_live = true;
        } else {
            market.is_crypto_live = false;
        }

        let (headlines, is_news_live) = match news_res {
            Some(items) if !items.is_empty() => (items, true),
            _ => (previous.headlines.clone(), false),
        };

        WorldSnapshot {
            fetched_at: Utc::now().to_rfc3339(),
            read_only_sandbox: true,
            weather,
            market,
            headlines,
            is_news_live,
        }
    }

    async fn fetch_weather(&self) -> Option<WeatherData> {
        #[derive(Deserialize)]
        struct OpenMeteoResp {
            current: OpenMeteoCurrent,
        }
        #[derive(Deserialize)]
        struct OpenMeteoCurrent {
            temperature_2m: f64,
            weather_code: i32,
            wind_speed_10m: f64,
        }

        let url = "https://api.open-meteo.com/v1/forecast?latitude=41.0082&longitude=28.9784&current=temperature_2m,weather_code,wind_speed_10m";
        let resp = self.client.get(url).send().await.ok()?;
        let data: OpenMeteoResp = resp.json().await.ok()?;
        Some(WeatherData {
            city: "İstanbul".to_string(),
            temperature_c: (data.current.temperature_2m * 10.0).round() / 10.0,
            wind_kmh: (data.current.wind_speed_10m * 10.0).round() / 10.0,
            condition: weather_code_to_tr(data.current.weather_code).to_string(),
            is_live: true,
        })
    }

    async fn fetch_fx(&self) -> Option<(f64, f64, f64)> {
        #[derive(Deserialize)]
        struct FxResp {
            rates: std::collections::HashMap<String, f64>,
        }

        let url = "https://open.er-api.com/v6/latest/USD";
        let resp = self.client.get(url).send().await.ok()?;
        let data: FxResp = resp.json().await.ok()?;
        let usd_try = *data.rates.get("TRY")?;
        let usd_eur = *data.rates.get("EUR")?;
        let usd_gbp = *data.rates.get("GBP")?;
        if usd_eur <= 0.0 || usd_gbp <= 0.0 {
            return None;
        }
        let eur_try = usd_try / usd_eur;
        let gbp_try = usd_try / usd_gbp;
        Some((
            (usd_try * 100.0).round() / 100.0,
            (eur_try * 100.0).round() / 100.0,
            (gbp_try * 100.0).round() / 100.0,
        ))
    }

    async fn fetch_crypto(&self) -> Option<(f64, f64, f64)> {
        let url = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin,tether-gold&vs_currencies=try&include_24hr_change=true";
        let resp = self.client.get(url).send().await.ok()?;
        let val: serde_json::Value = resp.json().await.ok()?;
        let btc_try = val.get("bitcoin")?.get("try")?.as_f64()?;
        let btc_change = val
            .get("bitcoin")
            .and_then(|b| b.get("try_24h_change"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        // tether-gold is 1 troy ounce (31.1035 grams) of gold
        let gold_oz_try = val
            .get("tether-gold")
            .and_then(|g| g.get("try"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let gold_gram_try = if gold_oz_try > 0.0 {
            (gold_oz_try / 31.1035).round()
        } else {
            0.0
        };

        Some((
            btc_try.round(),
            (btc_change * 100.0).round() / 100.0,
            gold_gram_try,
        ))
    }

    async fn fetch_rss_headlines(&self) -> Option<Vec<NewsHeadline>> {
        let url = "https://feeds.bbci.co.uk/turkce/rss.xml";
        let resp = self.client.get(url).send().await.ok()?;
        let body = resp.text().await.ok()?;
        let mut headlines = Vec::new();

        // Simple, dependency-free RSS <item><title>...</title> parser
        for chunk in body.split("<item>").skip(1).take(5) {
            if let Some(title) = extract_xml_tag(chunk, "title") {
                let clean = clean_xml_text(&title);
                if !clean.is_empty() {
                    headlines.push(NewsHeadline {
                        title: clean,
                        source: "BBC Türkçe (RSS)".to_string(),
                    });
                }
            }
        }
        if headlines.is_empty() {
            None
        } else {
            Some(headlines)
        }
    }
}

fn extract_xml_tag(text: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let start = text.find(&open)? + open.len();
    let rest = &text[start..];
    let end = rest.find(&close)?;
    Some(rest[..end].to_string())
}

fn clean_xml_text(raw: &str) -> String {
    let s = raw.trim();
    let s = s
        .strip_prefix("<![CDATA[")
        .and_then(|v| v.strip_suffix("]]>"))
        .unwrap_or(s);
    s.replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .trim()
        .to_string()
}

fn weather_code_to_tr(code: i32) -> &'static str {
    match code {
        0 => "Açık",
        1 | 2 => "Az Bulutlu",
        3 => "Kapalı",
        45 | 48 => "Sisli",
        51 | 53 | 55 => "Çiseleyen Yağmur",
        61 | 63 | 65 => "Yağmurlu",
        71 | 73 | 75 => "Karlı",
        80 | 81 | 82 => "Sağanak Yağışlı",
        95 | 96 | 99 => "Fırtınalı",
        _ => "Değişken",
    }
}
