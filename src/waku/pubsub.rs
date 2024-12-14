/// Module containing the WakuClient implementation.
///
/// This module provides a Rust client for interacting with the Waku protocol, which is a decentralized
/// messaging protocol. The client allows sending and receiving messages, connecting to peers, and
/// retrieving message history.
use crate::common::config::WakuConfig;
use aes_gcm::{Aes256Gcm, KeyInit};
use chrono::Utc;
use nostr_sdk::prelude::Event as NostrEvent;
use rand::thread_rng;
use secp256k1::SecretKey;
use std::io::{self, BufRead};
use std::net::IpAddr;
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::time::{Duration, SystemTime};
use std::{collections::HashSet, str::from_utf8};
use tokio::sync::mpsc::{self};
use waku_bindings::{
    waku_default_pubsub_topic, waku_new, waku_set_event_callback, ContentFilter, Encoding, Event,
    Key, MessageId, Multiaddr, PagingOptions, ProtocolId, Running, StoreQuery, WakuContentTopic,
    WakuLogLevel, WakuMessage, WakuNodeConfig, WakuNodeHandle, WakuPubSubTopic,
};

/// Struct representing a Waku client.
///
/// This struct contains configuration for the client, a handle to the running Waku node, an elliptic
/// curve private key for encryption, an AES key for additional encryption, and topics for content
/// and pubsub.
pub struct WakuClient {
    config: WakuConfig,
    node_handle: WakuNodeHandle<Running>,
    ec_privkey: SecretKey,
    aes_key: Key<Aes256Gcm>,
    content_topic: WakuContentTopic,
    pubsub_topic: WakuPubSubTopic,
}

impl WakuClient {
    /// Struct representing a Waku client.
    ///
    /// This struct contains configuration for the client, a handle to the running Waku node, an elliptic
    /// curve private key for encryption, an AES key for additional encryption, and topics for content
    /// and pubsub.
    pub async fn new(config: WakuConfig) -> Result<WakuClient, String> {
        let node_url = config.node_url.clone();
        let node_addr = config.node_addr.clone();
        let node_config = WakuNodeConfig {
            host: IpAddr::from_str(node_url.as_str()).ok(),
            log_level: Some(WakuLogLevel::Error),
            ..Default::default()
        };

        let node = waku_new(Some(node_config))?;
        let node = node.start()?;
        tracing::info!("Node peer id: {}", node.peer_id()?);

        let address: Multiaddr = node_addr.parse().unwrap();
        let peer_id = node.add_peer(&address, ProtocolId::Relay)?;
        node.connect_peer_with_id(&peer_id, None)?;

        let content_topic: WakuContentTopic = config.content_topic.parse().unwrap();
        let content_filter = ContentFilter::new(
            Some(config.pubsub_topic.parse().unwrap()),
            vec![content_topic.clone()],
        );
        node.relay_subscribe(&content_filter)?;

        let sk = SecretKey::new(&mut thread_rng());
        let ssk = Aes256Gcm::generate_key(&mut thread_rng());

	let pubsub = config.pubsub_topic.clone();

        Ok(WakuClient {
            config,
            ec_privkey: sk,
            aes_key: ssk,
            node_handle: node,
            content_topic,
            pubsub_topic: pubsub.parse().unwrap(),
        })
    }

    fn try_publish_relay_messages(&self, msg: &WakuMessage) -> Result<HashSet<MessageId>, String> {
        Ok(HashSet::from([self
            .node_handle
            .relay_publish_message(msg, None, None)?]))
    }

    fn try_publish_lightpush_messages(
        self,
        msg: &WakuMessage,
    ) -> Result<HashSet<MessageId>, String> {
        let peer_id = self
            .node_handle
            .peers()
            .unwrap()
            .iter()
            .map(|peer| peer.peer_id())
            .find(|id| id.as_str() != self.node_handle.peer_id().unwrap().as_str())
            .unwrap()
            .clone();

        Ok(HashSet::from([self
            .node_handle
            .lightpush_publish(msg, None, peer_id, None)?]))
    }

    /// Sends a message through the Waku relay.
    ///
    /// This method creates a new Waku message, publishes it through the relay, and returns the
    /// message IDs of the successfully sent messages.
    pub async fn send_message(&self, content: String) -> Result<HashSet<MessageId>, String> {
        let message = WakuMessage::new(
            content,
            self.content_topic.clone(),
            1,
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis()
                .try_into()
                .unwrap(),
            Vec::new(),
            false,
        );

        let ids = self
            .try_publish_relay_messages(&message)
            .expect("send relay messages");

        Ok(ids)
    }

    pub async fn listening_message_gowrapper(&self, tx: mpsc::Sender<String>) {
        let mut child = Command::new(self.config.waku_bin.clone())
            .arg("verify")
            .arg("--shard")
            .arg(self.config.shared.clone())
            .arg("--maddr")
            .arg(self.config.node_addr.clone())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let stdout = child.stdout.take().expect("Failed to capture stdout");

        let reader = io::BufReader::new(stdout);

        for line in reader.lines() {
            match line {
                Ok(line) => {
                    println!("Received from Go: {}", line);
                    tx.send(line).await;
                }
                Err(e) => eprintln!("Error reading line: {}", e),
            }
        }

        let status = child.wait().unwrap();
        println!("Go server exited with status: {}", status);
    }

    pub async fn listening_message(&self, tx: mpsc::Sender<NostrEvent>) {
        //let history = self.retrieve_history();

	let content_topic_cl = self.content_topic.clone();
        waku_set_event_callback(move |signal| {
            if let Event::WakuMessage(message) = signal.event() {
                let id = message.message_id();
                tracing::info!("got waku event: {:?}", id);
                let message = message.waku_message();

                if message.content_topic() != &content_topic_cl {
                    return;
                }
                let payload = message.payload().to_vec();
                let msg = from_utf8(&payload).expect("should be valid message");
                match serde_json::from_str::<NostrEvent>(msg) {
                    Ok(event) => {
                        futures::executor::block_on(tx.send(event))
                            .expect("send response to the receiver");
                    }
                    Err(e) => {
                        tracing::error!("{:?}", e);
                    }
                }
            }
        });
    }

    fn retrieve_history(&self) -> waku_bindings::Result<Vec<NostrEvent>> {
        let self_id = self.node_handle.peer_id().unwrap();
        let peer = self
            .node_handle
            .peers()?
            .iter()
            .find(|&peer| peer.peer_id() != &self_id)
            .cloned()
            .unwrap();

        let result = self.node_handle.store_query(
            &StoreQuery {
                pubsub_topic: None,
                content_topics: vec![self.content_topic.clone()],
                start_time: Some(
                    (Duration::from_secs(Utc::now().timestamp() as u64)
                        - Duration::from_secs(60 * 60 * 24))
                    .as_nanos() as usize,
                ),
                end_time: None,
                paging_options: Some(PagingOptions {
                    page_size: 25,
                    cursor: None,
                    forward: true,
                }),
            },
            peer.peer_id(),
            Some(Duration::from_secs(10)),
        )?;

        Ok(result
            .messages()
            .iter()
            .map(|waku_message| {
                let msg = from_utf8(waku_message.payload()).expect("should be valid message");
                serde_json::from_str::<NostrEvent>(msg)
                    .expect("Toy chat messages should be decodeable")
            })
            .collect())
    }
}
