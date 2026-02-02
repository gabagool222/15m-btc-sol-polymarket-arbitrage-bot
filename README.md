# gabagool Polymarket 15m BTC/SOL Arbitrage Bot

A Rust-based arbitrage bot for [Polymarket](https://polymarket.com) that monitors **SOL** and **BTC** 15-minute price prediction markets and executes trades when arbitrage opportunities are detected.

**Author:** [@gabagool222](https://t.me/gabagool222)  
**Repository:** [github.com/gabagool222/15m-btc-sol-polymarket-arbitrage-bot](https://github.com/gabagool222/15m-btc-sol-polymarket-arbitrage-bot)

---

## Documentation

| Document | Description |
|----------|-------------|
| **[Getting Started](docs/getting-started.md)** | Install, configure, and run the bot (beginner-friendly). |
| **[Trading Strategy](docs/trading-strategy.md)** | How the arbitrage strategy works in plain language. |
| **[Configuration](docs/configuration.md)** | All config options and reference. |

---

## Quick Start

1. **Install Rust** ([rustup.rs](https://rustup.rs)).
2. **Clone and build:**
   ```bash
   git clone https://github.com/gabagool222/15m-btc-sol-polymarket-arbitrage-bot.git
   cd 15m-btc-sol-polymarket-arbitrage-bot
   cargo build --release
   ```
3. **Run in simulation** (no real trades):
   ```bash
   cargo run --release -- --simulation
   ```
4. **Configure** `config.json` (created on first run). See [Configuration](docs/configuration.md).
5. **Production** (real trades; requires Polymarket API key):
   ```bash
   cargo run --release -- --no-simulation
   ```

---

## How It Works

The bot continuously monitors two Polymarket markets:

- **SOL 15-minute price change** (Up / Down)
- **BTC 15-minute price change** (Up / Down)

### Arbitrage Idea

It looks for cases where the **sum of two complementary tokens** (one from each market) is **less than $1.00**.

**Example:**

- SOL Up: $0.47  
- BTC Down: $0.40  
- **Total: $0.87**  
- When the markets resolve, one of the two tokens pays $1.00, so you lock in at least **$0.13** profit per pair (before fees).

When such an opportunity is found, the bot:

1. Buys the Up token in the SOL market  
2. Buys the Down token in the BTC market (or the opposite pair: SOL Down + BTC Up)  
3. Holds until resolution to realize profit  

For a full, beginner-friendly explanation, see [Trading Strategy](docs/trading-strategy.md).

---

## Project Structure

```
src/
├── main.rs           # Entry point and orchestration
├── config.rs         # CLI args and config loading
├── models.rs         # Data types (markets, orders, etc.)
├── api/              # Polymarket API client
│   ├── mod.rs
│   └── client.rs
├── strategy/         # Arbitrage detection
│   ├── mod.rs
│   └── detector.rs
├── monitor/          # Market data and 15m period handling
│   ├── mod.rs
│   └── market.rs
└── trading/          # Simulation and live execution
    ├── mod.rs
    └── executor.rs
```

- **API** — Gamma and CLOB API calls  
- **Strategy** — Detects when SOL+BTC token pairs cost &lt; $1 and meet your threshold  
- **Monitor** — Fetches SOL/BTC 15m markets and prices, handles new 15m periods  
- **Trading** — Executes (or simulates) buys and tracks pending trades  

---

## Configuration Summary

| Option | Default | Description |
|--------|---------|-------------|
| `min_profit_threshold` | `0.01` | Min profit ($) per unit to trade |
| `max_position_size` | `100.0` | Max $ per trade (both legs) |
| `check_interval_ms` | `1000` | How often to check markets (ms) |
| `api_key` | `null` | Polymarket API key (required for production) |

Full reference: [Configuration](docs/configuration.md).

---

## Command-Line Options

| Option | Default | Description |
|--------|---------|-------------|
| `--simulation` | `true` | Run in simulation (no real orders) |
| `--no-simulation` | — | Run in production |
| `--config <path>` | `config.json` | Config file path |

---

## Notes

- The bot runs until you stop it (**Ctrl+C**).
- In simulation mode, trades are logged but **not** sent to Polymarket.
- The bot discovers the current 15-minute SOL/BTC markets automatically unless you set `sol_condition_id` and `btc_condition_id` in config.
- For production, ensure sufficient balance and API permissions on Polymarket.

---

## Support

- **Telegram:** [@gabagool222](https://t.me/gabagool222)  
- **GitHub:** [github.com/gabagool222/15m-btc-sol-polymarket-arbitrage-bot](https://github.com/gabagool222/15m-btc-sol-polymarket-arbitrage-bot)  
- **Docs:** [docs/](docs/README.md)
