# zero-to-life

Gerçek dünya verilerine **salt-okunur (read-only)** bağlı, sandbox tabanlı otonom yapay zeka yaşam simülasyonu (Rust + Axum + WebSocket).

18 yaşında, sıfırdan İstanbul'da hayata başlayan bir AI ajanının barınma, çalışma, yetenek geliştirme, yatırım ve hayatta kalma kararlarını canlı bir web arayüzünden takip edebilirsiniz.

## Özellikler
- **Salt-Okunur Gerçek Dünya Köprüsü:** İstanbul hava durumu (Open-Meteo), döviz kurları (USD/EUR/GBP), BTC & Altın fiyatları ve Türkçe haber başlıkları (RSS).
- **Çoklu AI Beyni:** API anahtarsız Demo/Heuristik mod, yerel **Ollama**, **OpenAI / OpenRouter / Groq** uyumlu API ve **Google Gemini API**.
- **Canlı Web Dashboard:** Düşünce akışı, vitaller, yetenekler, net değer grafiği, hız/duraklatma kontrolü ve canlı olay enjeksiyonu.

## Çalıştırma
```bash
cargo run
```
Ardından tarayıcıda `http://127.0.0.1:3030` adresini açın.
