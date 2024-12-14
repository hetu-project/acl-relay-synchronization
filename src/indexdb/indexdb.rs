//!This module provides functionality for handling and processing Nostr events,
//!converting them into structured data, and sending them to an external
//!IndexDB server for storage or further processing.

use crate::common::error;
use crate::nostr;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Metadata associated with a Nostr event.
#[derive(Serialize, Deserialize, Debug)]
pub struct NostrMetadata {
    pub message: String,
    pub timestamp: u64,
    pub platform: String,
    pub version: String,
    pub clock: u64,
}

/// Represents an authorization event in the Nostr protocol.
#[derive(Serialize, Deserialize, Debug)]
pub struct NostrAuthEvent {
    user: String,
    scope: Vec<String>,
    project_id: String,
    metadata: serde_json::Value,
    r#type: String,
}

/// Defines the content structure for an invitation event.
#[derive(Debug, Serialize, Deserialize)]
struct NostrInviteEventContent {
    inviter: String,
    invitee: String,
    #[serde(rename = "projectId")]
    project_id: String,
    metadata: NostrMetadata,
    #[serde(rename = "type")]
    event_type: String,
}

/// A simplified representation of an invite event.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct InviteMsgEvent {
    from: String,
    to: String,
}

/// Represents a structured invitation message converted from raw events.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct InviteMsg {
    project: String,
    id: String,
    account: String,
    event_type: String,
    event: InviteMsgEvent,
}

impl TryFrom<nostr_sdk::Event> for InviteMsg {
    type Error = ();

    /// Attempts to convert a raw `nostr_sdk::Event` into an `InviteMsg`.
    fn try_from(event: nostr_sdk::Event) -> Result<Self, Self::Error> {
        let invite: NostrInviteEventContent = serde_json::from_str(event.content.as_str()).unwrap();

        Ok(Self {
            project: invite.project_id,
            id: event.id.into(),
            account: event.pubkey.to_string(),
            event_type: invite.event_type,
            event: InviteMsgEvent {
                from: invite.inviter,
                to: invite.invitee,
            },
        })
    }
}

/// A client wrapper for sending events to an IndexDB server.
pub struct IndexdbServer(reqwest::Client);

impl IndexdbServer {
    /// Creates a new IndexdbServer instance with the specified base URL.
    pub fn new() -> Self {
        IndexdbServer(reqwest::Client::new())
    }

    /// Sends an invitation event to the IndexDB server.
    /// Logs the status of the HTTP response.
    pub async fn send_invite_event_to_indexdb(
        &self,
        url: &str,
        event: nostr_sdk::Event,
    ) -> error::Result<()> {
        tracing::info!("got nostr event: {:?}", event);

        let req: InviteMsg = event.try_into().unwrap();
        let response = self.0.post(url).json(&req).send().await.unwrap();

        tracing::info!("{:?}", response);

        if response.status().is_success() {
            tracing::info!("success 200");
        } else {
            tracing::info!("responded with status: {}", response.status());
        }

        Ok(())
    }
}
