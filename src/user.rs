use std::{collections::HashMap, time::Duration};

use rand::Rng;
use tokio::{sync::{broadcast, RwLock, mpsc}, time::{sleep, timeout, Duration as TokioDuration}};

use crate::{
    device::device_task,
    types::{DeviceCmd, DeviceCmdTx, DeviceConfig, DeviceEvent, DeviceId, PriceTick, UserEvent, UserEventTx, UserId, SYMBOLS},
};

pub struct UserCtx {
    pub id: UserId,
    pub event_tx: UserEventTx,
    pub device_bus: RwLock<HashMap<DeviceId, DeviceCmdTx>>, // id -> cmd sender
    next_device_id: RwLock<DeviceId>,
}

impl UserCtx {
    pub fn new(id: UserId, event_tx: UserEventTx) -> Self {
        Self {
            id,
            event_tx,
            device_bus: RwLock::new(HashMap::new()),
            next_device_id: RwLock::new(1),
        }
    }
}

fn random_symbol() -> &'static str {
    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0..SYMBOLS.len());
    SYMBOLS[idx]
}

fn random_threshold() -> f64 {
    let mut rng = rand::thread_rng();
    // around 100 +/- 15
    100.0 + rng.gen_range(-15.0..15.0)
}

fn random_name(id: DeviceId) -> String { format!("Dev-{id}") }

pub async fn spawn_device_for_user(
    user: &UserCtx,
    price_tx: &broadcast::Sender<PriceTick>,
) -> DeviceId {

    // Generate id
    let id = {
        let mut guard = user.next_device_id.write().await;
        let id = *guard;
        *guard += 1;
        id
    };

    // Pick up to 2 existing devices as children
    let children: Vec<DeviceId> = {
        let bus = user.device_bus.read().await;
        let ids: Vec<DeviceId> = bus.keys().copied().collect();
        let mut picks = Vec::new();
        let pick_count = rand::thread_rng().gen_range(0..=2);
        for _ in 0..pick_count {
            if ids.is_empty() { break; }
            let cidx = rand::thread_rng().gen_range(0..ids.len());
            let child = ids[cidx];
            if !picks.contains(&child) { picks.push(child); }
        }
        picks
    };

    let cfg = DeviceConfig {
        user: user.id,
        id,
        name: random_name(id),
        symbol: random_symbol(),
        threshold: random_threshold(),
        children,
    };

    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<DeviceCmd>();
    user.device_bus.write().await.insert(id, cmd_tx);

    let price_rx = price_tx.subscribe();
    let user_evt_rx_for_children = Some(user.event_tx.subscribe());
    let user_evt_tx = user.event_tx.clone();

    tokio::spawn(async move {
        device_task(cfg, price_rx, cmd_rx, user_evt_tx, user_evt_rx_for_children).await;
    });

    id
}

pub async fn shutdown_random_device(user: &UserCtx) {
    let maybe_id = {
        let bus = user.device_bus.read().await;
        if bus.is_empty() { return; }
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(0..bus.len());
        bus.keys().copied().nth(idx)
    };
    if let Some(id) = maybe_id {
        if let Some(tx) = user.device_bus.write().await.remove(&id) {
            let _ = tx.send(DeviceCmd::Shutdown);
        }
    }
}

pub async fn spawn_user_controller(user_id: UserId, price_tx: broadcast::Sender<PriceTick>) {
    // Per-user event bus
    let (evt_tx, _evt_rx_drop) = broadcast::channel::<UserEvent>(4096);
    let user = UserCtx::new(user_id, evt_tx.clone());

    // Initial devices
    for _ in 0..64 {
        let _ = spawn_device_for_user(&user, &price_tx).await;
    }

    // Session flapper: join/leave randomly
    tokio::spawn(async move {
        loop {
            // online window length
            let online_ms: u64 = rand::thread_rng().gen_range(2_000..5_000);
            // online: subscribe and print events with timeout lapses to allow graceful exit
            {
                let mut rx = evt_tx.subscribe();
                let deadline = std::time::Instant::now() + Duration::from_millis(online_ms);
                while std::time::Instant::now() < deadline {
                    // Poll events with small timeout to periodically check the deadline
                    match timeout(TokioDuration::from_millis(250), rx.recv()).await {
                        Ok(Ok(UserEvent::Device { user, device, event })) if user == user_id => {
                            match event {
                                DeviceEvent::Started => {
                                    println!("[USER {user}] device {device} started");
                                }
                                DeviceEvent::Threshold { symbol, threshold, price, above } => {
                                    let side = if above {"above"} else {"below"};
                                    println!("[USER {user}] device {device} {symbol} price={price:.2} is {side} threshold={threshold:.2}");
                                }
                                DeviceEvent::ParentNoted { child } => {
                                    println!("[USER {user}] device {device} observed child {child} threshold event");
                                }
                                DeviceEvent::Shutdown => {
                                    println!("[USER {user}] device {device} shutdown");
                                }
                            }
                        }
                        Ok(Ok(_)) => { /* other user; ignore */ }
                        Ok(Err(_)) => { break; }
                        Err(_) => { /* timeout tick */ }
                    }
                }
            }

            // offline window
            let offline_ms: u64 = rand::thread_rng().gen_range(2_000..5_000);
            sleep(Duration::from_millis(offline_ms)).await;
        }
    });

    // Churn loop: roughly once per second add/remove devices
    tokio::spawn(async move {
        loop {
            let add = rand::random::<bool>();
            if add {
                let _ = spawn_device_for_user(&user, &price_tx).await;
            } else {
                shutdown_random_device(&user).await;
            }
            sleep(Duration::from_millis(1000)).await;
        }
    });
}
