# The App Image Package Manager (AIP-Man)

## Description

The goal of this project is a portable tool for managing the installing, removal, and upgrading of App Images either for usage in various Linux distros along with the potential for an AppImage-based distro.

It is called the "aip-man" because that looks like it would be pronounced "ape man" which is fitting as simplicity, or returning to ape, is an inherent part of making a portable tool like this. I also think it's funny.

## Building

Dependencies:
- (Optional) [appimaged](https://github.com/probonopd/go-appimage) to scrape your installed packages and show them in your system menus.

It's a Rust project, so run `cargo build --release`

## Usage

There are 5 commands supported by the aip-man:

- Install
  + Usage: `aipman install <package-name>`
  + The `install` command will search the global package listing for the package you typed in. It will then download and install the package unless it is already installed. If it is already installed and there is a new version, it will upgrade to the latest version.
- Remove
  + Usage: `aipman remove <package-name>`
  + The `remove` command will remove a package from your system if it is installed.
- Upgrade
  + Usage: `aipman upgrade`
  + This command pulls the latest list of packages and versions and upgrades your installed ones if available.
- List
  + Usage: `aipman list`
  + List out installed packages.
- Run
  + Usage: `aipman run <app-name>`
  + This command will run one of your installed apps, so you don't have to navigate to the install directory to launch them.

## Contributing

Please contribute! Add packages to the [global package listing](https://raw.githubusercontent.com/blueOkiris/aip-man-pkg-list/main/pkgs.json) or improve the tool itself. I'd love your help!

In order to contribute, you probably need to understand how everything works under the hood, so I will explain it.

### CLI Parsing

The aip-man makes use of a Rust package known as [clap](https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html) to handle cli parsing. This is done to make it extremely simple to maintain and add new features.

Each sub-command is handled via a `match` statement in the main function.

### Installed Packages

AppImage packages are installed to "$HOME/AppImages". This is found in Rust using the [dirs](https://docs.rs/dirs/latest/dirs/) library.

Inside this folder there will also be a manifest containing a list of each package and the installed version.

The package manager will read the manifest to know about installed packages as well as the versioning. When installing, it uses this to check if a package already exists and is up to date. It will also update the manifest after installing a new package. The same is true for the remove and upgrade commands.

The manifest is in the same JSON format as the global package list.

The aip-man uses the [serde_json](https://docs.rs/serde_json/latest/serde_json/) Rust library to parse the manifest.

### Global Package List Format

The global package list contains information on each available package.

The format for an entry in the list is as follows:

```
[
    ...
    {
        "name": "<package name>",
        "version": "<version string>",
        "description": "<description>",
        "url": "<link to file to download>"
    }, {
        "name": "audacity",
        "version": "3.2.3",
        "description": "Audacity is an easy-to-use, multi-track audio editor and recorder for Windows, macOS, GNU/Linux and other operating systems. Audacity is free, open source software.",
        "url": "https://github.com/audacity/audacity/releases/download/Audacity-3.2.3/audacity-linux-3.2.3-x64.AppImage"
    },
    ...
]
```

If you want to update a package or add a new one, simply fill out the necessary information and post a PR at the [GitHub for the list](https://github.com/blueOkiris/aip-man-pkg-list).

The aip-man uses the [reqwest](https://docs.rs/reqwest/latest/reqwest/) library to pull the info for the global package list and uses serde_json again to parse it.

### Installing to System

Obviously the idea of App Images is that they are applications. One wants their applications to show up in their app listings and see the icons.

The way this is achieved is through the use of [appimaged](https://github.com/probonopd/go-appimage) which is why it is a dependency.

This happens automatically though, so there isn't code to do it.

