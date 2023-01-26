/*
 * Author(s): Dylan Turner
 * Description:
 *   Abstraction of packages, pulling them, and installing them to keep code in main simpler
 */

use std::{
    path::Path,
    fs::{
        File, create_dir_all, read_to_string, remove_file 
    }, io::{
        Write, copy
    }, process::{
        Stdio, Command
    }
};
use dirs::home_dir;
use reqwest::blocking::get;
use serde::{
    Serialize, Deserialize
};
use serde_json::{
    from_str, to_string_pretty
};
use version_compare::{
    compare, Cmp
};

const PKG_LIST_URL: &'static str =
    "https://raw.githubusercontent.com/blueOkiris/aip-man-pkg-list/main/pkgs.json";
const APP_DIR: &'static str = "Applications"; 

/// Structure used to parse JSON info from package list.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub description: String,
    pub url: String
}

impl Package {
    /// This isn't necessary due to Debug, and I could write a to_string, but I want to print this
    /// a lot considering `list` and `install` and stuff use it.
    pub fn print(&self) {
        println!("Package:");
        println!("| Name: {}", self.name);
        println!("| Description: {}", self.description);
        println!("| Version: {}", self.version);
        println!("| Url: {}", self.url);
    }

    /// Check if another package is a newer version.
    pub fn upgradable_to(&self, other: &Self) -> bool {
        if self.name == other.name
                && compare(self.clone().version, other.clone().version) == Ok(Cmp::Lt) {
            true
        } else {
            false    
        }
    }

    pub fn download(&self) {
        // First, create the /home/AppImages directory if it doesn't exist
        let mut app_dir = home_dir().expect(
            "Um. Somehow you don't have a home directory. You can't use this tool"
        );
        app_dir.push(APP_DIR);
        create_dir_all(app_dir.clone()).expect("Failed to create Application path");

        // Grab the file
        let mut pkg_file = get(self.url.clone()).expect(
            "Failed to download package"
        );
        
        // Write it
        let mut out = File::create(format!(
            "{}/{}-{}.AppImage", app_dir.as_os_str().to_str().unwrap(), self.name, self.version
        )).expect("Failed to save file");
        copy(&mut pkg_file, &mut out).expect("Failed to write package content to file");
    }

    pub fn remove(&self) {
        let mut app_dir = home_dir().expect(
            "Um. Somehow you don't have a home directory. You can't use this tool"
        );
        app_dir.push(APP_DIR);
        remove_file(format!(
            "{}/{}-{}.AppImage", app_dir.as_os_str().to_str().unwrap(), self.name, self.version
        )).expect("Failed to delete package");
    }

    pub fn run(&self, args: &Vec<String>) {
        let mut app_dir = home_dir().expect(
            "Um. Somehow you don't have a home directory. You can't use this tool"
        );
        app_dir.push(APP_DIR);
        let file_name = format!(
            "{}/{}-{}.AppImage", app_dir.as_os_str().to_str().unwrap(), self.name, self.version
        );

        Command::new(file_name).args(args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output().expect("Failed to start app");
    }
}

/// Pull the package list and parse it into our abstraction.
///
/// Note that this is used to hide complexity from the top level functions, so it cannot return a
/// result/error. All errors must be handled here.
pub fn pull_package_list() -> Vec<Package> {
    let list_json = get(PKG_LIST_URL).expect("Failed to download package list")
        .text().expect("Failed to get package list text");
    from_str(list_json.as_str()).expect("Failed to parse global package list")
}

/// Read (or create) the installed package manifest
pub fn get_pkg_manifest() -> Vec<Package> {
    // First, create the /home/AppImages directory if it doesn't exist
    let mut app_dir = home_dir().expect(
        "Um. Somehow you don't have a home directory. You can't use this tool"
    );
    app_dir.push(APP_DIR);
    create_dir_all(app_dir.clone()).expect("Failed to create Application path");

    let file_name = format!("{}/aip_man_pkg_list.json", app_dir.as_os_str().to_str().unwrap());

    // Create the manifest if it doesn't exist
    if !Path::new(&file_name).exists() {
        println!("Local manifest does not exist. Creating...");
        let mut output = File::create(file_name.clone()).expect("Failed to create manifest");
        write!(output, "[\n]").expect("Failed to write to new manifest file");
    }

    let manifest_text = read_to_string(file_name).expect("Failed to read manifest");
    from_str(&manifest_text).expect("Failed to parse manifest file")
}

/// Overwrite the manifest with new data
pub fn update_pkg_manifest(manifest: &Vec<Package>) {
    // First, create the /home/AppImages directory if it doesn't exist
    let mut app_dir = home_dir().expect(
        "Um. Somehow you don't have a home directory. You can't use this tool"
    );
    app_dir.push(APP_DIR);
    create_dir_all(app_dir.clone()).expect("Failed to create Application path");

    let file_name = format!("{}/aip_man_pkg_list.json", app_dir.as_os_str().to_str().unwrap());

    // Create the manifest if it doesn't exist
    println!("Updating manifest...");
    let manifest_json = to_string_pretty(manifest).expect("Failed to format new manifest");
    let mut output = File::create(file_name.clone()).expect("Failed to open manifest for writing");
    write!(output, "{}", manifest_json).expect("Failed to save manifest");
}

