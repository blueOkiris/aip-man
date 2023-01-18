/*
 * Author(s): Dylan Turner
 * Description: Entry point for the App Image Package Manager
 */

mod args;
mod pkg;

use clap::Parser;
use crate::{
    pkg::{
        pull_package_list, get_pkg_manifest, update_pkg_manifest,
    }, args::{
        Args, Commands
    }
};

fn main() {
    match Args::parse().command {
        Commands::Install { package } => {
            install_package(&package)
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

/// Attempt to install a package or upgrade to a newer version.
fn install_package(pkg_name: &str) {
    let pkg_list = pull_package_list();

    if !pkg_list.iter().any(|pkg| pkg.name == pkg_name) {
        println!("Could not find package by the name of '{}'.", pkg_name);
        return;
    }

    let pkg = pkg_list.iter().find(|elem| elem.name == pkg_name).unwrap().clone();
    pkg.print();

    // Check for if installed
    let mut pkg_manifest = get_pkg_manifest();
    if pkg_manifest.iter().any(|installed_pkg| installed_pkg.name == pkg.name) {
        let installed = pkg_manifest.iter().find(|elem| elem.name == pkg.name).unwrap().clone();
        if installed.upgradable_to(&pkg) {
            println!(
                "Package '{}' is already installed. However there is an upgrade available.",
                pkg_name
            );
            println!("The current version will be removed.");

            // Remove from manifest
            for i in 0..pkg_manifest.len() {
                if pkg_manifest[i].name == pkg.name {
                    pkg_manifest.remove(i);
                    break;
                }
            }

            // Delete the file
            installed.remove();
        } else {
            println!("Package '{}' version '{}' is already installed.", pkg.name, pkg.version);
            return;
        }
    }

    println!("Downloading...");
    pkg.download();

    // Update manifest
    pkg_manifest.push(pkg.clone());
    update_pkg_manifest(&pkg_manifest);
}

