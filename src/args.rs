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

    /// Create a backup of ~/Applications that can be restored from
    #[arg(short, long)]
    pub backup: bool,

    /// Use a different package repo than https://github.com/blueOkiris/aip-man-pkg-list.
    /// To upgrade from that repo, run upgrade with this flag. Works with local repos via file://.
    /// Should be a link to pkgs.json like:
    /// https://raw.githubusercontent.com/blueOkiris/aip-man-pkg-list/main/pkgs.json
    #[arg(short, long)]
    pub repo: Option<String>,

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
        app: String,

        /// Arguments to pass to the application.
        app_args: Option<Vec<String>>
    },

    /// Restore ~/Applications from backup
    Restore
}

