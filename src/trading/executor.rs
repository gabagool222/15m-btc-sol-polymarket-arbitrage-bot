use crate::api::PolymarketApi;
use crate::config::TradingConfig;
use crate::models::*;
use anyhow::Result;
use log::{debug, info, warn};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Clone)]
struct CachedMarketData {
    market: MarketDetails,
    cached_at: Instant,
}

pub struct Trader {
    api: Arc<PolymarketApi>,
    config: TradingConfig,
    simulation_mode: bool,
    total_profit: Arc<Mutex<f64>>,
    trades_executed: Arc<Mutex<u64>>,
    pending_trades: Arc<Mutex<HashMap<String, PendingTrade>>>,
    market_cache: Arc<Mutex<HashMap<String, CachedMarketData>>>,
}

impl Trader {
    pub fn new(api: Arc<PolymarketApi>, config: TradingConfig, simulation_mode: bool) -> Self {
        Self {
            api,
            config,
            simulation_mode,
            total_profit: Arc::new(Mutex::new(0.0)),
            trades_executed: Arc::new(Mutex::new(0)),
            pending_trades: Arc::new(Mutex::new(HashMap::new())),
            market_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Check and settle pending trades when markets close
    pub async fn check_pending_trades(&self) -> Result<()> {
        let mut pending = self.pending_trades.lock().await;
        let mut to_remove = Vec::new();

        // Only check trades that are at least 14 minutes old (markets close after 15 minutes)
        let min_age = Duration::from_secs(14 * 60);

        let pending_count = pending.len();
        if pending_count > 0 {
            debug!("Checking {} pending trades for market closure...", pending_count);
        }

        for (key, trade) in pending.iter() {
            let age = trade.timestamp.elapsed();

            if age < min_age {
                debug!(
                    "Trade {} is too recent (age: {:.1}s, need: {:.1}s), skipping",
                    key,
                    age.as_secs_f64(),
                    min_age.as_secs_f64()
                );
                continue;
            }

            info!(
                "🔍 Checking market closure for trade {} (age: {:.1} minutes)",
                key,
                age.as_secs_f64() / 60.0
            );

            let (sol_closed, sol_winner) = self
                .check_market_result_cached(&trade.sol_condition_id, &trade.sol_token_id)
                .await?;
            let (btc_closed, btc_winner) = self
                .check_market_result_cached(&trade.btc_condition_id, &trade.btc_token_id)
                .await?;

            info!(
                "   SOL Market ({}): closed={}, winner={}",
                &trade.sol_condition_id[..16],
                sol_closed,
                sol_winner
            );
            info!(
                "   BTC Market ({}): closed={}, winner={}",
                &trade.btc_condition_id[..16],
                btc_closed,
                btc_winner
            );

            if sol_closed && btc_closed {
                if !self.simulation_mode {
                    self.sell_winning_tokens(&trade, sol_winner, btc_winner).await;
                }

                let actual_profit = self.calculate_actual_profit(&trade, sol_winner, btc_winner);

                let mut total = self.total_profit.lock().await;
                *total += actual_profit;
                let total_profit = *total;
                drop(total);

                info!(
                    "💰 Market Closed - SOL Winner: {}, BTC Winner: {} | Actual Profit: ${:.4} | Total Profit: ${:.2}",
                    if sol_winner { "WON" } else { "LOST" },
                    if btc_winner { "WON" } else { "LOST" },
                    actual_profit,
                    total_profit
                );

                to_remove.push(key.clone());
            } else {
                info!(
                    "   ⏳ Markets not both closed yet (SOL: {}, BTC: {}), will check again...",
                    sol_closed, btc_closed
                );
            }
        }

        for key in to_remove {
            pending.remove(&key);
        }

        Ok(())
    }

    async fn check_market_result_cached(
        &self,
        condition_id: &str,
        token_id: &str,
    ) -> Result<(bool, bool)> {
        let cache_ttl = Duration::from_secs(60);
        let mut cache = self.market_cache.lock().await;

        if let Some(cached) = cache.get(condition_id) {
            if cached.cached_at.elapsed() < cache_ttl {
                let market = &cached.market;
                if market.closed {
                    let winner = market
                        .tokens
                        .iter()
                        .find(|t| t.token_id == token_id)
                        .map(|t| t.winner)
                        .unwrap_or(false);
                    debug!("Using cached market data for condition_id: {}", condition_id);
                    return Ok((true, winner));
                } else {
                    debug!(
                        "Using cached market data (not closed yet) for condition_id: {}",
                        condition_id
                    );
                    return Ok((false, false));
                }
            }
        }

        drop(cache);
        match self.api.get_market(condition_id).await {
            Ok(market) => {
                let mut cache = self.market_cache.lock().await;
                cache.insert(
                    condition_id.to_string(),
                    CachedMarketData {
                        market: market.clone(),
                        cached_at: Instant::now(),
                    },
                );
                drop(cache);

                if market.closed {
                    let winner = market
                        .tokens
                        .iter()
                        .find(|t| t.token_id == token_id)
                        .map(|t| t.winner)
                        .unwrap_or(false);
                    Ok((true, winner))
                } else {
                    Ok((false, false))
                }
            }
            Err(e) => {
                warn!("Failed to fetch market {}: {}", condition_id, e);
                Ok((false, false))
            }
        }
    }

    async fn sell_winning_tokens(
        &self,
        trade: &PendingTrade,
        sol_winner: bool,
        btc_winner: bool,
    ) {
        let sell_price = "1.0";

        if sol_winner {
            let sell_order = OrderRequest {
                token_id: trade.sol_token_id.clone(),
                side: "SELL".to_string(),
                size: format!("{:.6}", trade.units),
                price: sell_price.to_string(),
                order_type: "LIMIT".to_string(),
            };

            match self.api.place_order(&sell_order).await {
                Ok(_) => {
                    info!("✅ Sold {} units of SOL Up token (winner) at $1.00", trade.units);
                }
                Err(e) => {
                    warn!("⚠️  Failed to sell SOL Up token: {}", e);
                }
            }
        }

        if btc_winner {
            let sell_order = OrderRequest {
                token_id: trade.btc_token_id.clone(),
                side: "SELL".to_string(),
                size: format!("{:.6}", trade.units),
                price: sell_price.to_string(),
                order_type: "LIMIT".to_string(),
            };

            match self.api.place_order(&sell_order).await {
                Ok(_) => {
                    info!("✅ Sold {} units of BTC Down token (winner) at $1.00", trade.units);
                }
                Err(e) => {
                    warn!("⚠️  Failed to sell BTC Down token: {}", e);
                }
            }
        }

        if !sol_winner && !btc_winner {
            warn!("⚠️  Both tokens lost - nothing to sell (both worth $0)");
        }
    }

    fn calculate_actual_profit(
        &self,
        trade: &PendingTrade,
        sol_winner: bool,
        btc_winner: bool,
    ) -> f64 {
        let payout_per_unit = if sol_winner && btc_winner {
            2.0
        } else if sol_winner || btc_winner {
            1.0
        } else {
            0.0
        };

        let total_payout = payout_per_unit * trade.units;
        let actual_profit = total_payout - trade.investment_amount;

        if actual_profit < 0.0 {
            warn!(
                "⚠️  LOSS: Both tokens lost! Lost ${:.4} on this trade",
                -actual_profit
            );
        }

        actual_profit
    }

    /// Execute arbitrage trade
    pub async fn execute_arbitrage(&self, opportunity: &ArbitrageOpportunity) -> Result<()> {
        if self.simulation_mode {
            self.simulate_trade(opportunity).await
        } else {
            self.execute_real_trade(opportunity).await
        }
    }

    async fn simulate_trade(&self, opportunity: &ArbitrageOpportunity) -> Result<()> {
        info!("🔍 SIMULATION: Arbitrage opportunity detected!");
        info!("   SOL Up Token Price: ${:.4}", opportunity.sol_up_price);
        info!("   BTC Down Token Price: ${:.4}", opportunity.btc_down_price);
        info!("   Total Cost: ${:.4}", opportunity.total_cost);
        info!(
            "   Expected Profit: ${:.4} ({:.2}%)",
            opportunity.expected_profit,
            (opportunity.expected_profit / opportunity.total_cost) * Decimal::from(100)
        );
        info!("   SOL Token ID: {}", opportunity.sol_up_token_id);
        info!("   BTC Token ID: {}", opportunity.btc_down_token_id);

        let position_size = self.calculate_position_size(opportunity);
        info!("   Position Size: ${:.2} (total investment amount)", position_size);

        let cost_per_unit = f64::try_from(opportunity.total_cost).unwrap_or(1.0);
        let units = position_size / cost_per_unit;
        info!(
            "   Units: {:.2} (each unit = ${:.4}, so ${:.2} / ${:.4} = {:.2} units)",
            units, cost_per_unit, position_size, cost_per_unit, units
        );
        info!(
            "   SOL Up amount: ${:.2} ({} units × ${:.4})",
            units * f64::try_from(opportunity.sol_up_price).unwrap_or(0.0),
            units,
            opportunity.sol_up_price
        );
        info!(
            "   BTC Down amount: ${:.2} ({} units × ${:.4})",
            units * f64::try_from(opportunity.btc_down_price).unwrap_or(0.0),
            units,
            opportunity.btc_down_price
        );

        let trade_key = format!(
            "{}_{}",
            opportunity.sol_condition_id,
            opportunity.btc_condition_id
        );

        let mut pending = self.pending_trades.lock().await;

        if let Some(existing_trade) = pending.get_mut(&trade_key) {
            existing_trade.units += units;
            existing_trade.investment_amount += position_size;
            info!(
                "   📊 Accumulated trade: Total units: {:.2}, Total investment: ${:.2}",
                existing_trade.units, existing_trade.investment_amount
            );
        } else {
            let pending_trade = PendingTrade {
                sol_token_id: opportunity.sol_up_token_id.clone(),
                btc_token_id: opportunity.btc_down_token_id.clone(),
                sol_condition_id: opportunity.sol_condition_id.clone(),
                btc_condition_id: opportunity.btc_condition_id.clone(),
                investment_amount: position_size,
                units,
                timestamp: std::time::Instant::now(),
            };
            pending.insert(trade_key, pending_trade);
        }
        drop(pending);

        let mut trades = self.trades_executed.lock().await;
        *trades += 1;
        let trades_count = *trades;
        drop(trades);

        info!(
            "   ✅ Simulated Trade Executed - Investment: ${:.2} | Expected Profit: ${:.4} | Trades: {}",
            position_size,
            f64::try_from(opportunity.expected_profit).unwrap_or(0.0) * units,
            trades_count
        );

        Ok(())
    }

    async fn execute_real_trade(&self, opportunity: &ArbitrageOpportunity) -> Result<()> {
        info!("🚀 PRODUCTION: Executing real arbitrage trade...");

        let position_size = self.calculate_position_size(opportunity);
        let size_str = format!("{:.6}", position_size);

        let sol_order = OrderRequest {
            token_id: opportunity.sol_up_token_id.clone(),
            side: "BUY".to_string(),
            size: size_str.clone(),
            price: opportunity.sol_up_price.to_string(),
            order_type: "LIMIT".to_string(),
        };

        let btc_order = OrderRequest {
            token_id: opportunity.btc_down_token_id.clone(),
            side: "BUY".to_string(),
            size: size_str.clone(),
            price: opportunity.btc_down_price.to_string(),
            order_type: "LIMIT".to_string(),
        };

        let (sol_result, btc_result) = tokio::join!(
            self.api.place_order(&sol_order),
            self.api.place_order(&btc_order)
        );

        match sol_result {
            Ok(response) => info!("SOL Up order placed: {:?}", response),
            Err(e) => warn!("Failed to place SOL Up order: {}", e),
        }

        match btc_result {
            Ok(response) => info!("BTC Down order placed: {:?}", response),
            Err(e) => warn!("Failed to place BTC Down order: {}", e),
        }

        let cost_per_unit = f64::try_from(opportunity.total_cost).unwrap_or(1.0);
        let units = position_size / cost_per_unit;

        let trade_key = format!(
            "{}_{}",
            opportunity.sol_condition_id,
            opportunity.btc_condition_id
        );

        let mut pending = self.pending_trades.lock().await;

        if let Some(existing_trade) = pending.get_mut(&trade_key) {
            existing_trade.units += units;
            existing_trade.investment_amount += position_size;
            info!(
                "   📊 Accumulated trade: Total units: {:.2}, Total investment: ${:.2}",
                existing_trade.units, existing_trade.investment_amount
            );
        } else {
            let pending_trade = PendingTrade {
                sol_token_id: opportunity.sol_up_token_id.clone(),
                btc_token_id: opportunity.btc_down_token_id.clone(),
                sol_condition_id: opportunity.sol_condition_id.clone(),
                btc_condition_id: opportunity.btc_condition_id.clone(),
                investment_amount: position_size,
                units,
                timestamp: std::time::Instant::now(),
            };
            pending.insert(trade_key, pending_trade);
        }
        drop(pending);

        let mut trades = self.trades_executed.lock().await;
        *trades += 1;
        let trades_count = *trades;
        drop(trades);

        info!(
            "✅ Real Trade Executed - Investment: ${:.2} | Expected Profit: ${:.4} | Trades: {}",
            position_size,
            f64::try_from(opportunity.expected_profit).unwrap_or(0.0) * units,
            trades_count
        );

        Ok(())
    }

    fn calculate_position_size(&self, opportunity: &ArbitrageOpportunity) -> f64 {
        let max_size = self.config.max_position_size;
        let cost_per_unit = f64::try_from(opportunity.total_cost).unwrap_or(1.0);

        let units = max_size / cost_per_unit;
        let position_size = (units * cost_per_unit).min(max_size);

        position_size
    }

    pub async fn get_stats(&self) -> (f64, u64) {
        let total = *self.total_profit.lock().await;
        let trades = *self.trades_executed.lock().await;
        (total, trades)
    }
}
