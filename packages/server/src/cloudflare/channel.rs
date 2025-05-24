use std::collections::HashMap;

use super::{cloudflare_websocket_upgrade::WebSocketUpgrade, get_config};
use axum::response::IntoResponse;
use protocol::{
    ClientMessage,
    ServerMessage, ServerPresenceMessage,
    StorageSyncMessage, StorageType, StorageUpdateMessage,
};
use worker::{wasm_bindgen, WebSocketIncomingMessage};

use crate::{
    channel::Channel,
    config::Config,
    storage::{
        self,
        yjs::{init_storage, YjsStorage},
        Storage, StorageError,
    },
};

#[worker::durable_object]
pub struct CloudflareChannel {
    state: worker::State,
    user_state: HashMap<String, (String, i32)>,
    channel_name: Option<String>,
    tenant_id: Option<String>,
    #[cfg(feature = "webhook")]
    queue_name: Option<String>,
    storage_initialized: bool,
    config: Config,
    storage: Option<Box<dyn storage::Storage + Send + Sync>>,
    provider: YjsStorage,
    env: worker::Env,
}

impl CloudflareChannel {
    #[cfg(feature = "webhook")]
    async fn participant_removed(&self, participant_id: &String) -> Result<(), worker::Error> {
        let queue = self.env.queue(self.queue_name.as_ref().unwrap().as_str())?;

        let participant_count = self.state.get_websockets().len() - 1;

        participant_removed(
            &self.channel_name.clone().unwrap(),
            &self.tenant_id,
            participant_id,
            &participant_count,
            queue,
        )
        .await
        .ok();

        Ok(())
    }

    async fn create_storage(
        &mut self,
        storage_type: Option<StorageType>,
        channel_name: &String,
    ) -> Result<(), StorageError> {
        if let Some(storage_type) = storage_type {
            match storage_type {
                StorageType::Yjs => {
                    let yjs_storage: Box<dyn storage::Storage + Send + Sync> =
                        Box::new(init_storage());

                    self.storage = Some(yjs_storage);

                    if !self.storage_initialized {
                        self.storage
                            .as_ref()
                            .unwrap()
                            .init_storage_from_endpoint(
                                channel_name,
                                &self.config.storage_endpoint,
                                &self.tenant_id,
                            )
                            .await?;

                        self.storage_initialized = true;
                    }
                }
            }
        }

        Ok(())
    }
}

#[worker::durable_object]
impl worker::DurableObject for CloudflareChannel {
    fn new(state: State, env: worker::Env) -> Self {
        // If we are here that means the other fetch request was able to parse the config
        let config = get_config(&env).unwrap();

        Self {
            state,
            env,
            channel_name: None,
            config,
            storage: None,
            storage_initialized: false,
            user_state: HashMap::new(),
            provider: YjsStorage::new(yrs::Doc::new()),
            tenant_id: None,
            #[cfg(feature = "webhook")]
            queue_name: None,
        }
    }

    #[must_use]
    async fn fetch(&mut self, req: worker::Request) -> worker::Result<worker::Response> {
        let query_params: HashMap<String, String> = req.query()?;

        #[cfg(feature = "webhook")]
        self.queue_name
            .replace(req.headers().get("x-queue-name").unwrap().unwrap());

        let channel_name = query_params.get("channel_name").unwrap();

        let tenant_id = {
            #[cfg(feature = "multi-tenant")]
            {
                query_params.get("tenant_id").cloned()
            }

            #[cfg(not(feature = "multi-tenant"))]
            {
                None
            }
        };

        self.tenant_id = tenant_id.clone();

        let storage_type: Option<StorageType> = query_params
            .get("storage_type")
            .map(|storage_type| format!("\"{storage_type}\""))
            .map(|storage_type| serde_json::from_str(&storage_type).ok())
            .flatten();

        let socket = WebSocketUpgrade::try_from(req);

        let socket = match socket {
            Ok(socket) => socket,
            Err(e) => return worker::Response::try_from(e.into_response()),
        };

        self.channel_name.replace(channel_name.clone());

        if self.storage.is_none() {
            match self.create_storage(storage_type, channel_name).await {
                Err(_e) => {
                    // TODO notify the client we weren't able to sync the data from the endpoint
                }
                Ok(()) => {}
            }
        }

        let participant_id = uuid::Uuid::new_v4().to_string();

        #[cfg(feature = "webhook")]
        let queue = self.env.queue(self.queue_name.as_ref().unwrap().as_str())?;
        #[cfg(feature = "webhook")]
        let websocket_count = self.state.get_websockets().len() + 1;
        #[cfg(feature = "webhook")]
        let channel_name = channel_name.clone();

        worker::Response::try_from(socket.on_upgrade(
            &self.state,
            &vec![participant_id.clone().as_str()],
            move |_ws| async move {
                #[cfg(feature = "webhook")]
                participant_added(
                    &channel_name,
                    &tenant_id,
                    &participant_id,
                    &websocket_count,
                    queue,
                )
                .await
                .ok();
            },
        ))
    }

    async fn websocket_message(
        &mut self,
        ws: worker::WebSocket,
        message: WebSocketIncomingMessage,
    ) -> worker::Result<()> {
        let tags = self.state.get_tags(&ws);

        let participant_id = &tags[0];

        match message {
            WebSocketIncomingMessage::String(message) => {
                if message == "ping" {
                    ws.send(&"pong").ok();
                    return Ok(());
                }

                let message: ClientMessage = match serde_json::from_str(&message) {
                    Ok(message) => message,
                    Err(e) => {
                        web_sys::console::log_1(&format!("Error parsing message: {}", e).into());
                        return Err(worker::Error::SerdeJsonError(e));
                    }
                };

                match message {
                    ClientMessage::ClientPresenceMessage(msg) => {
                        self.user_state
                            .insert(participant_id.clone(), (msg.presence.clone(), msg.clock));

                        self.broadcast(
                            ServerMessage::ServerPresenceMessage(ServerPresenceMessage {
                                id: participant_id.clone(),
                                clock: msg.clock,
                                presence: Some(msg.presence),
                            }),
                            Some(participant_id),
                        );
                    }
                    ClientMessage::StorageSyncMessage(msg) => {
                        match self.handle_storage_sync_message(msg) {
                            Ok(Some(msgs)) => {
                                for msg in msgs {
                                    match ws.send(&ServerMessage::StorageSyncMessage(msg))
                                    {
                                        Err(e) => {
                                            web_sys::console::log_1(
                                                &format!("Error sending message: {}", e).into(),
                                            );
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            Err(e) => {
                                web_sys::console::log_1(
                                    &format!("Could not sync storage: {}", e).into(),
                                );
                            }
                            _ => {}
                        }
                    }
                    ClientMessage::StorageUpdateMessage(msg) => {
                        match self.handle_storage_update_message(msg, participant_id) {
                            Err(e) => {
                                web_sys::console::log_1(
                                    &format!("Could not handle storage update: {}", e).into(),
                                );
                            }
                            _ => {}
                        }
                    }
                    ClientMessage::ProviderSyncMessage(msg) => {
                        match self.handle_provider_sync_message(msg) {
                            Ok(Some(msgs)) => {
                                for msg in msgs {
                                    match ws.send(&ServerMessage::ProviderSyncMessage(msg))
                                    {
                                        Err(e) => {
                                            web_sys::console::log_1(
                                                &format!("Error sending message: {}", e).into(),
                                            );
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            Err(e) => {
                                web_sys::console::log_1(
                                    &format!("Could not sync storage: {}", e).into(),
                                );
                            }
                            _ => {}
                        }
                    }
                    ClientMessage::ProviderUpdateMessage(msg) => {
                        match self.handle_provider_update_message(msg, participant_id) {
                            Err(e) => {
                                web_sys::console::log_1(
                                    &format!("Could not handle storage update: {}", e).into(),
                                );
                            }
                            _ => {}
                        }
                    }
                    ClientMessage::CustomMessage(msg) => {
                        self.broadcast(ServerMessage::CustomMessage(msg), Some(participant_id));
                    }
                }
            }
            WebSocketIncomingMessage::Binary(_message) => {
                todo!()
            }
        }

        Ok(())
    }

    async fn websocket_close(
        &mut self,
        ws: worker::WebSocket,
        _code: usize,
        _reason: String,
        _was_clean: bool,
    ) -> worker::Result<()> {
        let tags = self.state.get_tags(&ws);

        let participant_id = &tags[0];

        let user_state = self.user_state.remove(participant_id);

        match user_state {
            Some((_, clock)) => {
                self.broadcast(
                    ServerMessage::ServerPresenceMessage(ServerPresenceMessage {
                        id: participant_id.clone(),
                        clock,
                        presence: None,
                    }),
                    Some(participant_id),
                );
            }
            None => {}
        }

        #[cfg(feature = "webhook")]
        self.participant_removed(participant_id).await?;

        Ok(())
    }
}

impl Channel for CloudflareChannel {
    fn broadcast(&self, message: ServerMessage, exclude_id: Option<&String>) {
        self.state.get_websockets().iter().for_each(|ws| {
            let tags = self.state.get_tags(&ws);

            let participant_id = &tags[0];

            if exclude_id.is_some_and(|id| *id == *participant_id) {
                return;
            }

            match ws.send(&message) {
                Err(e) => {
                    web_sys::console::log_1(&format!("Error sending message: {}", e).into());
                }
                _ => {}
            };
        });
    }

    fn handle_storage_sync_message(
        &self,
        message: StorageSyncMessage,
    ) -> Result<Option<Vec<StorageSyncMessage>>, StorageError> {
        if let Some(storage) = &self.storage {
            return storage.handle_sync_message(&message);
        }

        Ok(None)
    }

    fn handle_storage_update_message(
        &self,
        message: StorageUpdateMessage,
        participant_id: &String,
    ) -> Result<(), StorageError> {
        if let Some(storage) = &self.storage {
            storage.handle_update_message(&message)?;
        }

        self.broadcast(ServerMessage::StorageUpdateMessage(message), Some(participant_id));

        Ok(())
    }

    fn handle_provider_sync_message(
        &self,
        message: StorageSyncMessage,
    ) -> Result<Option<Vec<StorageSyncMessage>>, StorageError> {
        self.provider.handle_sync_message(&message)
    }

    fn handle_provider_update_message(
        &self,
        message: StorageUpdateMessage,
        participant_id: &String,
    ) -> Result<(), StorageError> {
        self.provider.handle_update_message(&message)?;

        self.broadcast(ServerMessage::StorageUpdateMessage(message), Some(participant_id));

        Ok(())
    }
}

#[cfg(feature = "webhook")]
#[cfg(not(feature = "multi-tenant"))]
async fn participant_added(
    channel_name: &String,
    _tenant_id: &Option<String>,
    participant_id: &String,
    participant_count: &usize,
    queue: worker::Queue,
) {
    use web_time::{SystemTime, UNIX_EPOCH};

    use protocol::{NewParticipant, WebhookMessage};

    use crate::cloudflare::CloudflareQueueMessage;

    let start = SystemTime::now();

    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let webhook_message = WebhookMessage::NewParticipant(NewParticipant {
        id: uuid::Uuid::new_v4().into(),
        channel_name: channel_name.clone(),
        timestamp: since_the_epoch.as_millis() as u64,
        participant_id: participant_id.clone(),
        num_pariticipants: *participant_count,
    });

    match queue
        .send(CloudflareQueueMessage {
            message: webhook_message,
        })
        .await
    {
        Ok(_) => {}
        Err(e) => {
            web_sys::console::log_1(&format!("Error sending message: {}", e).into());
        }
    }
}

#[cfg(feature = "webhook")]
#[cfg(feature = "multi-tenant")]
async fn participant_added(
    channel_name: &String,
    tenant_id: &Option<String>,
    participant_id: &String,
    participant_count: &usize,
    queue: worker::Queue,
) -> Result<(), worker::Error> {
    use web_time::{SystemTime, UNIX_EPOCH};

    use protocol::{NewParticipant, WebhookMessage};

    use crate::cloudflare::CloudflareQueueMessage;

    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let webhook_message = WebhookMessage::NewParticipant(NewParticipant {
        id: uuid::Uuid::new_v4().into(),
        channel_name: channel_name.clone(),
        timestamp: since_the_epoch.as_millis() as u64,
        participant_id: participant_id.clone(),
        num_pariticipants: *participant_count,
    });

    queue
        .send(CloudflareQueueMessage {
            message: webhook_message,
            tenant_id: tenant_id.clone().unwrap(),
        })
        .await?;

    Ok(())
}

#[cfg(feature = "webhook")]
#[cfg(not(feature = "multi-tenant"))]
async fn participant_removed(
    channel_name: &String,
    _tenant_id: &Option<String>,
    participant_id: &String,
    participant_count: &usize,
    queue: worker::Queue,
) {
    use web_time::{SystemTime, UNIX_EPOCH};

    use protocol::{RemoveParticipant, WebhookMessage};

    use crate::cloudflare::CloudflareQueueMessage;

    let start = SystemTime::now();

    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let webhook_message = WebhookMessage::RemoveParticipant(RemoveParticipant {
        id: uuid::Uuid::new_v4().into(),
        channel_name: channel_name.clone(),
        timestamp: since_the_epoch.as_millis() as u64,
        participant_id: participant_id.clone(),
        num_pariticipants: *participant_count,
    });

    match queue
        .send(CloudflareQueueMessage {
            message: webhook_message,
        })
        .await
    {
        Ok(_) => {}
        Err(e) => {
            web_sys::console::log_1(&format!("Error sending message: {}", e).into());
        }
    }
}

#[cfg(feature = "webhook")]
#[cfg(feature = "multi-tenant")]
async fn participant_removed(
    channel_name: &String,
    tenant_id: &Option<String>,
    participant_id: &String,
    participant_count: &usize,
    queue: worker::Queue,
) -> Result<(), worker::Error> {
    use web_time::{SystemTime, UNIX_EPOCH};

    use protocol::{RemoveParticipant, WebhookMessage};

    use crate::cloudflare::CloudflareQueueMessage;

    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let webhook_message = WebhookMessage::RemoveParticipant(RemoveParticipant {
        id: uuid::Uuid::new_v4().into(),
        channel_name: channel_name.clone(),
        timestamp: since_the_epoch.as_millis() as u64,
        participant_id: participant_id.clone(),
        num_pariticipants: *participant_count,
    });

    match queue
        .send(CloudflareQueueMessage {
            message: webhook_message,
            tenant_id: tenant_id.clone().unwrap(),
        })
        .await
    {
        Ok(_) => {}
        Err(e) => {
            web_sys::console::log_1(&format!("Error sending message: {}", e).into());
        }
    }

    Ok(())
}
