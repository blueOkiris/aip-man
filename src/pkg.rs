/*
 * Author(s): Dylan Turner
 * Description:
 *   Abstraction of packages, pulling them, and installing them to keep code in main simpler
 */

use std::{
    path::Path,
    fs::{
        File, create_dir_all, remove_file, Permissions, read, read_to_string, rename, remove_dir_all
    }, io::{
        Write, copy, BufReader, Cursor
    }, process::{
        Stdio, Command
    }, os::unix::fs::PermissionsExt
};
use dirs::home_dir;
use flate2::read::GzDecoder;
use glob::glob;
use reqwest::blocking::get;
use serde::{
    Serialize, Deserialize
};
use serde_json::{
    from_str, to_string_pretty, from_reader
};
use tar::Archive;
use version_compare::{
    compare, Cmp
};
use zip_extract::extract;

const PKG_LIST_URL: &'static str =
    "https://raw.githubusercontent.com/blueOkiris/aip-man-pkg-list/main/pkgs.json";
pub const APP_DIR: &'static str = "Applications"; 
pub const PERMISSION: u32 = 0o755; // -rwxr-xr-x.

/// Structure used to parse JSON info from package list.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub description: String,
    pub url: String,
    pub compressed: Option<bool>
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
        let mut app_dir = home_dir()
            .expect("Um. Somehow you don't have a home directory. You can't use this tool");
        app_dir.push(APP_DIR);
        create_dir_all(app_dir.clone()).expect("Failed to create Application path");

        // Grab the file
        let mut pkg_file = get(self.url.clone()).expect(
            "Failed to download package"
        );

        // Write it
        let app_image_path = format!(
            "{}/{}-{}.AppImage", app_dir.as_os_str().to_str().unwrap(), self.name, self.version
        );
        let mut out = File::create(app_image_path.clone()).expect("Failed to save file");
        copy(&mut pkg_file, &mut out).expect("Failed to write package content to file");

        // If it's compressed, extract it
        if self.compressed.is_some() && self.compressed.unwrap() {
            println!("AppImage is within archive. Extracting...");

            // Extract file to ~/Applications/tmp-<name>
            let tmp_dir = format!("{}/tmp-{}", app_dir.as_os_str().to_str().unwrap(), self.name);
            if self.url.ends_with(".zip") {
                let file_contents = read(app_image_path.clone()).expect("Failed to read zip contents");
                extract(Cursor::new(file_contents), &Path::new(&tmp_dir), true)
                    .expect("Failed to extract zip.");
            } else if self.url.ends_with(".gz") {
                let tar_file = File::open(app_image_path.clone())
                    .expect("Failed to open archive.");
                let tar = GzDecoder::new(tar_file);
                let mut archive = Archive::new(tar);
                archive.unpack(tmp_dir.clone()).expect("Failed to unpack tar archive.");        
            }

            println!("Removing archive...");
            remove_file(app_image_path.clone()).expect("Failed to delete old archive.");

            // Move the underlying AppImage into place
            let entries = glob(format!("{}/*.AppImage", tmp_dir).as_str())
                .expect("Failed to find AppImage in archive.");
            for entry in entries {
                match entry {
                    Ok(path) => {
                        println!("Setting executable flag...");
                        let app_image_file = File::open(path.clone())
                            .expect("Failed to set executable.");
                        app_image_file.set_permissions(Permissions::from_mode(PERMISSION))
                            .expect("Failed to set package permissions.");

                        println!(
                            "Moving {} to {}",
                            path.as_os_str().to_str().unwrap(),
                            app_image_path.clone()
                        );
                        rename(path, Path::new(&app_image_path.clone()))
                            .expect("Failed to move AppImage into proper location.");
                        break;
                    },
                    Err(_) => {}
                }
            }
            
            match remove_dir_all(tmp_dir.clone()) {
                Ok(_) => {},
                Err(_) => println!("Failed to remove {}. Manual intervention necessary.", tmp_dir)
            }
        } else {
            // Set executable flag
            println!("Setting executable flag...");
            let app_image_file = File::create(app_image_path).expect("Failed to set executable.");
            app_image_file.set_permissions(Permissions::from_mode(PERMISSION))
                .expect("Failed to set package permissions.");
        }
    }

    pub fn remove(&self) {
        let mut app_dir = home_dir()
            .expect("Um. Somehow you don't have a home directory. You can't use this tool");
        app_dir.push(APP_DIR);
        match remove_file(format!(
            "{}/{}-{}.AppImage", app_dir.as_os_str().to_str().unwrap(), self.name, self.version
            )) {
            Ok(_) => {},
            Err(_) =>
                println!("Warning: Failed to remove file. Manual intervention may be required")
        }
    }

    pub fn run(&self, args: &Vec<String>) {
        let mut app_dir = home_dir()
            .expect("Um. Somehow you don't have a home directory. You can't use this tool");
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
pub fn pull_package_list(repo: &Option<String>) -> Vec<Package> {
    let url = repo.clone().unwrap_or(PKG_LIST_URL.to_string());
    if url.as_str().starts_with("file://") {
        // Local instead
        let file = File::open(url.split_at(7).1)
            .expect(&format!("Failed to open local repo '{}'", url));
        let reader = BufReader::new(file);
        from_reader(reader).expect("Failed to parse local package list.")
    } else {
        let list_json = get(url).expect("Failed to download package list")
            .text().expect("Failed to get package list text");
        from_str(list_json.as_str()).expect("Failed to parse package list.")
    }
}

/// Read (or create) the installed package manifest
pub fn get_pkg_manifest() -> Vec<Package> {
    // First, create the /home/AppImages directory if it doesn't exist
    let mut app_dir = home_dir()
        .expect("Um. Somehow you don't have a home directory. You can't use this tool");
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
    let mut app_dir = home_dir()
        .expect("Um. Somehow you don't have a home directory. You can't use this tool");
    app_dir.push(APP_DIR);
    create_dir_all(app_dir.clone()).expect("Failed to create Application path");

    let file_name = format!("{}/aip_man_pkg_list.json", app_dir.as_os_str().to_str().unwrap());

    // Create the manifest if it doesn't exist
    println!("Updating manifest...");
    let manifest_json = to_string_pretty(manifest).expect("Failed to format new manifest");
    let mut output = File::create(file_name.clone()).expect("Failed to open manifest for writing");
    write!(output, "{}", manifest_json).expect("Failed to save manifest");
}

