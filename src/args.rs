/*
 * Author(s): Dylan Turner
 * Description: Parse cli arguments
 */

use clap::{
    Parser, Subcommand
};

/// Structure defining CLI command and arguments.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Ask before changing package information
    #[arg(short, long)]
    pub ask: bool,

    /// One of the commands: install <pkg>, remove <pkg>, upgrade, etc.
    #[command(subcommand)]
    pub command: Commands
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Installs an AppImage from the global repo.
    Install {
        /// Package to install.
        package: String
    },

    /// Removes an installed AppImage.
    Remove {
        /// Package to uninstall.
        package: String
    },

    /// Upgrade installed packages.
    Upgrade,

    /// List installed packages.
    List,

    /// Run an installed application.
    Run {
        /// Installed application to run.
        app: String
    }
}

