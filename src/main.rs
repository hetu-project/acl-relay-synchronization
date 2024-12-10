mod cli;
mod common;
mod db;
mod nostr;
mod services;
mod waku;
mod indexdb;

use crate::common::consts::LOG_PATH;
use crate::common::logging;

#[tokio::main]
async fn main() {
    logging::logging_init(LOG_PATH).unwrap();

    cli::handle_cli().await;
}
