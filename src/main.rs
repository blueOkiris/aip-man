/*
 * Author(s): Dylan Turner
 * Description: Entry point for the App Image Package Manager
 */

mod args;

use clap::Parser;
use crate::args::{
    Args, Commands
};

fn main() {
    match Args::parse().command {
        Commands::Install { .. } => {
            // TODO: Install packages
        }, Commands::Remove { .. } => {
            // TODO: Remove packages
        }, Commands::Upgrade => {
            // TODO: Upgrade packages
        }, Commands::List => {
            // TODO: List installed packages
        }, Commands::Run { .. } => {
            // TODO: Launch an installed application
        }
    }
}

