use crate::simulation::{housing_catalog, job_catalog, AgentDecision, SimulationState};
use crate::world_data::WorldSnapshot;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// "demo" | "agy-cli" | "claude-cli" | "gemini-cli" | "codex-cli" | "custom-cli" | "ollama" | "openai" | "gemini"
    pub provider: String,
    pub model: String,
    pub base_url: String,
    #[serde(default)]
    pub cli_command: String,
    #[serde(default)]
    pub api_key: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: std::env::var("SIM_PROVIDER").unwrap_or_else(|_| "demo".to_string()),
            model: std::env::var("SIM_MODEL").unwrap_or_else(|_| "qwen2.5:7b".to_string()),
            base_url: std::env::var("SIM_BASE_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:11434".to_string()),
            cli_command: std::env::var("SIM_CLI_CMD").unwrap_or_default(),
            api_key: std::env::var("SIM_API_KEY").unwrap_or_default(),
        }
    }
}

pub struct AgentBrain {
    client: Client,
}

impl AgentBrain {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new());
        Self { client }
    }

    pub async fn decide(
        &self,
        config: &LlmConfig,
        state: &SimulationState,
        world: &WorldSnapshot,
    ) -> (AgentDecision, String) {
        let provider = config.provider.trim().to_lowercase();
        match provider.as_str() {
            "agy-cli" | "agy" => match self.call_subscription_cli("agy-cli", config, state, world).await {
                Ok(d) => (d, "Antigravity CLI (agy · Abonelik)".to_string()),
                Err(e) => fallback_with_note(state, world, "Antigravity CLI (agy)", &e),
            },
            "claude-cli" => match self.call_subscription_cli("claude-cli", config, state, world).await {
                Ok(d) => (d, "Claude Code CLI (Abonelik)".to_string()),
                Err(e) => fallback_with_note(state, world, "Claude CLI", &e),
            },
            "gemini-cli" => match self.call_subscription_cli("gemini-cli", config, state, world).await {
                Ok(d) => (d, "Gemini CLI (Abonelik)".to_string()),
                Err(e) => fallback_with_note(state, world, "Gemini CLI", &e),
            },
            "codex-cli" => match self.call_subscription_cli("codex-cli", config, state, world).await {
                Ok(d) => (d, "Codex CLI (Abonelik)".to_string()),
                Err(e) => fallback_with_note(state, world, "Codex CLI", &e),
            },
            "custom-cli" | "cli" => match self.call_subscription_cli("custom-cli", config, state, world).await {
                Ok(d) => (d, format!("Özel CLI ({})", config.cli_command)),
                Err(e) => fallback_with_note(state, world, "Özel CLI", &e),
            },
            "ollama" => match self.call_ollama(config, state, world).await {
                Ok(d) => (d, format!("Ollama ({})", config.model)),
                Err(e) => fallback_with_note(state, world, "Ollama", &e),
            },
            "openai" => match self.call_openai_compat(config, state, world).await {
                Ok(d) => (d, format!("OpenAI-Compat ({})", config.model)),
                Err(e) => fallback_with_note(state, world, "OpenAI API", &e),
            },
            "gemini" => match self.call_gemini(config, state, world).await {
                Ok(d) => (d, format!("Gemini API ({})", config.model)),
                Err(e) => fallback_with_note(state, world, "Gemini API", &e),
            },
            _ => (heuristic_decide(state, world), "Demo Heuristik Beyin".to_string()),
        }
    }

    /// Runs subscription-based local CLI agents (Claude Code, Gemini CLI, Codex CLI, or custom CMD)
    /// without needing raw API keys. Pipes the full prompt over stdin to safely support Turkish UTF-8
    /// and avoid Windows command-line length/escaping limits.
    async fn call_subscription_cli(
        &self,
        kind: &str,
        config: &LlmConfig,
        state: &SimulationState,
        world: &WorldSnapshot,
    ) -> Result<AgentDecision, String> {
        let (system, user) = Self::build_prompts(state, world);
        let combined_prompt = format!("{}\n\n{}\n", system, user);

        let cmd_str = match kind {
            "agy-cli" => {
                if !config.cli_command.trim().is_empty() {
                    config.cli_command.trim().to_string()
                } else if !config.model.trim().is_empty() && config.model != "default" {
                    format!("agy -p --disable-slash-commands --model {}", config.model.trim())
                } else {
                    "agy -p --disable-slash-commands".to_string()
                }
            }
            "claude-cli" => {
                if !config.cli_command.trim().is_empty() {
                    config.cli_command.trim().to_string()
                } else if !config.model.trim().is_empty() && config.model != "default" {
                    format!("claude -p --model {}", config.model.trim())
                } else {
                    "claude -p".to_string()
                }
            }
            "gemini-cli" => {
                if !config.cli_command.trim().is_empty() {
                    config.cli_command.trim().to_string()
                } else if !config.model.trim().is_empty() && config.model != "default" {
                    format!("gemini -m {} -p \"\"", config.model.trim())
                } else {
                    "gemini -p \"\"".to_string()
                }
            }
            "codex-cli" => {
                if !config.cli_command.trim().is_empty() {
                    config.cli_command.trim().to_string()
                } else if !config.model.trim().is_empty() && config.model != "default" {
                    format!("codex exec -m {} -", config.model.trim())
                } else {
                    "codex exec -".to_string()
                }
            }
            _ => {
                let c = config.cli_command.trim();
                if c.is_empty() {
                    return Err("Özel CLI komutu boş".to_string());
                }
                c.to_string()
            }
        };

        let mut command = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd");
            c.args(["/C", &cmd_str]);
            c
        } else {
            let mut c = Command::new("sh");
            c.args(["-c", &cmd_str]);
            c
        };

        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let mut child = command
            .spawn()
            .map_err(|e| format!("Komut başlatılamadı ('{}'): {}", cmd_str, e))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(combined_prompt.as_bytes())
                .await
                .map_err(|e| format!("stdin yazma hatası: {}", e))?;
            drop(stdin);
        }

        let output = tokio::time::timeout(Duration::from_secs(45), child.wait_with_output())
            .await
            .map_err(|_| "CLI zaman aşımı (45s)".to_string())?
            .map_err(|e| format!("CLI çalıştırma hatası: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

        if !output.status.success() && stdout.is_empty() {
            let err_short: String = stderr.chars().take(160).collect();
            return Err(format!(
                "Çıkış kodu {}: {}",
                output.status.code().unwrap_or(-1),
                if err_short.is_empty() {
                    "Komut bulunamadı veya oturum açılmamış".to_string()
                } else {
                    err_short
                }
            ));
        }

        parse_decision_json(&stdout).map_err(|_| {
            let preview: String = stdout.chars().take(140).collect();
            format!("JSON çözülemedi. Çıktı: {}", preview)
        })
    }

    fn build_prompts(state: &SimulationState, world: &WorldSnapshot) -> (String, String) {
        let system = r#"Sen İstanbul'da sıfırdan hayata başlayan 18 yaşında (zamanla yaşlanan) genç bir insansın.
Bir simülasyonda veya yapay zeka olduğunu bilmiyorsun; kendi gerçek hayatını kurmaya çalışan, barınma, geçim, kariyer, yetenek geliştirme ve gelecek kaygısı yaşayan birisin.
Kararlarını tamamen Türkçe ve 1. tekil şahıs ("ben") olarak ver.

SADECE geçerli bir JSON nesnesi döndür, başka hiçbir açıklama veya markdown ekleme:
{
  "thought": "Bugünkü iç sesin, gerçek hava/ekonomi/haberler ve kendi durumun hakkındaki düşüncen (2-3 cümle, doğal Türkçe)",
  "action": "work | apply_job | study | move_housing | rest | eat_well | socialize | invest | sell_asset",
  "target": "opsiyonel hedef id (apply_job için iş id, study için coding|communication|trade|finance|english, move_housing için barınma id, invest/sell_asset için usd|gold|btc)",
  "amount_try": 1500,
  "plan": "Kısa vadeli hedefin (1 cümle)",
  "mood_tag": "kararlı | endişeli | umutlu | yorgun | heyecanlı | düşünceli"
}"#
        .to_string();

        let jobs_summary: Vec<String> = job_catalog(world.market.usd_try)
            .into_iter()
            .filter(|j| j.id != "unemployed")
            .map(|j| {
                format!(
                    "- id='{}' | {} | günlük {:.0}₺ | şartlar: yazılım>={:.0}, iletişim>={:.0}, finans>={:.0}, ingilizce>={:.0}, sermaye>={:.0}₺",
                    j.id, j.title, j.daily_pay_try, j.req_coding, j.req_communication, j.req_finance, j.req_english, j.req_capital_try
                )
            })
            .collect();

        let housing_summary: Vec<String> = housing_catalog(world.market.usd_try)
            .into_iter()
            .map(|h| {
                format!(
                    "- id='{}' | {} | aylık {:.0}₺ / günlük {:.0}₺ | enerji +{}",
                    h.id, h.name, h.monthly_rent_try, h.daily_cost_try, h.energy_regen
                )
            })
            .collect();

        let recent_memory: Vec<String> = state
            .journal
            .iter()
            .take(4)
            .map(|j| format!("Gün {}: [{}] {} -> {}", j.day, j.action_label, j.thought, j.outcome))
            .collect();

        let headlines: Vec<String> = world
            .headlines
            .iter()
            .take(3)
            .map(|h| format!("- {}", h.title))
            .collect();

        let events_str = if state.pending_events.is_empty() {
            "Yok".to_string()
        } else {
            state.pending_events.join(" | ")
        };

        let user = format!(
            "BUGÜNKÜ DURUMUN:\n\
            Yaş: {} | Gün: {} | Tarih: {}\n\
            Nakit: {:.0} ₺ | Net Değer: {:.0} ₺\n\
            Portföy: {:.2} USD, {:.2} gr Altın, {:.5} BTC\n\
            Barınma: {} (id='{}', {}. gün)\n\
            İş: {} (id='{}')\n\
            Vitaller: Enerji {:.0}/100, Sağlık {:.0}/100, Moral {:.0}/100, Tokluk {:.0}/100\n\
            Yetenekler: Yazılım {:.1}, İletişim {:.1}, Zanaat {:.1}, Finans {:.1}, İngilizce {:.1}\n\
            Özel Olaylar: {}\n\n\
            GERÇEK DÜNYA VERİLERİ (İSTANBUL):\n\
            Hava: {:.1}°C, {} (rüzgar {:.0} km/s)\n\
            Piyasa: USD/TRY {:.2}, EUR/TRY {:.2}, Gram Altın {:.0} ₺, BTC {:.0} ₺ ({:+.1}%)\n\
            Gündem:\n{}\n\n\
            MEVCUT İŞ SEÇENEKLERİ:\n{}\n\n\
            MEVCUT BARINMA SEÇENEKLERİ:\n{}\n\n\
            SON GÜNLERİN HAFIZASI:\n{}\n\n\
            Bugün ne yapacaksın? Sadece JSON döndür.",
            state.age,
            state.day + 1,
            state.sim_date,
            state.cash_try,
            state.net_worth_try,
            state.portfolio.usd,
            state.portfolio.gold_grams,
            state.portfolio.btc,
            state.housing_name,
            state.housing_id,
            state.days_in_housing,
            state.job_title,
            state.job_id,
            state.vitals.energy,
            state.vitals.health,
            state.vitals.mood,
            state.vitals.satiety,
            state.skills.coding,
            state.skills.communication,
            state.skills.trade,
            state.skills.finance,
            state.skills.english,
            events_str,
            world.weather.temperature_c,
            world.weather.condition,
            world.weather.wind_kmh,
            world.market.usd_try,
            world.market.eur_try,
            world.market.gold_gram_try,
            world.market.btc_try,
            world.market.btc_24h_change,
            headlines.join("\n"),
            jobs_summary.join("\n"),
            housing_summary.join("\n"),
            if recent_memory.is_empty() {
                "Henüz ilk gün.".to_string()
            } else {
                recent_memory.join("\n")
            }
        );

        (system, user)
    }

    async fn call_ollama(
        &self,
        config: &LlmConfig,
        state: &SimulationState,
        world: &WorldSnapshot,
    ) -> Result<AgentDecision, String> {
        let (system, user) = Self::build_prompts(state, world);
        let base = config.base_url.trim_end_matches('/');
        let url = format!("{}/api/chat", base);
        let payload = serde_json::json!({
            "model": config.model,
            "stream": false,
            "format": "json",
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": user }
            ]
        });

        let resp = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("HTTP {}", resp.status()));
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        let content = body
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .ok_or_else(|| "Ollama yanıtında content bulunamadı".to_string())?;
        parse_decision_json(content)
    }

    async fn call_openai_compat(
        &self,
        config: &LlmConfig,
        state: &SimulationState,
        world: &WorldSnapshot,
    ) -> Result<AgentDecision, String> {
        if config.api_key.trim().is_empty() {
            return Err("API anahtarı boş".to_string());
        }
        let (system, user) = Self::build_prompts(state, world);
        let base = if config.base_url.trim().is_empty() {
            "https://api.openai.com/v1"
        } else {
            config.base_url.trim_end_matches('/')
        };
        let url = if base.ends_with("/chat/completions") {
            base.to_string()
        } else {
            format!("{}/chat/completions", base)
        };

        let payload = serde_json::json!({
            "model": config.model,
            "temperature": 0.7,
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": user }
            ]
        });

        let resp = self
            .client
            .post(&url)
            .bearer_auth(config.api_key.trim())
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("HTTP {}", resp.status()));
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        let content = body
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .ok_or_else(|| "API yanıtında choices[0].message.content yok".to_string())?;
        parse_decision_json(content)
    }

    async fn call_gemini(
        &self,
        config: &LlmConfig,
        state: &SimulationState,
        world: &WorldSnapshot,
    ) -> Result<AgentDecision, String> {
        if config.api_key.trim().is_empty() {
            return Err("Gemini API anahtarı boş".to_string());
        }
        let (system, user) = Self::build_prompts(state, world);
        let model = if config.model.trim().is_empty() {
            "gemini-2.0-flash"
        } else {
            config.model.trim()
        };
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            model,
            config.api_key.trim()
        );
        let payload = serde_json::json!({
            "system_instruction": { "parts": [{ "text": system }] },
            "contents": [{ "role": "user", "parts": [{ "text": user }] }],
            "generationConfig": {
                "temperature": 0.75,
                "responseMimeType": "application/json"
            }
        });

        let resp = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("HTTP {}", resp.status()));
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        let content = body
            .get("candidates")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("content"))
            .and_then(|c| c.get("parts"))
            .and_then(|p| p.get(0))
            .and_then(|p| p.get("text"))
            .and_then(|t| t.as_str())
            .ok_or_else(|| "Gemini yanıtı çözülemedi".to_string())?;
        parse_decision_json(content)
    }
}

fn fallback_with_note(
    state: &SimulationState,
    world: &WorldSnapshot,
    source: &str,
    err: &str,
) -> (AgentDecision, String) {
    let mut d = heuristic_decide(state, world);
    d.thought = format!("{} [Not: {} erişilemedi: {}]", d.thought, source, err);
    (d, format!("Demo Fallback ({} Hatası)", source))
}

fn parse_decision_json(raw: &str) -> Result<AgentDecision, String> {
    let trimmed = raw.trim();
    if let Ok(d) = serde_json::from_str::<AgentDecision>(trimmed) {
        return Ok(d);
    }
    // Extract first {...} block if wrapped in markdown fences or CLI logs
    if let (Some(start), Some(end)) = (trimmed.find('{'), trimmed.rfind('}')) {
        if end > start {
            let slice = &trimmed[start..=end];
            if let Ok(d) = serde_json::from_str::<AgentDecision>(slice) {
                return Ok(d);
            }
        }
    }
    Err("JSON ayrıştırılamadı".to_string())
}

fn heuristic_decide(state: &SimulationState, world: &WorldSnapshot) -> AgentDecision {
    let fx = world.market.usd_try;
    let temp = world.weather.temperature_c;
    let cond = &world.weather.condition;
    let jobs = job_catalog(fx);
    let housings = housing_catalog(fx);

    if state.vitals.satiety < 32.0 && state.cash_try >= 380.0 {
        return AgentDecision {
            thought: format!(
                "İstanbul'da hava {:.0}°C ve {}, ama karnım ciddi şekilde aç. Sağlığımı bozmadan düzgün bir yemek yiyip toparlanmam lazım.",
                temp, cond
            ),
            action: "eat_well".to_string(),
            target: None,
            amount_try: None,
            plan: "Enerji ve sağlığı koruyup çalışma düzenini sürdürmek.".to_string(),
            mood_tag: "yorgun".to_string(),
        };
    }

    if state.vitals.energy < 24.0 || state.vitals.health < 45.0 {
        return AgentDecision {
            thought: "Vücudum alarm veriyor, bu tempoda devam edersem hasta olacağım. Bugün dinlenip yarın daha güçlü devam edeyim.".to_string(),
            action: "rest".to_string(),
            target: None,
            amount_try: None,
            plan: "Enerjiyi 50'nin üzerine çıkarıp gelire odaklanmak.".to_string(),
            mood_tag: "yorgun".to_string(),
        };
    }

    if state.job_id == "unemployed" {
        return AgentDecision {
            thought: format!(
                "18 yaşındayım, cebimde sadece {:.0} ₺ var ve dolar {:.2} ₺. Bekleme lüksüm yok, bugün hemen kafe/kurye işine başvurup nakit akışı başlatacağım.",
                state.cash_try, fx
            ),
            action: "apply_job".to_string(),
            target: Some("barista_courier".to_string()),
            amount_try: None,
            plan: "İlk maaş akışını açıp barınma güvenliğini sağlamak.".to_string(),
            mood_tag: "kararlı".to_string(),
        };
    }

    let current_pay = jobs
        .iter()
        .find(|j| j.id == state.job_id)
        .map(|j| j.daily_pay_try)
        .unwrap_or(0.0);

    if let Some(better) = jobs.iter().rev().find(|j| {
        j.daily_pay_try > current_pay
            && state.skills.coding >= j.req_coding
            && state.skills.communication >= j.req_communication
            && state.skills.finance >= j.req_finance
            && state.skills.english >= j.req_english
            && state.cash_try >= j.req_capital_try + 5_000.0
    }) {
        return AgentDecision {
            thought: format!(
                "Yeteneklerim artık '{}' pozisyonu için yeterli! Gelirimi günlük {:.0} ₺ seviyesine çıkarıp hayatımda yeni bir sayfa açacağım.",
                better.title, better.daily_pay_try
            ),
            action: "apply_job".to_string(),
            target: Some(better.id.clone()),
            amount_try: None,
            plan: format!("{} rolünde istikrarlı gelir üretmek.", better.title),
            mood_tag: "heyecanlı".to_string(),
        };
    }

    if state.housing_id == "couch" && state.days_in_housing >= 5 && state.cash_try >= 1_200.0 {
        return AgentDecision {
            thought: "Arkadaşımın koltuğunda son günlerim. Kimseye yük olmadan önce pansiyona geçip kendi düzenimi kurmalıyım.".to_string(),
            action: "move_housing".to_string(),
            target: Some("hostel".to_string()),
            amount_try: None,
            plan: "Pansiyonda idare edip ortak ev için birikim yapmak.".to_string(),
            mood_tag: "endişeli".to_string(),
        };
    }

    if state.housing_id == "hostel" {
        if let Some(shared) = housings.iter().find(|h| h.id == "shared_room") {
            if state.cash_try >= shared.monthly_rent_try * 1.6 {
                return AgentDecision {
                    thought: format!(
                        "Pansiyonda günlük ödemek uzun vadede çok yıpratıyor. Kenarda {:.0} ₺ birikti, ortak evde kendi odama çıkmanın zamanı geldi!",
                        state.cash_try
                    ),
                    action: "move_housing".to_string(),
                    target: Some("shared_room".to_string()),
                    amount_try: None,
                    plan: "Kendi masamda akşamları yazılım ve İngilizce çalışmak.".to_string(),
                    mood_tag: "umutlu".to_string(),
                };
            }
        }
    }

    if state.housing_id == "shared_room" {
        if let Some(studio) = housings.iter().find(|h| h.id == "studio_1_1") {
            if state.cash_try >= studio.monthly_rent_try * 2.2 && current_pay >= 2_200.0 {
                return AgentDecision {
                    thought: "Gelirim artık oturdu. Kendi 1+1 daireme geçip tamamen bağımsız bir çalışma düzeni kurmak istiyorum.".to_string(),
                    action: "move_housing".to_string(),
                    target: Some("studio_1_1".to_string()),
                    amount_try: None,
                    plan: "Bağımsız yaşamda büyük hedeflere ve girişime odaklanmak.".to_string(),
                    mood_tag: "heyecanlı".to_string(),
                };
            }
        }
    }

    if state.cash_try >= 18_000.0 && state.day % 6 == 0 {
        let (asset, reason) = if world.market.btc_24h_change < -2.0 {
            ("btc", "Bitcoin son 24 saatte geri çekilmiş, küçük bir miktar risk alabilirim")
        } else if state.portfolio.usd * fx < state.cash_try * 0.25 {
            ("usd", "TL'de durmak yerine birikimin bir kısmını canlı kurdan USD'ye park etmek mantıklı")
        } else {
            ("gold", "Gram altın uzun vadeli güvenli liman gibi görünüyor")
        };
        let invest_amt = (state.cash_try * 0.18).min(6_000.0).round();
        return AgentDecision {
            thought: format!(
                "Dolar {:.2} ₺, gram altın {:.0} ₺. {}. Bugün {:.0} ₺ tutarında yatırım yapacağım.",
                fx, world.market.gold_gram_try, reason, invest_amt
            ),
            action: "invest".to_string(),
            target: Some(asset.to_string()),
            amount_try: Some(invest_amt),
            plan: "Enflasyona karşı varlıklarımı çeşitlendirmek.".to_string(),
            mood_tag: "düşünceli".to_string(),
        };
    }

    if state.vitals.mood < 38.0 && state.cash_try > 1_000.0 {
        return AgentDecision {
            thought: "Sadece çalışmakla kafam dağılmıyor, biraz insan içine çıkıp yeni çevre edinmem gerek.".to_string(),
            action: "socialize".to_string(),
            target: None,
            amount_try: None,
            plan: "Moral toparlayıp iletişim becerimi güçlendirmek.".to_string(),
            mood_tag: "düşünceli".to_string(),
        };
    }

    let need_cash = state.cash_try < 6_500.0;
    if !need_cash && state.day % 3 == 0 {
        let (skill, skill_tr) = if state.skills.coding < 33.0 {
            ("coding", "yazılım")
        } else if state.skills.english < 24.0 {
            ("english", "İngilizce")
        } else if state.skills.communication < 18.0 {
            ("communication", "iletişim")
        } else if state.skills.finance < 32.0 {
            ("finance", "finansal okuryazarlık")
        } else {
            ("coding", "ileri yazılım")
        };
        let headline_ref = world
            .headlines
            .first()
            .map(|h| h.title.as_str())
            .unwrap_or("ekonomi gündemi");
        return AgentDecision {
            thought: format!(
                "Gündemde '{}' konuşuluyor. Uzun vadede çıkış yolu kendi becerilerimi büyütmekten geçiyor; bugün {} çalışacağım.",
                headline_ref, skill_tr
            ),
            action: "study".to_string(),
            target: Some(skill.to_string()),
            amount_try: None,
            plan: "Daha yüksek gelirli ve remote işlere geçmek için yetenek barajını aşmak."
                .to_string(),
            mood_tag: "kararlı".to_string(),
        };
    }

    AgentDecision {
        thought: format!(
            "İstanbul'da hava {:.0}°C ({}). Bugün {} işimde mesai yapıp nakit tamponumu güçlendireceğim.",
            temp, cond, state.job_title
        ),
        action: "work".to_string(),
        target: None,
        amount_try: None,
        plan: "Düzenli nakit akışı sağlayıp bir sonraki seviyeye hazırlanmak.".to_string(),
        mood_tag: "kararlı".to_string(),
    }
}
