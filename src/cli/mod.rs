//! Module for handling the command-line interface (CLI).  
//!  
//! This module is responsible for parsing command-line arguments and invoking  
//! the appropriate subcommands or functionality based on the user's input.  
//! It typically defines a function, such as `handle_cli`, which serves as the  
//! entry point for the CLI application.

mod cli;
mod migrate_cmd;
mod run_cmd;

pub use cli::handle_cli;
