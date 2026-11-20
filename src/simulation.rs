use crate::world_data::WorldSnapshot;
use chrono::{Duration as ChronoDuration, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vitals {
    pub energy: f64,  // 0..100
    pub health: f64,  // 0..100
    pub mood: f64,    // 0..100
    pub satiety: f64, // 0..100 (tokluk)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skills {
    pub coding: f64,        // Yazılım
    pub communication: f64, // İletişim & Satış
    pub trade: f64,         // Zanaat & Hizmet
    pub finance: f64,       // Finansal Okuryazarlık
    pub english: f64,       // İngilizce
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Portfolio {
    pub usd: f64,
    pub gold_grams: f64,
    pub btc: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HousingOption {
    pub id: String,
    pub name: String,
    pub monthly_rent_try: f64,
    pub daily_cost_try: f64,
    pub energy_regen: f64,
    pub mood_bonus: f64,
    pub max_days: Option<u32>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobOption {
    pub id: String,
    pub title: String,
    pub daily_pay_try: f64,
    pub usd_indexed: bool,
    pub energy_cost: f64,
    pub req_coding: f64,
    pub req_communication: f64,
    pub req_trade: f64,
    pub req_finance: f64,
    pub req_english: f64,
    pub req_capital_try: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDecision {
    pub thought: String,
    pub action: String,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub amount_try: Option<f64>,
    #[serde(default)]
    pub plan: String,
    #[serde(default)]
    pub mood_tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub day: u32,
    pub sim_date: String,
    pub age: u32,
    pub thought: String,
    pub action: String,
    pub action_label: String,
    pub outcome: String,
    pub plan: String,
    pub mood_tag: String,
    pub cash_delta_try: f64,
    pub net_worth_try: f64,
    pub provider_used: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetWorthPoint {
    pub day: u32,
    pub cash_try: f64,
    pub net_worth_try: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationState {
    pub agent_name: String,
    pub city: String,
    pub age: u32,
    pub day: u32,
    pub sim_date: String,
    pub alive: bool,
    pub status_banner: String,
    pub cash_try: f64,
    pub portfolio: Portfolio,
    pub net_worth_try: f64,
    pub housing_id: String,
    pub housing_name: String,
    pub days_in_housing: u32,
    pub job_id: String,
    pub job_title: String,
    pub vitals: Vitals,
    pub skills: Skills,
    pub milestones: Vec<String>,
    pub journal: Vec<JournalEntry>,
    pub history: Vec<NetWorthPoint>,
    pub pending_events: Vec<String>,
    pub total_earned_try: f64,
    pub total_spent_try: f64,
}

pub fn housing_catalog(usd_try: f64) -> Vec<HousingOption> {
    let fx_scale = (usd_try / 36.0).clamp(0.85, 2.5);
    vec![
        HousingOption {
            id: "couch".to_string(),
            name: "Arkadaş Koltuğu (Geçici)".to_string(),
            monthly_rent_try: 0.0,
            daily_cost_try: 0.0,
            energy_regen: 6.0,
            mood_bonus: -3.0,
            max_days: Some(7),
            description: "Ücretsiz ama en fazla 7 gün kalabilirsin; sonra mutlaka yer bulmalısın."
                .to_string(),
        },
        HousingOption {
            id: "hostel".to_string(),
            name: "Pansiyon / Paylaşımlı Yurt".to_string(),
            monthly_rent_try: 0.0,
            daily_cost_try: (420.0 * fx_scale).round(),
            energy_regen: 12.0,
            mood_bonus: -1.0,
            max_days: None,
            description: "Günlük ödenir, depozito istemez ama uzun vadede yorucu.".to_string(),
        },
        HousingOption {
            id: "shared_room".to_string(),
            name: "Ortak Evde Oda".to_string(),
            monthly_rent_try: (11_500.0 * fx_scale).round(),
            daily_cost_try: 0.0,
            energy_regen: 18.0,
            mood_bonus: 2.0,
            max_days: None,
            description: "Aylık kira ödenir. Kendi odan ve çalışma masan var.".to_string(),
        },
        HousingOption {
            id: "studio_1_1".to_string(),
            name: "1+1 Kiralık Daire".to_string(),
            monthly_rent_try: (22_000.0 * fx_scale).round(),
            daily_cost_try: 0.0,
            energy_regen: 24.0,
            mood_bonus: 5.0,
            max_days: None,
            description: "Tam bağımsız yaşam alanı, yüksek odaklanma ve dinlenme.".to_string(),
        },
        HousingOption {
            id: "modern_loft".to_string(),
            name: "Modern Home-Office Stüdyo".to_string(),
            monthly_rent_try: (36_000.0 * fx_scale).round(),
            daily_cost_try: 0.0,
            energy_regen: 30.0,
            mood_bonus: 8.0,
            max_days: None,
            description: "Yüksek yaşam kalitesi, güçlü çalışma ortamı.".to_string(),
        },
    ]
}

pub fn job_catalog(usd_try: f64) -> Vec<JobOption> {
    let fx_scale = (usd_try / 36.0).clamp(0.85, 2.5);
    vec![
        JobOption {
            id: "unemployed".to_string(),
            title: "İşsiz (İş Arıyor)".to_string(),
            daily_pay_try: 0.0,
            usd_indexed: false,
            energy_cost: 0.0,
            req_coding: 0.0,
            req_communication: 0.0,
            req_trade: 0.0,
            req_finance: 0.0,
            req_english: 0.0,
            req_capital_try: 0.0,
            description: "Gelir yok. Acilen bir iş veya gelir akışı bulmalısın.".to_string(),
        },
        JobOption {
            id: "barista_courier".to_string(),
            title: "Kafe Baristası / Kurye".to_string(),
            daily_pay_try: (980.0 * fx_scale).round(),
            usd_indexed: false,
            energy_cost: 28.0,
            req_coding: 0.0,
            req_communication: 0.0,
            req_trade: 0.0,
            req_finance: 0.0,
            req_english: 0.0,
            req_capital_try: 0.0,
            description: "Giriş seviyesi, hemen başlanabilir. Fiziksel olarak yorucu.".to_string(),
        },
        JobOption {
            id: "sales_support".to_string(),
            title: "Satış & Müşteri Temsilcisi".to_string(),
            daily_pay_try: (1_350.0 * fx_scale).round(),
            usd_indexed: false,
            energy_cost: 22.0,
            req_coding: 0.0,
            req_communication: 16.0,
            req_trade: 0.0,
            req_finance: 0.0,
            req_english: 0.0,
            req_capital_try: 0.0,
            description: "İletişim >= 16 gerektirir. Daha dengeli gelir.".to_string(),
        },
        JobOption {
            id: "freelance_digital".to_string(),
            title: "Freelance İçerik / Çeviri / Tasarım".to_string(),
            daily_pay_try: (1_850.0 * fx_scale).round(),
            usd_indexed: true,
            energy_cost: 18.0,
            req_coding: 0.0,
            req_communication: 18.0,
            req_trade: 0.0,
            req_finance: 0.0,
            req_english: 20.0,
            req_capital_try: 0.0,
            description: "İngilizce >= 20 ve İletişim >= 18. Döviz kuruna duyarlı gelir."
                .to_string(),
        },
        JobOption {
            id: "junior_dev".to_string(),
            title: "Junior Yazılım Geliştirici (Remote)".to_string(),
            daily_pay_try: (2_900.0 * fx_scale).round(),
            usd_indexed: true,
            energy_cost: 17.0,
            req_coding: 32.0,
            req_communication: 12.0,
            req_trade: 0.0,
            req_finance: 0.0,
            req_english: 22.0,
            req_capital_try: 0.0,
            description: "Yazılım >= 32, İngilizce >= 22, İletişim >= 12. Güçlü remote maaş."
                .to_string(),
        },
        JobOption {
            id: "micro_saas".to_string(),
            title: "Kendi Mikro SaaS / Bağımsız Girişim".to_string(),
            daily_pay_try: (5_200.0 * fx_scale).round(),
            usd_indexed: true,
            energy_cost: 20.0,
            req_coding: 55.0,
            req_communication: 25.0,
            req_trade: 0.0,
            req_finance: 30.0,
            req_english: 30.0,
            req_capital_try: 25_000.0,
            description: "Yazılım >= 55, Finans >= 30, İngilizce >= 30, Sermaye >= 25.000 ₺."
                .to_string(),
        },
    ]
}

impl SimulationState {
    pub fn new(world: &WorldSnapshot) -> Self {
        let start_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let initial_cash = 2_500.0;
        let mut s = Self {
            agent_name: "Deniz (18)".to_string(),
            city: "İstanbul".to_string(),
            age: 18,
            day: 0,
            sim_date: start_date.format("%Y-%m-%d").to_string(),
            alive: true,
            status_banner: "18 yaşında, cebinde 2.500 ₺ ve arkadaşının koltuğunda 7 günlük süreyle sıfırdan hayata başladı.".to_string(),
            cash_try: initial_cash,
            portfolio: Portfolio::default(),
            net_worth_try: initial_cash,
            housing_id: "couch".to_string(),
            housing_name: "Arkadaş Koltuğu (Geçici)".to_string(),
            days_in_housing: 0,
            job_id: "unemployed".to_string(),
            job_title: "İşsiz (İş Arıyor)".to_string(),
            vitals: Vitals {
                energy: 82.0,
                health: 92.0,
                mood: 70.0,
                satiety: 75.0,
            },
            skills: Skills {
                coding: 6.0,
                communication: 8.0,
                trade: 5.0,
                finance: 4.0,
                english: 10.0,
            },
            milestones: vec![
                "Gün 0: 18 yaşında İstanbul'da sıfırdan hayata başladı (2.500 ₺ nakit).".to_string(),
            ],
            journal: vec![],
            history: vec![NetWorthPoint {
                day: 0,
                cash_try: initial_cash,
                net_worth_try: initial_cash,
            }],
            pending_events: vec![],
            total_earned_try: 0.0,
            total_spent_try: 0.0,
        };
        s.recalculate_net_worth(world);
        s
    }

    pub fn recalculate_net_worth(&mut self, world: &WorldSnapshot) {
        let portfolio_try = self.portfolio.usd * world.market.usd_try
            + self.portfolio.gold_grams * world.market.gold_gram_try
            + self.portfolio.btc * world.market.btc_try;
        self.net_worth_try = ((self.cash_try + portfolio_try) * 100.0).round() / 100.0;
    }

    pub fn inject_event(&mut self, title: String, cash_delta: f64, mood_delta: f64, world: &WorldSnapshot) {
        self.cash_try = (self.cash_try + cash_delta).round();
        if cash_delta > 0.0 {
            self.total_earned_try += cash_delta;
        } else {
            self.total_spent_try += -cash_delta;
        }
        self.vitals.mood = (self.vitals.mood + mood_delta).clamp(0.0, 100.0);
        let msg = format!(
            "Operatör Olayı: {} ({:+.0} ₺, Moral {:+.0})",
            title, cash_delta, mood_delta
        );
        self.pending_events.push(msg.clone());
        self.milestones.insert(0, format!("Gün {}: {}", self.day, msg));
        if self.milestones.len() > 40 {
            self.milestones.truncate(40);
        }
        self.recalculate_net_worth(world);
    }

    pub fn apply_decision(
        &mut self,
        decision: AgentDecision,
        world: &WorldSnapshot,
        provider_used: String,
    ) {
        if !self.alive {
            return;
        }

        self.day += 1;
        self.days_in_housing += 1;
        self.age = 18 + (self.day / 365);
        self.agent_name = format!("Deniz ({})", self.age);

        let start_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let current_date = start_date + ChronoDuration::days(self.day as i64);
        self.sim_date = current_date.format("%Y-%m-%d").to_string();

        let starting_cash = self.cash_try;
        let housings = housing_catalog(world.market.usd_try);
        let jobs = job_catalog(world.market.usd_try);

        let mut outcome_parts: Vec<String> = Vec::new();
        let fx_scale = (world.market.usd_try / 36.0).clamp(0.85, 2.5);

        // Daily baseline living cost (food + Istanbul transit)
        let daily_living_cost = (240.0 * fx_scale).round();
        self.cash_try -= daily_living_cost;
        self.total_spent_try += daily_living_cost;
        self.vitals.satiety = (self.vitals.satiety - 14.0).clamp(0.0, 100.0);

        // Weather effect on mood/energy
        if world.weather.condition.contains("Yağmur") || world.weather.condition.contains("Fırtına") {
            self.vitals.mood = (self.vitals.mood - 1.5).clamp(0.0, 100.0);
        } else if world.weather.condition == "Açık" {
            self.vitals.mood = (self.vitals.mood + 1.5).clamp(0.0, 100.0);
        }

        // Housing daily cost & regen + eviction check for couch
        if let Some(h) = housings.iter().find(|h| h.id == self.housing_id) {
            if h.daily_cost_try > 0.0 {
                self.cash_try -= h.daily_cost_try;
                self.total_spent_try += h.daily_cost_try;
            }
            self.vitals.energy = (self.vitals.energy + h.energy_regen).clamp(0.0, 100.0);
            self.vitals.mood = (self.vitals.mood + h.mood_bonus).clamp(0.0, 100.0);

            if let Some(max_d) = h.max_days {
                if self.days_in_housing > max_d {
                    // Forced move to hostel if couch time is up
                    if let Some(hostel) = housings.iter().find(|x| x.id == "hostel") {
                        self.housing_id = hostel.id.clone();
                        self.housing_name = hostel.name.clone();
                        self.days_in_housing = 1;
                        self.vitals.mood = (self.vitals.mood - 8.0).clamp(0.0, 100.0);
                        outcome_parts.push(
                            "Arkadaş koltuğunda 7 günlük süre doldu! Zorunlu olarak pansiyona geçildi."
                                .to_string(),
                        );
                        self.add_milestone("Arkadaş koltuğundan çıkıp pansiyona geçti.");
                    }
                }
            }

            // Monthly rent on every 30th day
            if h.monthly_rent_try > 0.0 && self.day % 30 == 0 {
                self.cash_try -= h.monthly_rent_try;
                self.total_spent_try += h.monthly_rent_try;
                outcome_parts.push(format!(
                    "Aylık kira ödendi (-{:.0} ₺).",
                    h.monthly_rent_try
                ));
            }
        }

        // Execute agent's chosen action
        let action_norm = decision.action.trim().to_lowercase();
        let action_label = match action_norm.as_str() {
            "work" => {
                if let Some(job) = jobs.iter().find(|j| j.id == self.job_id) {
                    if job.id == "unemployed" {
                        outcome_parts.push("Mevcut bir işi olmadığı için çalışamadı; iş başvurusu yapması gerekiyor.".to_string());
                    } else if self.vitals.energy < 15.0 {
                        self.vitals.health = (self.vitals.health - 8.0).clamp(0.0, 100.0);
                        outcome_parts.push("Aşırı yorgunluk nedeniyle verimsiz mesai yaptı; sağlığı düştü.".to_string());
                        let half_pay = (job.daily_pay_try * 0.5).round();
                        self.cash_try += half_pay;
                        self.total_earned_try += half_pay;
                    } else {
                        let pay = job.daily_pay_try;
                        self.cash_try += pay;
                        self.total_earned_try += pay;
                        self.vitals.energy = (self.vitals.energy - job.energy_cost).clamp(0.0, 100.0);
                        self.vitals.satiety = (self.vitals.satiety - 8.0).clamp(0.0, 100.0);
                        // Small on-the-job skill growth
                        match job.id.as_str() {
                            "barista_courier" => self.skills.trade = (self.skills.trade + 0.8).min(100.0),
                            "sales_support" => self.skills.communication = (self.skills.communication + 1.0).min(100.0),
                            "freelance_digital" => {
                                self.skills.english = (self.skills.english + 0.7).min(100.0);
                                self.skills.communication = (self.skills.communication + 0.6).min(100.0);
                            }
                            "junior_dev" => {
                                self.skills.coding = (self.skills.coding + 1.2).min(100.0);
                                self.skills.english = (self.skills.english + 0.5).min(100.0);
                            }
                            "micro_saas" => {
                                self.skills.coding = (self.skills.coding + 1.0).min(100.0);
                                self.skills.finance = (self.skills.finance + 1.2).min(100.0);
                            }
                            _ => {}
                        }
                        outcome_parts.push(format!(
                            "{} olarak çalıştı (+{:.0} ₺).",
                            job.title, pay
                        ));
                    }
                }
                "ÇALIŞMA".to_string()
            }
            "apply_job" => {
                let target_id = decision.target.clone().unwrap_or_else(|| "barista_courier".to_string());
                if let Some(job) = jobs.iter().find(|j| j.id == target_id) {
                    let meets = self.skills.coding >= job.req_coding
                        && self.skills.communication >= job.req_communication
                        && self.skills.trade >= job.req_trade
                        && self.skills.finance >= job.req_finance
                        && self.skills.english >= job.req_english
                        && self.cash_try >= job.req_capital_try;
                    if meets {
                        if job.req_capital_try > 0.0 && self.job_id != job.id {
                            self.cash_try -= job.req_capital_try;
                            self.total_spent_try += job.req_capital_try;
                        }
                        self.job_id = job.id.clone();
                        self.job_title = job.title.clone();
                        self.vitals.mood = (self.vitals.mood + 10.0).clamp(0.0, 100.0);
                        self.vitals.energy = (self.vitals.energy - 10.0).clamp(0.0, 100.0);
                        outcome_parts.push(format!("Yeni işe kabul edildi: {}!", job.title));
                        self.add_milestone(&format!("Yeni iş: {}", job.title));
                    } else {
                        self.vitals.mood = (self.vitals.mood - 4.0).clamp(0.0, 100.0);
                        outcome_parts.push(format!(
                            "{} başvurusu yetenek/sermaye yetersizliği nedeniyle reddedildi.",
                            job.title
                        ));
                    }
                } else {
                    outcome_parts.push("Geçersiz iş hedefi seçildi.".to_string());
                }
                "İŞ BAŞVURUSU".to_string()
            }
            "study" => {
                let skill = decision.target.clone().unwrap_or_else(|| "coding".to_string()).to_lowercase();
                let gain = if self.vitals.energy >= 40.0 { 3.2 } else { 1.6 };
                self.vitals.energy = (self.vitals.energy - 18.0).clamp(0.0, 100.0);
                let label = match skill.as_str() {
                    "coding" | "yazilim" => {
                        self.skills.coding = (self.skills.coding + gain).min(100.0);
                        "Yazılım"
                    }
                    "communication" | "iletisim" => {
                        self.skills.communication = (self.skills.communication + gain).min(100.0);
                        "İletişim"
                    }
                    "trade" | "zanaat" => {
                        self.skills.trade = (self.skills.trade + gain).min(100.0);
                        "Zanaat/Hizmet"
                    }
                    "finance" | "finans" => {
                        self.skills.finance = (self.skills.finance + gain).min(100.0);
                        "Finansal Okuryazarlık"
                    }
                    "english" | "ingilizce" => {
                        self.skills.english = (self.skills.english + gain).min(100.0);
                        "İngilizce"
                    }
                    _ => {
                        self.skills.coding = (self.skills.coding + gain).min(100.0);
                        "Yazılım"
                    }
                };
                self.vitals.mood = (self.vitals.mood + 2.0).clamp(0.0, 100.0);
                outcome_parts.push(format!("{} çalıştı (+{:.1} puan).", label, gain));
                "EĞİTİM / GELİŞİM".to_string()
            }
            "move_housing" => {
                let target_id = decision.target.clone().unwrap_or_else(|| "hostel".to_string());
                if let Some(h) = housings.iter().find(|x| x.id == target_id) {
                    let upfront = if h.monthly_rent_try > 0.0 {
                        h.monthly_rent_try
                    } else {
                        h.daily_cost_try * 2.0
                    };
                    if self.cash_try >= upfront {
                        self.cash_try -= upfront;
                        self.total_spent_try += upfront;
                        self.housing_id = h.id.clone();
                        self.housing_name = h.name.clone();
                        self.days_in_housing = 1;
                        self.vitals.mood = (self.vitals.mood + 8.0).clamp(0.0, 100.0);
                        outcome_parts.push(format!(
                            "Yeni yere taşındı: {} (peşinat/ilk ödeme -{:.0} ₺).",
                            h.name, upfront
                        ));
                        self.add_milestone(&format!("Taşındı: {}", h.name));
                    } else {
                        outcome_parts.push(format!(
                            "{} için nakit yetersiz (gereken {:.0} ₺).",
                            h.name, upfront
                        ));
                    }
                } else {
                    outcome_parts.push("Geçersiz barınma hedefi.".to_string());
                }
                "BARINMA DEĞİŞİMİ".to_string()
            }
            "rest" => {
                self.vitals.energy = (self.vitals.energy + 32.0).clamp(0.0, 100.0);
                self.vitals.health = (self.vitals.health + 8.0).clamp(0.0, 100.0);
                self.vitals.mood = (self.vitals.mood + 6.0).clamp(0.0, 100.0);
                outcome_parts.push("Gün boyu dinlendi; enerji ve sağlık toparlandı.".to_string());
                "DİNLENME".to_string()
            }
            "eat_well" => {
                let meal_cost = (380.0 * fx_scale).round();
                if self.cash_try >= meal_cost {
                    self.cash_try -= meal_cost;
                    self.total_spent_try += meal_cost;
                    self.vitals.satiety = (self.vitals.satiety + 42.0).clamp(0.0, 100.0);
                    self.vitals.health = (self.vitals.health + 7.0).clamp(0.0, 100.0);
                    self.vitals.energy = (self.vitals.energy + 12.0).clamp(0.0, 100.0);
                    outcome_parts.push(format!("Dengeli beslendi (-{:.0} ₺).", meal_cost));
                } else {
                    self.vitals.satiety = (self.vitals.satiety + 15.0).clamp(0.0, 100.0);
                    outcome_parts.push("Bütçe kısıtlı olduğu için ekonomik öğün yedi.".to_string());
                }
                "BESLENME".to_string()
            }
            "socialize" => {
                let cost = (250.0 * fx_scale).round();
                if self.cash_try >= cost {
                    self.cash_try -= cost;
                    self.total_spent_try += cost;
                }
                self.vitals.mood = (self.vitals.mood + 14.0).clamp(0.0, 100.0);
                self.skills.communication = (self.skills.communication + 1.5).min(100.0);
                self.vitals.energy = (self.vitals.energy - 8.0).clamp(0.0, 100.0);
                outcome_parts.push("Yeni insanlarla tanıştı ve çevre edindi (+1.5 İletişim, +14 Moral).".to_string());
                "NETWORK / SOSYAL".to_string()
            }
            "invest" => {
                let asset = decision.target.clone().unwrap_or_else(|| "usd".to_string()).to_lowercase();
                let amount = decision.amount_try.unwrap_or(1_500.0).clamp(200.0, self.cash_try.max(0.0));
                if amount >= 200.0 && self.cash_try >= amount {
                    self.cash_try -= amount;
                    match asset.as_str() {
                        "usd" => {
                            let qty = amount / world.market.usd_try.max(1.0);
                            self.portfolio.usd += qty;
                            outcome_parts.push(format!(
                                "Gerçek kurdan {:.2} USD aldı (-{:.0} ₺, kur: {:.2}).",
                                qty, amount, world.market.usd_try
                            ));
                        }
                        "gold" | "altin" => {
                            let qty = amount / world.market.gold_gram_try.max(1.0);
                            self.portfolio.gold_grams += qty;
                            outcome_parts.push(format!(
                                "Gerçek fiyattan {:.2} gr Altın aldı (-{:.0} ₺).",
                                qty, amount
                            ));
                        }
                        "btc" | "bitcoin" => {
                            let qty = amount / world.market.btc_try.max(1.0);
                            self.portfolio.btc += qty;
                            outcome_parts.push(format!(
                                "Gerçek fiyattan {:.5} BTC aldı (-{:.0} ₺).",
                                qty, amount
                            ));
                        }
                        _ => {
                            let qty = amount / world.market.usd_try.max(1.0);
                            self.portfolio.usd += qty;
                            outcome_parts.push(format!("{:.2} USD aldı (-{:.0} ₺).", qty, amount));
                        }
                    }
                    self.skills.finance = (self.skills.finance + 0.8).min(100.0);
                    if self.milestones.iter().all(|m| !m.contains("ilk yatırım")) {
                        self.add_milestone("Gerçek piyasa verisiyle ilk yatırımını yaptı.");
                    }
                } else {
                    outcome_parts.push("Yatırım yapmak için yeterli nakit yok.".to_string());
                }
                "YATIRIM (SANDBOX)".to_string()
            }
            "sell_asset" => {
                let asset = decision.target.clone().unwrap_or_else(|| "usd".to_string()).to_lowercase();
                let mut gained = 0.0;
                match asset.as_str() {
                    "usd" if self.portfolio.usd > 0.0 => {
                        gained = (self.portfolio.usd * world.market.usd_try).round();
                        self.portfolio.usd = 0.0;
                    }
                    "gold" | "altin" if self.portfolio.gold_grams > 0.0 => {
                        gained = (self.portfolio.gold_grams * world.market.gold_gram_try).round();
                        self.portfolio.gold_grams = 0.0;
                    }
                    "btc" | "bitcoin" if self.portfolio.btc > 0.0 => {
                        gained = (self.portfolio.btc * world.market.btc_try).round();
                        self.portfolio.btc = 0.0;
                    }
                    _ => {}
                }
                if gained > 0.0 {
                    self.cash_try += gained;
                    outcome_parts.push(format!("Portföyden {} bozduruldu (+{:.0} ₺).", asset.to_uppercase(), gained));
                } else {
                    outcome_parts.push("Bozdurulacak varlık bulunamadı.".to_string());
                }
                "VARLIK SATIŞI".to_string()
            }
            _ => {
                self.vitals.energy = (self.vitals.energy + 10.0).clamp(0.0, 100.0);
                outcome_parts.push("Günü planlama ve gözlemle geçirdi.".to_string());
                "GÖZLEM".to_string()
            }
        };

        // Health & debt consequences
        if self.vitals.satiety < 20.0 {
            self.vitals.health = (self.vitals.health - 7.0).clamp(0.0, 100.0);
            self.vitals.mood = (self.vitals.mood - 5.0).clamp(0.0, 100.0);
            outcome_parts.push("Yetersiz beslenme sağlığını olumsuz etkiliyor.".to_string());
        }
        if self.cash_try < -10_000.0 {
            self.vitals.mood = (self.vitals.mood - 10.0).clamp(0.0, 100.0);
            outcome_parts.push("Ciddi borç baskısı altında!".to_string());
        }
        if self.vitals.health <= 0.0 {
            self.alive = false;
            self.status_banner = format!(
                "Gün {} ({} yaş): Sağlık tükendi, simülasyon durdu.",
                self.day, self.age
            );
            self.add_milestone("Sağlık tükendiği için simülasyon sonlandı.");
        } else {
            self.status_banner = format!(
                "Gün {} · {} · {} · Net Değer: {:.0} ₺",
                self.day, self.housing_name, self.job_title, self.net_worth_try
            );
        }

        // Clear consumed operator events
        self.pending_events.clear();

        self.recalculate_net_worth(world);
        let cash_delta = ((self.cash_try - starting_cash) * 100.0).round() / 100.0;

        let entry = JournalEntry {
            day: self.day,
            sim_date: self.sim_date.clone(),
            age: self.age,
            thought: decision.thought,
            action: action_norm,
            action_label,
            outcome: outcome_parts.join(" "),
            plan: decision.plan,
            mood_tag: if decision.mood_tag.is_empty() {
                "odaklı".to_string()
            } else {
                decision.mood_tag
            },
            cash_delta_try: cash_delta,
            net_worth_try: self.net_worth_try,
            provider_used,
        };

        self.journal.insert(0, entry);
        if self.journal.len() > 80 {
            self.journal.truncate(80);
        }

        self.history.push(NetWorthPoint {
            day: self.day,
            cash_try: self.cash_try.round(),
            net_worth_try: self.net_worth_try.round(),
        });
        if self.history.len() > 120 {
            self.history.remove(0);
        }
    }

    fn add_milestone(&mut self, text: &str) {
        self.milestones.insert(0, format!("Gün {}: {}", self.day, text));
        if self.milestones.len() > 40 {
            self.milestones.truncate(40);
        }
    }
}
