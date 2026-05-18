use std::sync::Arc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::sync::Notify;
use tokio::task::JoinHandle;

use crate::observer::{ChannelEvent, Observer, ObserverContext};

/// Cheap-to-clone sender handle. Held by [`ChannelRegistry`] and every
/// channel. `emit` is fire-and-forget — the drain task is responsible for
/// delivering to observers.
#[derive(Clone)]
pub(crate) struct ObserverBus {
    tx: Option<UnboundedSender<(ChannelEvent, ObserverContext)>>,
}

impl ObserverBus {
    pub(crate) fn noop() -> Self {
        Self { tx: None }
    }

    pub(crate) fn emit(&self, event: ChannelEvent, ctx: ObserverContext) {
        if let Some(tx) = &self.tx {
            let _ = tx.send((event, ctx));
        }
    }
}

/// Owning handle for the drain task and the observers. Server keeps this
/// alive until shutdown.
pub(crate) struct ObserverBusHandle {
    observers: Vec<Arc<dyn Observer>>,
    join: Option<JoinHandle<()>>,
    notify: Arc<Notify>,
}

impl ObserverBusHandle {
    pub(crate) async fn shutdown(mut self) {
        self.notify.notify_one();
        if let Some(handle) = self.join.take() {
            let _ = handle.await;
        }
        for obs in &self.observers {
            obs.shutdown().await;
        }
    }
}

pub(crate) fn start_observer_bus(
    observers: Vec<Arc<dyn Observer>>,
) -> (ObserverBus, ObserverBusHandle) {
    if observers.is_empty() {
        return (
            ObserverBus::noop(),
            ObserverBusHandle {
                observers: Vec::new(),
                join: None,
                notify: Arc::new(Notify::new()),
            },
        );
    }

    let (tx, mut rx) = unbounded_channel::<(ChannelEvent, ObserverContext)>();
    let notify = Arc::new(Notify::new());

    let drain_observers = observers.clone();
    let drain_notify = notify.clone();
    let handle = tokio::spawn(async move {
        loop {
            tokio::select! {
                biased;
                _ = drain_notify.notified() => {
                    while let Ok((event, ctx)) = rx.try_recv() {
                        for obs in &drain_observers {
                            obs.on_event(event.clone(), ctx.clone()).await;
                        }
                    }
                    break;
                }
                msg = rx.recv() => {
                    match msg {
                        Some((event, ctx)) => {
                            for obs in &drain_observers {
                                obs.on_event(event.clone(), ctx.clone()).await;
                            }
                        }
                        None => break,
                    }
                }
            }
        }
    });

    (
        ObserverBus { tx: Some(tx) },
        ObserverBusHandle {
            observers,
            join: Some(handle),
            notify,
        },
    )
}
