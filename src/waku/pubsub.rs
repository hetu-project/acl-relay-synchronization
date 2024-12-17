/// Module containing the WakuClient implementation.
///
/// This module provides a Rust client for interacting with the Waku protocol, which is a decentralized
/// messaging protocol. The client allows sending and receiving messages, connecting to peers, and
/// retrieving message history.
use crate::common::config::WakuConfig;
use aes_gcm::{Aes256Gcm, KeyInit};
use chrono::Utc;
use libloading::{Library, Symbol};
use nostr_sdk::prelude::Event as NostrEvent;
use rand::thread_rng;
use secp256k1::SecretKey;
use std::ffi::c_void;
use std::io::{self, BufRead};
use std::net::IpAddr;
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use std::{collections::HashSet, str::from_utf8};
use tokio::sync::mpsc;

pub const dns_url: &str = "enrtree://AMOJVZX4V6EXP7NTJPMAYJYST2QP6AJXYW76IU6VGJS7UVSNDYZG4@boot.prod.status.nodes.status.im";

pub struct WakuNodeHandle {
    pub ctx: WakuNodeContext,
}

pub struct WakuNodeContext {
    pub obj_ptr: *mut c_void,
}

unsafe impl Sync for WakuNodeHandle {}
unsafe impl Send for WakuNodeHandle {}

#[derive(Debug)]
pub struct Response {
    pub payload: String,
}

/// Struct representing a Waku client.
///
/// This struct contains configuration for the client, a handle to the running Waku node, an elliptic
/// curve private key for encryption, an AES key for additional encryption, and topics for content
/// and pubsub.
pub struct WakuClient {
    config: WakuConfig,
    node_handle: WakuNodeHandle,
    content_topic: String,
    pubsub_topic: String,
}

impl WakuClient {
    /// Struct representing a Waku client.
    ///
    /// This struct contains configuration for the client, a handle to the running Waku node, an elliptic
    /// curve private key for encryption, an AES key for additional encryption, and topics for content
    /// and pubsub.
    pub fn new(config: WakuConfig) -> Result<WakuClient, String> {
        println!("{:?}", config);
        unsafe {
            let lib = Library::new(config.waku_dylib.clone()).unwrap();
            let waku_new: Symbol<
                unsafe fn(
                    usize,
                    Vec<usize>,
                    &str,
                    Option<SecretKey>,
                    &str,
                ) -> Result<WakuNodeHandle, String>,
            > = lib.get(b"waku_new_wrapper").unwrap();

            let node = waku_new(
                config.cluster_id,
                config.shared.clone(),
                config.dns_url.as_str(),
                config
                    .key
                    .clone()
                    .map(|k| SecretKey::from_str(k.as_str()).unwrap()),
                config.pubsub_topic.as_str(),
            )
            .unwrap();

            Ok(WakuClient {
                config: config.clone(),
                node_handle: node,
                content_topic: config.content_topic.clone(),
                pubsub_topic: config.pubsub_topic.clone(),
            })
        }
    }

    /// Sends a message through the Waku relay.
    ///
    /// This method creates a new Waku message, publishes it through the relay, and returns the
    /// message IDs of the successfully sent messages.
    pub fn send_message(&self, content: String) -> Result<(), String> {
        unsafe {
            let lib = Library::new(self.config.waku_dylib.clone()).unwrap();
            let waku_send: Symbol<
                unsafe fn(&WakuNodeHandle, &str, &str, String) -> Result<(), String>,
            > = lib.get(b"waku_send").unwrap();

            waku_send(
                &self.node_handle,
                &self.pubsub_topic,
                &self.content_topic,
                content,
            );
        }

        Ok(())
    }

    pub fn listening_message(&self, tx: mpsc::Sender<Response>) {
        unsafe {
            let lib = Library::new(self.config.waku_dylib.clone()).unwrap();
            let waku_listen: Symbol<
                unsafe fn(
                    &WakuNodeHandle,
                    &str,
                    &str,
                    mpsc::Sender<Response>,
                ) -> Result<(), String>,
            > = lib.get(b"waku_listen").unwrap();

            waku_listen(
                &self.node_handle,
                self.pubsub_topic.as_str(),
                self.content_topic.as_str(),
                tx,
            );
        }
    }
}
