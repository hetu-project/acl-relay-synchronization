//!This module provides a Rust client for interacting with the Nostr protocol,
//!a decentralized messaging platform. The `NostrClient` struct enables
//!convenient management of relays, event filtering, event fetching, and
//!event publishing.

use crate::common::error;
use nostr_sdk::prelude::*;
use std::time::Duration;

/// Configuration for event filtering in Nostr.
/// Includes event kind, tag, and limit for the number of events to fetch.
#[derive(Debug, Clone)]
struct FilterConfig {
    kind: Kind,   // The kind of Nostr event to filter.
    tag: String,  // The tag used for filtering events.
    limit: usize, // Maximum number of events to fetch.
}

impl FilterConfig {
    /// Creates a new `FilterConfig` with the specified kind, tag, and limit.
    fn new(k: Kind, t: &str, l: usize) -> Self {
        Self {
            kind: k,
            tag: t.to_string(),
            limit: l,
        }
    }
}

impl Default for FilterConfig {
    fn default() -> Self {
        /// Provides a default `FilterConfig` with kind as `TextNote`, tag as "waku",
        /// and a limit of 100 events.
        Self {
            kind: Kind::TextNote,
            tag: "waku".to_string(),
            limit: 100,
        }
    }
}

/// A client for interacting with the Nostr protocol.
/// Provides functionality to manage relays, filter and fetch events, and send events.
#[derive(Debug)]
pub struct NostrClient {
    signer: Keys,         // The cryptographic keys used for signing events.
    filter: FilterConfig, // Configuration for filtering events.
    client: Client,       // The underlying Nostr SDK client.
}

impl NostrClient {
    /// Creates a new `NostrClient` with the provided private key and optional relay URL.
    ///
    /// # Arguments
    /// - `priv_key`: A private key string for the Nostr client.
    /// - `relay`: An optional relay URL to connect to.
    ///
    /// # Returns
    /// A `Result` containing the initialized `NostrClient` or an error.
    pub async fn new(priv_key: &str, relay: Option<&str>) -> error::Result<Self> {
        let keys = Keys::parse(priv_key)?;
        let opts = Options::new().gossip(true);
        let client_builder = Client::builder().signer(keys.clone()).opts(opts);
        let client = client_builder.build();

        if let Some(url) = relay {
            client.add_relay(url).await?;
        }
        client.connect().await;

        Ok(Self {
            signer: keys,
            filter: Default::default(),
            client,
        })
    }

    /// Creates a new `NostrClient` with a custom database.
    ///
    /// # Arguments
    /// - `priv_key`: A private key string for the Nostr client.
    /// - `relay`: An optional relay URL to connect to.
    /// - `db`: A database implementation compatible with the Nostr SDK.
    ///
    /// # Returns
    /// A `Result` containing the initialized `NostrClient` or an error.
    pub async fn new_with_db<T: IntoNostrDatabase>(
        priv_key: &str,
        relay: Option<&str>,
        db: T,
    ) -> error::Result<Self> {
        let keys = Keys::parse(priv_key)?;
        let opts = Options::new().gossip(true);
        let client_builder = Client::builder()
            .signer(keys.clone())
            .opts(opts)
            .database(db);
        let client = client_builder.build();

        if let Some(url) = relay {
            client.add_relay(url).await?;
        }
        client.connect().await;

        Ok(Self {
            signer: keys,
            filter: Default::default(),
            client,
        })
    }

    /// Updates the filter configuration for the Nostr client.
    ///
    /// # Arguments
    /// - `k`: The kind of events to filter.
    /// - `t`: The tag used for filtering.
    /// - `l`: The maximum number of events to fetch.
    pub fn set_filter_config(&mut self, k: Kind, t: &str, l: usize) {
        self.filter = FilterConfig::new(k, t, l);
    }

    /// Fetches events from the relay based on the filter configuration.
    ///
    /// # Arguments
    /// - `since`: A timestamp specifying the starting point for fetching events.
    ///
    /// # Returns
    /// A `Result` containing the fetched events or an error.
    pub async fn fetch_from_relay(&self, since: u64) -> error::Result<Events> {
        let filter = Filter::new()
            .kind(self.filter.kind)
            .hashtag(self.filter.tag.clone())
            .since(since.into())
            .limit(self.filter.limit);

        let events = self
            .client
            .fetch_events(vec![filter], Some(Duration::from_secs(10)))
            .await?;

        Ok(events)
    }

    /// Fetches events from the local database based on the filter configuration.
    ///
    /// # Arguments
    /// - `since`: A timestamp specifying the starting point for fetching events.
    ///
    /// # Returns
    /// A `Result` containing the fetched events or an error.
    pub async fn fetch_from_db(&self, since: u64) -> error::Result<Events> {
        let filter = Filter::new()
            .kind(self.filter.kind)
            .hashtag(self.filter.tag.clone())
            .since(since.into())
            .limit(self.filter.limit);

        let events = self.client.database().query(vec![filter]).await?;

        Ok(events)
    }

    /// Sends an event to the Nostr network.
    ///
    /// # Arguments
    /// - `event`: The event to be sent.
    ///
    /// # Returns
    /// A `Result` containing the event ID of the sent event or an error.
    pub async fn send_event(&self, event: Event) -> error::Result<EventId> {
        Ok(self.client.send_event(event).await?.id().to_owned())
    }
}
