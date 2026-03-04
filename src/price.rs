use std::time::Duration;

use rand::Rng;
use tokio::time::interval;

use crate::types::{PriceTick, SYMBOLS};

fn start_price() -> f64 {
    let mut rng = rand::thread_rng();
    100.0 + rng.gen_range(-10.0..10.0)
}

/// Broadcasts a simple random walk for each symbol.
pub async fn price_feed_task(tx: tokio::sync::broadcast::Sender<PriceTick>) {
    // symbol, price, volatility
    let mut series: Vec<(&'static str, f64, f64)> = SYMBOLS
        .into_iter()
        .map(|s| (s, start_price(), 0.8))
        .collect();

    let mut tick = interval(Duration::from_millis(300));
    loop {
        tick.tick().await;
        for (sym, px, vol) in &mut series {
            let step: f64 = (rand::random::<f64>() - 0.5) * *vol;
            *px = (*px + step).max(0.01);
            let _ = tx.send(PriceTick {
                symbol: sym,
                price: *px,
            });
        }
    }
}

