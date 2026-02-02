# Getting Started

This guide walks you through installing, configuring, and running the Polymarket 15-minute BTC/SOL arbitrage bot from scratch.

---

## What You Need

- **Rust** (1.70+). Install from [rustup.rs](https://rustup.rs).
- **Polymarket account** and (for live trading) an **API key** from [Polymarket](https://polymarket.com).
- **Basic terminal** (PowerShell, Command Prompt, or a Unix shell).

---

## 1. Install Rust

If Rust is not installed:

**Windows (PowerShell):**
```powershell
winget install Rustlang.Rustup
```
Then restart your terminal.

**macOS / Linux:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Check the install:
```bash
cargo --version
rustc --version
```

---

## 2. Clone and Build the Bot

Clone the repo and build in release mode:

```bash
git clone https://github.com/gabagool222/15m-btc-sol-polymarket-arbitrage-bot.git
cd 15m-btc-sol-polymarket-arbitrage-bot
cargo build --release
```

The binary will be at `target/release/gabagool-polymarket-arb-bot.exe` (Windows) or `target/release/gabagool-polymarket-arb-bot` (macOS/Linux).

---

## 3. Run the Bot (Simulation First)

**Always start in simulation mode** so no real money is used:

```bash
cargo run --release -- --simulation
```

Or run the built binary:

```bash
# Windows
.\target\release\gabagool-polymarket-arb-bot.exe --simulation

# macOS / Linux
./target/release/gabagool-polymarket-arb-bot --simulation
```

On first run, the bot creates a `config.json` in the current directory. You’ll see logs about:

- Discovering SOL and BTC 15-minute markets  
- Fetching prices  
- Detecting (simulated) arbitrage opportunities  

Stop the bot with **Ctrl+C**.

---

## 4. Configure the Bot

Edit `config.json` in the project root. Important sections:

### Polymarket API

- **`api_key`**  
  Leave `null` for simulation. For live trading, set your Polymarket API key (string).

### Trading

- **`min_profit_threshold`** (default: `0.01`)  
  Minimum profit in dollars per “unit” (one SOL token + one BTC token) to trigger a trade.  
  Example: `0.05` = only trade if expected profit ≥ $0.05 per unit.

- **`max_position_size`** (default: `100.0`)  
  Maximum total dollar amount per trade (sum of both legs).  
  Example: `50` = never invest more than $50 per opportunity.

- **`check_interval_ms`** (default: `1000`)  
  How often the bot checks markets, in milliseconds.  
  Example: `2000` = check every 2 seconds.

- **`sol_condition_id`** / **`btc_condition_id`**  
  Usually leave `null`. The bot discovers 15-minute markets automatically. Set these only if you want to force specific markets.

Example `config.json`:

```json
{
  "polymarket": {
    "gamma_api_url": "https://gamma-api.polymarket.com",
    "clob_api_url": "https://clob.polymarket.com",
    "ws_url": "wss://clob-ws.polymarket.com",
    "api_key": null
  },
  "trading": {
    "min_profit_threshold": 0.02,
    "max_position_size": 50.0,
    "sol_condition_id": null,
    "btc_condition_id": null,
    "check_interval_ms": 1000
  }
}
```

See [Configuration](configuration.md) for full options.

---

## 5. Run in Production (Real Trading)

Only after you understand the strategy and have tested in simulation:

1. Get a Polymarket API key and add it to `config.json` under `polymarket.api_key`.
2. Ensure you have enough balance and permissions on Polymarket.
3. Run without simulation:

```bash
cargo run --release -- --no-simulation
```

Or:

```bash
.\target\release\gabagool-polymarket-arb-bot.exe --no-simulation
```

The bot will place real orders when it finds opportunities that meet your `min_profit_threshold` and `max_position_size`.

---

## 6. Command-Line Options

| Option | Short | Default | Description |
|--------|--------|---------|-------------|
| `--simulation` | - | `true` | Run in simulation (no real orders). |
| `--no-simulation` | - | - | Run in production (real orders). |
| `--config <path>` | - | `config.json` | Path to config file. |

Examples:

```bash
# Custom config path
cargo run --release -- --simulation --config my-config.json

# Production with custom config
cargo run --release -- --no-simulation --config prod.json
```

---

## 7. What the Logs Mean

- **Starting Polymarket Arbitrage Bot** – Bot has started; next line shows SIMULATION or PRODUCTION.
- **SOL Market: … / BTC Market: …** – Which 15-minute markets are being used.
- **SIMULATION: Arbitrage opportunity detected!** – In simulation, a chance to arbitrage was found; no real order was placed.
- **PRODUCTION: Executing real arbitrage trade...** – Real orders are being placed.
- **Market Closed / Actual Profit** – A 15-minute market resolved; the bot reports result and profit/loss.

For how opportunities are chosen, see [Trading Strategy](trading-strategy.md).

---

## 8. Troubleshooting

| Issue | What to do |
|--------|------------|
| **“Could not find active SOL/BTC 15-minute market”** | Ensure Polymarket has active 15m SOL/BTC markets. Try again later or set `sol_condition_id` / `btc_condition_id` in config if you have specific IDs. |
| **“cargo” not found** | Install Rust via rustup and restart the terminal. |
| **Build errors** | Run `cargo clean` then `cargo build --release` again. |
| **API errors in production** | Check API key, permissions, and balance on Polymarket. |
| **No opportunities in simulation** | Normal. Opportunities depend on live order books. Run longer or try different times. |

---

## Next Steps

- Read [Trading Strategy](trading-strategy.md) to understand how the bot finds and executes arbitrage.
- Tune [Configuration](configuration.md) for your risk and capital.

For questions or issues, open a GitHub issue or contact [@gabagool222](https://t.me/gabagool222) on Telegram.
