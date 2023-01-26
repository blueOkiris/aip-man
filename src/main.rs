/*
 * Author(s): Dylan Turner
 * Description: Entry point for the App Image Package Manager
 */

mod args;
mod pkg;

use std::io::{
    stdin, stdout, Write
};
use clap::Parser;
use crate::{
    pkg::{
        pull_package_list, get_pkg_manifest, update_pkg_manifest,
    }, args::{
        Args, Commands
    }
};

fn main() {
    let args = Args::parse();
    match args.command {
        Commands::Install { package } => install_package(&package, args.ask),
        Commands::Remove { package } => remove_package(&package, args.ask),
        Commands::Upgrade => upgrade_packages(args.ask),
        Commands::List => list_packages(),
        Commands::Run { app } => run_app(&app, args.ask)
    }
}

/// Attempt to install a package or upgrade to a newer version.
fn install_package(pkg_name: &str, ask: bool) {
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

            if !prompt("Do you want to upgrade the package?", ask) {
                return;
            }

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

    if !prompt("Package found.\nDo you want to install the package?", ask) {
        return;
    }

    println!("Downloading...");
    pkg.download();

    // Update manifest
    pkg_manifest.push(pkg.clone());
    update_pkg_manifest(&pkg_manifest);
}

/// Remove a package
fn remove_package(pkg_name: &str, ask: bool) {
    let mut manifest = get_pkg_manifest();
    if !manifest.iter().any(|pkg| pkg.name == pkg_name) {
        println!("No such package '{}' installed!", pkg_name);
        return
    }
    
    if !prompt("Package found.\nAre you sure you want to remove the package?", ask) {
        return;
    }

    println!("Removing '{}'", pkg_name);
    
    let pkg = manifest.iter().find(|pkg| pkg.name == pkg_name).unwrap();
    pkg.remove();
    
    for i in 0..manifest.len() {
        if manifest[i].name == pkg.name {
            manifest.remove(i);
            break;
        }
    }
    update_pkg_manifest(&manifest);
}

/// Go through and upgrade all your installed packages.
fn upgrade_packages(ask: bool) {
    println!("Upgrading packages...");

    let mut new_manifest = Vec::new();
    let pkg_list = pull_package_list();
    let manifest = get_pkg_manifest();
    for inst_pkg in manifest.iter() {
        if pkg_list.iter().any(|pkg| pkg.name == inst_pkg.name) {
            let upstream = pkg_list.iter().find(|pkg| pkg.name == inst_pkg.name).unwrap();
            if inst_pkg.upgradable_to(upstream) {
                println!(
                    "Found upgrade for '{}:' {} -> {}",
                    inst_pkg.name, inst_pkg.version, upstream.version
                );

                if !prompt("Install?", ask) {
                    new_manifest.push(inst_pkg.clone());
                }

                new_manifest.push(upstream.clone());
                
                inst_pkg.remove();
                println!("Downloading...");
                upstream.download();

                println!("Upgraded.");
            } else {
                new_manifest.push(inst_pkg.clone());
            }
        } else {
            new_manifest.push(inst_pkg.clone());
        }
    }

    println!("Done with upgrade.");
    update_pkg_manifest(&new_manifest);
}

/// List currently installed packages
fn list_packages() {
    let manifest = get_pkg_manifest();
    for pkg in manifest {
        pkg.print();
        println!();
    }
}

/// Execute an application
fn run_app(app_name: &str, ask: bool) {
    let manifest = get_pkg_manifest();
    if !manifest.iter().any(|pkg| pkg.name == app_name) {
        println!("No such package '{}' installed!", app_name);
        return
    }

    if prompt(format!("Are you sure you want to run '{}'?", app_name).as_str(), ask) {
        manifest.iter().find(|pkg| pkg.name == app_name).unwrap().run();
    }
}

/// Get a Yes/No response from the user
fn prompt(msg: &str, ask: bool) -> bool {
    if !ask {
        return true;
    }

    let mut response = String::from("Dylan is AWESOME!");
    while response != "\n"
            && response.to_lowercase() != "y\n" && response.to_lowercase() != "n\n" {
        print!("{} [Y/n] ", msg);
        stdout().flush().expect("Failed to flush stdout.");

        response = String::new();
        let _ = stdin().read_line(&mut response).unwrap();
    }

    if response.to_lowercase() == "n\n" {
        false
    } else {
        true    
    }
}

