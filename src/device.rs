use tokio::{
    select,
    sync::broadcast,
};

use crate::types::{
    DeviceCmd, DeviceCmdRx, DeviceEvent, DeviceId, DeviceConfig, PriceTick, UserEvent, UserEventTx,
};

pub async fn device_task(
    cfg: DeviceConfig,
    mut price_rx: broadcast::Receiver<PriceTick>,
    mut cmd_rx: DeviceCmdRx,
    user_evt_tx: UserEventTx,
    mut user_evt_rx_for_children: Option<broadcast::Receiver<UserEvent>>,
) {
    // Notify started
    let _ = user_evt_tx.send(UserEvent::Device {
        user: cfg.user,
        device: cfg.id,
        event: DeviceEvent::Started,
    });

    // Track last threshold side to avoid spamming unchanged status
    let mut last_above: Option<bool> = None;

    loop {
        select! {
            res = price_rx.recv() => {
                match res {
                    Ok(tick) if tick.symbol == cfg.symbol => {
                        let above = tick.price > cfg.threshold;
                        if last_above != Some(above) {
                            last_above = Some(above);
                            let _ = user_evt_tx.send(UserEvent::Device {
                                user: cfg.user,
                                device: cfg.id,
                                event: DeviceEvent::Threshold { symbol: cfg.symbol, threshold: cfg.threshold, price: tick.price, above },
                            });
                        }
                    }
                    Ok(_) => {/* not our symbol */}
                    Err(broadcast::error::RecvError::Lagged(_n)) => { /* skip for demo */ }
                    Err(broadcast::error::RecvError::Closed) => {
                        // Upstream gone; shut down
                        let _ = user_evt_tx.send(UserEvent::Device { user: cfg.user, device: cfg.id, event: DeviceEvent::Shutdown });
                        break;
                    }
                }
            }

            maybe = cmd_rx.recv() => {
                match maybe {
                    Some(DeviceCmd::Shutdown) | None => {
                        let _ = user_evt_tx.send(UserEvent::Device { user: cfg.user, device: cfg.id, event: DeviceEvent::Shutdown });
                        break;
                    }
                }
            }
        }
        // After each select tick, opportunistically drain child events without blocking
        if let Some(rx) = &mut user_evt_rx_for_children {
            loop {
                match rx.try_recv() {
                    Ok(UserEvent::Device { user: _, device, event }) => {
                        if cfg.children.contains(&device) {
                            if let DeviceEvent::Threshold { above: true, .. } = event {
                                let _ = user_evt_tx.send(UserEvent::Device { user: cfg.user, device: cfg.id, event: DeviceEvent::ParentNoted { child: device } });
                            }
                        }
                    }
                    Err(broadcast::error::TryRecvError::Empty) => break,
                    Err(broadcast::error::TryRecvError::Lagged(_)) => continue,
                    Err(broadcast::error::TryRecvError::Closed) => { user_evt_rx_for_children = None; break; }
                }
            }
        }
    }
}
