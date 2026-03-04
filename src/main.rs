mod types;
mod price;
mod device;
mod user;

use std::time::Duration;
use tokio::sync::broadcast;

use crate::{
    price::price_feed_task,
    types::PriceTick,
    user::spawn_user_controller,
};

#[tokio::main]
async fn main() {
    // Global price broadcaster for all symbols
    let (price_tx, _drop) = broadcast::channel::<PriceTick>(2048);
    tokio::spawn(price_feed_task(price_tx.clone()));

    // Spawn 8 users, each with ~64 devices initially
    for uid in 0u16..8u16 { // 8 users
        spawn_user_controller(uid, price_tx.clone()).await;
    }

    // Run for a while to observe activity
    tokio::time::sleep(Duration::from_secs(25)).await;
}

