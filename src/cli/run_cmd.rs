//! Module for handling configuration-related commands.  
//!  
//! This module defines the `ConfigCmd` struct which represents the configuration  
//! subcommand parsed from the command line. It also contains the logic to load  
//! and handle configuration files specified by the user.

use crate::common::config;
use crate::services::App;
use clap::Parser;

/// Represents the configuration subcommand parsed from the command line.  
///  
/// This struct is derived from the `clap::Parser` trait to automatically generate  
/// the command line interface for the configuration subcommand. It contains a  
/// single field `file` which represents the path to the configuration file.  
#[derive(Debug, Clone, Parser)]
pub struct RunCmd {
    /// The direction of event:
    /// 'n2w' - from nostr to waku.
    /// 'w2n' - from waku to nostr.
    /// 'n2i' - from waku to index db.
    #[arg(short, long, required = true)]
    direction: String,

    /// The path to the configuration file.  
    #[arg(short, long, value_name = "FILE", required = true)]
    config_file: String,
}

impl RunCmd {
    /// Handles the execution of the configuration subcommand.  
    pub async fn run(&self) {
        let config = config::Config::load_config(self.config_file.clone().into()).unwrap();
        let server = App::new(config).await.unwrap();
        tracing::info!("{:?}", "HH");

        match self.direction.as_str() {
            "n2w" => server.from_nostr_to_waku().await,
            "w2n" => server.from_waku_to_nostr().await,
            "n2i" => server.from_nostr_to_indexdb().await,
            _ => tracing::error!("unkown direction"),
        }
    }
}
