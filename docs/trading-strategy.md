# Trading Strategy Explained

This document explains **how the bot’s arbitrage strategy works** in plain language, so you can use and tune the bot with confidence.

---

## 1. What Markets the Bot Uses

The bot uses **two prediction markets** on Polymarket:

1. **SOL 15-minute market**  
   - Asks: “Will SOL go **Up** or **Down** in the next 15 minutes?”  
   - Two tokens: **SOL Up** and **SOL Down**.

2. **BTC 15-minute market**  
   - Asks: “Will BTC go **Up** or **Down** in the next 15 minutes?”  
   - Two tokens: **BTC Up** and **BTC Down**.

Each market **resolves after 15 minutes**. Exactly one of Up/Down wins per market and pays **$1** per share; the other pays **$0**.

---

## 2. Why “SOL Up + BTC Down” (or “SOL Down + BTC Up”)?

The bot does **not** bet on one market alone. It combines **one token from SOL** and **one token from BTC** so that **exactly one of the two tokens is guaranteed to win** when both markets resolve.

- **Strategy A:** Buy **SOL Up** + **BTC Down**  
  - If SOL goes up and BTC goes down → both tokens win → you get $1 + $1 = **$2** per pair.  
  - If SOL goes down and BTC goes up → both lose → **$0** per pair.  
  - Other cases (SOL up + BTC up, or SOL down + BTC down) → one wins, one loses → **$1** per pair.

- **Strategy B:** Buy **SOL Down** + **BTC Up**  
  - Same idea, with the opposite pair.  
  - One of the two tokens will always match the outcome of its market; the bot is effectively betting that **SOL and BTC move in opposite directions** (one up, one down).

So the bot is **not** guessing “SOL up” or “BTC down” in isolation. It is betting that **SOL and BTC move in opposite directions** and locking that in by buying the right pair (SOL Up + BTC Down or SOL Down + BTC Up).

---

## 3. Where the Arbitrage Comes From

In theory, for a binary outcome, **Up + Down** for the same market should cost **$1** (one of them always wins). In practice, order books are not perfect:

- **SOL Up** might be **$0.47** and **BTC Down** **$0.40**.  
- **Total cost = $0.87** for one “pair” (1 SOL Up + 1 BTC Down).  
- When markets resolve, **at least one** of the two tokens will win, so you get **at least $1** back per pair.  
- So you lock in **at least $1 − $0.87 = $0.13** profit per pair (before fees), as long as you hold to resolution.

The **arbitrage** is: buy the pair when **total cost < $1** and **expected profit ≥ your minimum threshold**.

The bot does the same idea for the other pair: **SOL Down + BTC Up**, whenever that pair’s total cost is below $1 by enough.

---

## 4. How the Bot Picks Trades

The bot continuously:

1. **Fetches** the best **ask (buy)** prices for:
   - SOL Up, SOL Down  
   - BTC Up, BTC Down  

2. **Checks two combinations:**
   - **SOL Up + BTC Down** (total cost = SOL Up ask + BTC Down ask)  
   - **SOL Down + BTC Up** (total cost = SOL Down ask + BTC Up ask)  

3. **Rules to open a trade:**
   - Total cost **< $1.00**.  
   - Expected profit **(1 − total cost)** **≥** your **`min_profit_threshold`** (e.g. $0.01 or $0.05).  
   - A **safety filter** in the code skips pairs where both tokens are very cheap (e.g. both &lt; $0.60) to avoid bad or illiquid situations.

4. **Position size**  
   - The bot caps the **total dollar amount** per trade at **`max_position_size`** (e.g. $50 or $100).  
   - It buys as many “units” (pairs) as that allows at the current cost per unit.

So: **arbitrage = buy the complementary pair (SOL vs BTC) when the sum of their prices is less than $1 by at least your minimum profit.**

---

## 5. Example (Simple Numbers)

| Token    | Ask price |
|----------|-----------|
| SOL Up   | $0.48     |
| BTC Down | $0.45     |
| **Total**| **$0.93** |

- Cost per pair: **$0.93**.  
- If you have `min_profit_threshold = 0.05`, then **$1.00 − $0.93 = $0.07 ≥ $0.05** → trade is allowed.  
- With `max_position_size = 100`, you could buy about **100 / 0.93 ≈ 107** pairs (the bot uses your config to compute the exact size).  
- When both markets resolve, **at least one** of SOL Up or BTC Down wins, so you get **at least $1 per pair** back; best case both win and you get **$2 per pair**.

So you’re **locking in a minimum payoff** by buying the pair cheap.

---

## 6. Risk in Plain Terms

- **Best case:** SOL and BTC move in opposite directions → **both** your tokens win → **$2** per pair (minus what you paid).  
- **Worst case:** They move the same way (both up or both down) → **both** your tokens lose → **$0** per pair; you lose the cost of the pair.  
- **Middle case:** One up, one down → **one** token wins → **$1** per pair; you may still profit or lose a bit depending on entry cost.

So the “arbitrage” is only **risk-free** in the sense that you’re buying **below $1** for a pair that pays **at least $1** in one of the outcomes. The remaining risk is **both tokens losing** (SOL and BTC move together). The bot’s safety filter (e.g. skipping when both legs are very cheap) helps avoid the worst mispricings.

---

## 7. How This Fits Your Config

- **`min_profit_threshold`**  
  Higher = fewer trades, but each trade needs a larger edge (e.g. $0.05 per pair). Lower = more trades, smaller edge per trade.

- **`max_position_size`**  
  Maximum total dollars per trade (both legs combined). Limits how much you risk on a single opportunity.

- **Simulation first**  
  Use **`--simulation`** to see how often the bot finds opportunities and what it would trade, without risking real money.

---

## 8. Summary

| Concept | Meaning |
|--------|---------|
| **Markets** | SOL 15m and BTC 15m Up/Down prediction markets on Polymarket. |
| **Pair** | One SOL token (Up or Down) + one BTC token (Down or Up) so that one leg is “up” and the other “down”. |
| **Arbitrage** | Buy that pair when **price(SOL token) + price(BTC token) < $1** and profit ≥ `min_profit_threshold`. |
| **Your role** | Set `min_profit_threshold`, `max_position_size`, and run in simulation before going live. |

For setup and run instructions, see [Getting Started](getting-started.md). For all options, see [Configuration](configuration.md).

**Author:** [@gabagool222](https://t.me/gabagool222)  
**Repo:** [github.com/gabagool222/15m-btc-sol-polymarket-arbitrage-bot](https://github.com/gabagool222/15m-btc-sol-polymarket-arbitrage-bot)
