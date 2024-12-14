//! The `App` module manages the application state and provides methods for integrating
//! with the `nostr` protocol, `waku` protocol, and other external systems like indexdb.
//! It utilizes asynchronous processing to handle communication between different systems.
use crate::common::config::Config;
use crate::common::error;
use crate::db;
use crate::nostr;
use crate::waku;
use crate::indexdb;
use base64;
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE},
    Client,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

/// The `App` struct holds the application state, including configurations, database storage,
/// and clients for external protocols like `nostr`, `waku`, and HTTP.
pub struct App {
    /// Database storage for managing application data.
    store: db::Storage,
    /// Application configuration containing settings for various integrations.
    config: Config,
    /// Client for interacting with the `nostr` protocol.
    nostr_client: Arc<nostr::NostrClient>,
    /// Client for interacting with the `waku` protocol.
    waku_client: Arc<waku::WakuClient>,
    /// HTTP client for sending data to external APIs, such as `indexdb`.
    indexdb_client: Arc<indexdb::IndexdbServer>,
}

/// Represents a message sent through the `waku` protocol.
/// Contains the payload data and content topic.
#[derive(Debug, Serialize, Deserialize)]
pub struct WakuMessage {
    /// Encoded payload of the message.
    payload: String,
    /// Topic under which the message is categorized.
    content_topic: String,
}

impl App {
    /// Creates a new instance of the `App` with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration object containing settings for the application.
    ///
    /// # Returns
    ///
    /// An `App` instance wrapped in a `Result`.
    pub async fn new(config: Config) -> error::Result<App> {
        // Initialize database storage.
        let store = db::Storage::new(config.database.clone()).await;

        // Initialize the nostr client.
        let nclient = nostr::NostrClient::new(
            config.nostr.priv_key.as_str(),
            Some(config.nostr.ws_url.as_str()),
        )
        .await?;

        // Initialize the waku client.
        let wclient = waku::WakuClient::new(config.waku.clone()).await.unwrap();

        // Return the app instance.
        Ok(App {
            store,
            config:config.clone(),
            nostr_client: Arc::new(nclient),
            waku_client: Arc::new(wclient),
            indexdb_client: Arc::new(indexdb::IndexdbServer::new()),
        })
    }

    /// Fetches events from `nostr` and sends them to the `waku` protocol.
    ///
    /// This method continuously retrieves events from the `nostr` relay, encodes them,
    /// and forwards them to a `waku` node using its API.
    pub async fn from_nostr_to_waku(&self) {
        let (tx, mut rx) = mpsc::channel(100);
        let wclient = self.waku_client.clone();
        let client = Client::new();
        let url = self.config.waku.send_api.clone();
        let content_topic = self.config.waku.content_topic.clone();

        // Spawn a background task to process and send events to Waku.
        tokio::task::spawn(async move {
            while let Some(event) = rx.recv().await {
                // Encode the event payload in base64 format.
                let encoded_payload = base64::encode(serde_json::to_string(&event).unwrap());

                // Prepare the HTTP request body.
                let body = json!({
                    "payload": encoded_payload,
                    "contentTopic": content_topic
                });

                // Send the payload to the Waku node.
                let response = client
                    .post(url.clone())
                    .header("Content-Type", "application/json")
                    .json(&body)
                    .send()
                    .await
                    .unwrap();

                tracing::info!("Response from server: {}", response.status());
                match response.text().await {
                    Ok(body) => tracing::info!("Response from server: {}", body),
                    Err(e) => tracing::error!("Response from server: {}", e),
                }
            }
        });

        // Main loop for fetching events from Nostr and forwarding them to Waku.
        loop {
            // Retrieve the last fetch time from the database.
            let mut last_fetch_time = self.store.get_last_update(0).await.unwrap();

            // fetch nostr events
            let events = self
                .nostr_client
                .fetch_from_relay(last_fetch_time)
                .await
                .unwrap();

            // Process each event and send it to the Waku client.
            for event in events.into_iter() {
                if let Some(_) = self.store.is_event_existed(event.id.into()).await {
                    if event.created_at.as_u64() > last_fetch_time {
                        last_fetch_time = event.created_at.as_u64();
                    }

                    self.store.add_new_event(event.id.into()).await.unwrap();

                    let _ = tx.send(event).await;
                }
            }

            //update last fetch time in database
            self.store
                .update_last_update(last_fetch_time)
                .await
                .unwrap();

            tokio::time::sleep(Duration::from_secs(10)).await
        }
    }

    /// Listens for events from the `waku` protocol and forwards them to the `nostr` client.
    pub async fn from_waku_to_nostr(&self) {
        let (tx, mut rx) = mpsc::channel(100);

        let wclient = self.waku_client.clone();
        tokio::task::spawn(async move {
            wclient.listening_message_gowrapper(tx).await;
        });

        //self.waku_client.listening_message(tx).await;

        let nclient = self.nostr_client.clone();
        while let Some(event) = rx.recv().await {
            tracing::info!("got event: {:?}", event);
            //let _ = nclient.send_event(event).await;
        }
    }

    /// Fetches events from `nostr` and sends them to an indexdb service.
    ///
    /// This method continuously retrieves events from the `nostr` relay and forwards them
    /// to an external indexdb service for indexing.
    pub async fn from_nostr_to_indexdb(&self) {
        let (tx, mut rx) = mpsc::channel::<nostr_sdk::Event>(100);
        let iclient = self.indexdb_client.clone();
	let invite_url = self.config.indexdb_backend.invite_url.clone();
        tokio::task::spawn(async move {
            while let Some(event) = rx.recv().await {
                let _ = iclient
                    .send_invite_event_to_indexdb(invite_url.as_str(), event)
                    .await;
            }
        });

        loop {
            // fetch last fetch time from database
            let mut last_fetch_time = self.store.get_last_update(0).await.unwrap();

            // fetch nostr events
            let events = self
                .nostr_client
                .fetch_from_relay(last_fetch_time)
                .await
                .unwrap();

            //process events
            for event in events.into_iter() {
                if let Some(_) = self.store.is_event_existed(event.id.into()).await {
                    if event.created_at.as_u64() > last_fetch_time {
                        last_fetch_time = event.created_at.as_u64();
                    }

                    self.store.add_new_event(event.id.into()).await.unwrap();

                    let _ = tx.send(event).await;
                }
            }

            //update last fetch time in database
            self.store
                .update_last_update(last_fetch_time)
                .await
                .unwrap();

            tokio::time::sleep(Duration::from_secs(10)).await
        }
    }
}
