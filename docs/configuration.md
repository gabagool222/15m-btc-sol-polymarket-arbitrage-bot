# Configuration Reference

This document describes every configuration option for the bot.

---

## Config File Location

- **Default:** `config.json` in the directory from which you run the bot.
- **Override:** Use `--config <path>` on the command line, e.g.  
  `cargo run --release -- --simulation --config my-config.json`

On first run, if the file does not exist, the bot creates a default `config.json`.

---

## Full Config Structure

```json
{
  "polymarket": {
    "gamma_api_url": "https://gamma-api.polymarket.com",
    "clob_api_url": "https://clob.polymarket.com",
    "ws_url": "wss://clob-ws.polymarket.com",
    "api_key": null
  },
  "trading": {
    "min_profit_threshold": 0.01,
    "max_position_size": 100.0,
    "sol_condition_id": null,
    "btc_condition_id": null,
    "check_interval_ms": 1000
  }
}
```

---

## Section: `polymarket`

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `gamma_api_url` | string | `https://gamma-api.polymarket.com` | Polymarket Gamma API base URL. Change only if you use a custom/proxy endpoint. |
| `clob_api_url` | string | `https://clob.polymarket.com` | Polymarket CLOB (order book / orders) API base URL. |
| `ws_url` | string | `wss://clob-ws.polymarket.com` | WebSocket URL (reserved for future use). |
| `api_key` | string or null | `null` | Your Polymarket API key. **Required for production.** Leave `null` for simulation. |

---

## Section: `trading`

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `min_profit_threshold` | number | `0.01` | Minimum profit in **dollars per unit** (one SOL token + one BTC token) to trigger a trade. Example: `0.05` = only trade when expected profit ≥ $0.05 per unit. |
| `max_position_size` | number | `100.0` | Maximum **total dollar amount** per trade (both legs combined). The bot will not invest more than this per opportunity. |
| `sol_condition_id` | string or null | `null` | Force a specific SOL 15-minute market by condition ID. Usually leave `null` so the bot auto-discovers the current market. |
| `btc_condition_id` | string or null | `null` | Force a specific BTC 15-minute market by condition ID. Usually leave `null`. |
| `check_interval_ms` | number | `1000` | How often the bot fetches prices and checks for opportunities, in **milliseconds**. Example: `2000` = every 2 seconds. |

---

## Recommended Values for Beginners

- **Simulation:**  
  `min_profit_threshold: 0.01`–`0.05`, `max_position_size: 50`–`100`, `api_key: null`, `check_interval_ms: 1000`.

- **Production (conservative):**  
  `min_profit_threshold: 0.02`–`0.05`, `max_position_size` equal to what you’re willing to risk per trade, valid `api_key`.

- **Production (aggressive):**  
  Lower `min_profit_threshold`, higher `max_position_size`. Only after you understand the strategy and have tested in simulation.

---

## Command-Line Overrides

The bot does **not** override config file values from the command line except for:

- **`--simulation`** / **`--no-simulation`** — run mode (simulation vs production).
- **`--config <path>`** — path to the config file.

All other settings (thresholds, sizes, API URLs, condition IDs) come from the config file.

---

## Environment and Security

- **API key:** Store only in `config.json` and keep the file out of version control (add `config.json` to `.gitignore` if it contains secrets). Do not share or commit your key.
- **File permissions:** On Linux/macOS, restrict access if the file contains your API key:  
  `chmod 600 config.json`

---

For how to use the bot and what the strategy does, see [Getting Started](getting-started.md) and [Trading Strategy](trading-strategy.md).

**Author:** [@gabagool222](https://t.me/gabagool222)  
**Repo:** [github.com/gabagool222/15m-btc-sol-polymarket-arbitrage-bot](https://github.com/gabagool222/15m-btc-sol-polymarket-arbitrage-bot)
