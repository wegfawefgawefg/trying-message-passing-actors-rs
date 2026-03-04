use tokio::sync::{broadcast, mpsc};

pub type UserId = u16;
pub type DeviceId = u32;

pub const SYMBOLS: [&str; 8] = [
    "BTCUSDT",
    "ETHUSDT",
    "SOLUSDT",
    "ADAUSDT",
    "XRPUSDT",
    "DOGEUSDT",
    "DOTUSDT",
    "LTCUSDT",
];

#[derive(Debug, Clone)]
pub struct PriceTick {
    pub symbol: &'static str,
    pub price: f64,
}

#[derive(Debug, Clone)]
pub enum DeviceCmd {
    Shutdown,
}

#[derive(Debug, Clone)]
pub enum DeviceEvent {
    Started,
    Threshold {
        symbol: &'static str,
        threshold: f64,
        price: f64,
        above: bool,
    },
    ParentNoted { child: DeviceId },
    Shutdown,
}

#[derive(Debug, Clone)]
pub struct DeviceConfig {
    pub user: UserId,
    pub id: DeviceId,
    pub name: String,
    pub symbol: &'static str,
    pub threshold: f64,
    pub children: Vec<DeviceId>,
}

#[derive(Debug, Clone)]
pub enum UserEvent {
    Device {
        user: UserId,
        device: DeviceId,
        event: DeviceEvent,
    },
}

// Convenience aliases
pub type UserEventTx = broadcast::Sender<UserEvent>;
pub type UserEventRx = broadcast::Receiver<UserEvent>;
pub type DeviceCmdTx = mpsc::UnboundedSender<DeviceCmd>;
pub type DeviceCmdRx = mpsc::UnboundedReceiver<DeviceCmd>;

