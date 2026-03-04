Goal
 - Simulate 8 users, each with ~64 devices (≈1024 total).
 - 8 coins; devices subscribe to their coin's price feed via broadcast.
 - Device has: id, user_id, coin, threshold. Emits events:
   - Started, Threshold { above|below, price }, Shutdown
 - Users join/leave randomly; when online, they "have a WS" that prints their events.
 - Users can spin devices up/down roughly once per second (churn).
 - Some devices have children. Devices can observe children and react to their events.

High-Level Architecture
 - Global price broadcaster: publishes PriceTick per coin at a fixed cadence.
 - Per-user event bus (broadcast): devices publish UserEvent(DeviceEvent) → user sessions subscribe/unsubscribe.
 - Per-device command channel (mpsc): device listens for Shutdown, etc.
 - Device dependency via event bus: parents subscribe to per-user DeviceEvent broadcast and react to their children’s events.

Concurrency Model
 - Price feed: single task broadcasting ticks for 8 symbols.
 - 8 user controllers: each maintains devices and a DeviceBus (id → mpsc::Sender<DeviceCmd>), churn loop, and a user-session loop that goes online/offline.
 - 1024 device tasks: each select! on price feed, command channel, and (optionally) device-event broadcast if it has children.

Event Flow
 - PriceTick(symbol, price) → Device if its symbol matches.
 - Device on tick: compares price to threshold; emits Threshold event (above/below) and dedupes chatter by emitting only on state change.
 - Device lifecycle emits Started and Shutdown.
 - Parent devices listen to per-user DeviceEvent broadcast and react when child Threshold events fire.
 - User session prints UserEvents when online; devices continue regardless of session state.

File Layout
 - src/types.rs
   - Common types: UserId, DeviceId, symbols, PriceTick, DeviceCmd, DeviceEvent, UserEvent, DeviceConfig.
 - src/price.rs
   - price_feed_task(symbols, tx): broadcast random-walk ticks.
 - src/device.rs
   - device_task(cfg, price_rx, cmd_rx, user_evt_tx, device_evt_rx_opt): core device loop with threshold logic and lifecycle.
 - src/user.rs
   - UserCtx: per-user state, event bus, device bus (id→cmd sender).
   - spawn_user(ctx, price_tx): churn loop (spawn/shutdown devices), session loop (join/leave), helpers.
 - src/sim.rs (optional)
   - Orchestrates 8 users and global price feed.
 - src/main.rs
   - Bootstrap and run the sim for a bounded duration.

Simplifications (Demo-Oriented)
 - Parent/child device communication is modeled by parents listening to user-scoped DeviceEvents and reacting when child ids match.
 - Commands are minimal (Shutdown). Threshold is static per device for now.
 - Event printing stands in for WebSocket IO.

Tuning Knobs
 - SYMBOLS: 8
 - USERS: 8
 - DEVICES_PER_USER: 64
 - PRICE_TICK_MS: 300–500ms (randomized per tick)
 - CHURN_INTERVAL_MS: 1000ms
 - SESSION_ONLINE/OFFLINE windows: 2–5s random

Success Criteria
 - Runs without panics.
 - Prints per-user event streams when users are online.
 - Shows device start/shutdown and threshold above/below transitions.
 - Demonstrates parent reacting to child threshold events.

