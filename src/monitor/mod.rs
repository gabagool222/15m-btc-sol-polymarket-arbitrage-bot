//! Market monitoring: fetches SOL and BTC 15-minute market data.

pub mod market;
pub use market::{MarketMonitor, MarketSnapshot};
