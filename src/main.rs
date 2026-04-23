//! # GIMP-Directory Getter
//!
//! This helper looks for GIMP 3.x settings directories and prints their paths.
//! It supports filtering by release cycles (even/odd), by versions, installation sources (tags).

use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Represents the program's configuration options.
struct Config {
    only_versions: HashSet<String>,
    only_tags: HashSet<String>,
    ignore_versions: HashSet<String>,
    ignore_tags: HashSet<String>,
    even_only: bool,
    odd_only: bool,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Handle help and version before anything else
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return;
    }

    if args.iter().any(|arg| arg == "-v" || arg == "--version") {
        println!("gimp-dir-getter {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let config = parse_args(&args);
    let search_paths = get_search_paths();
    let mut found_paths = Vec::new();

    for (base_path, tag) in search_paths {
        if !base_path.exists() || !base_path.is_dir() {
            continue;
        }

        if let Ok(entries) = fs::read_dir(base_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(file_name) = path.file_name().and_then(|s| s.to_str()).filter(|_| path.is_dir()) {
                    // Check if the directory name represents a 3.x version
                    if file_name.starts_with("3.") && file_name.chars().all(|c| c.is_ascii_digit() || c == '.') {
                        if should_include(file_name, tag, &config) {
                            found_paths.push(path);
                        }
                    }
                }
            }
        }
    }

    // Sort paths by parent directory first, then numerically by version components
    found_paths.sort_by(|a, b| {
        let parent_a = a.parent();
        let parent_b = b.parent();
        parent_a.cmp(&parent_b).then_with(|| {
            get_version_vec(a).cmp(&get_version_vec(b))
        })
    });

    for path in &found_paths {
        println!("{}", path.display());
    }

    if found_paths.is_empty() {
        std::process::exit(1);
    }
}

/// Prints help information to standard output.
fn print_help() {
    println!("Usage:\n  gimp-dir-getter [OPTION…] ");
    println!("\nExample usage:");
    println!("  gimp-dir-getter --only=3.0,flatpak");
    println!("  gimp-dir-getter --ignore=snap --ignore=3.99");
    println!("\nOptions:");
    println!("  --even-versions           Only show even minor versions.");
    println!("  --odd-versions            Only show odd minor versions.");
    println!("  --only=<V|TAG>,<V|TAG>    Only include specific versions AND tags.");
    println!("                            Tags: default, flatpak, snap, macos, windows");
    println!("  --ignore=<V|TAG>,<V|TAG>  Exclude specific versions or tags.");
    println!("  -h, --help                Show this help message");
    println!("  -v, --version             Show program version");
}

/// Parses command line arguments into a `Config` struct.
fn parse_args(args: &[String]) -> Config {
    let mut config = Config {
        only_versions: HashSet::new(),
        only_tags: HashSet::new(),
        ignore_versions: HashSet::new(),
        ignore_tags: HashSet::new(),
        even_only: false,
        odd_only: false,
    };

    let tags_list = ["default", "flatpak", "snap", "macos", "windows"];

    for arg in args.iter().skip(1) {
        if arg == "--even-versions" {
            config.even_only = true;
        } else if arg == "--odd-versions" {
            config.odd_only = true;
        } else if let Some(val) = arg.strip_prefix("--only=") {
            for v in val.split(',') {
                if !v.is_empty() {
                    if tags_list.contains(&v) {
                        config.only_tags.insert(v.to_string());
                    } else {
                        config.only_versions.insert(v.to_string());
                    }
                }
            }
        } else if let Some(val) = arg.strip_prefix("--ignore=") {
            for v in val.split(',') {
                if !v.is_empty() {
                    if tags_list.contains(&v) {
                        config.ignore_tags.insert(v.to_string());
                    } else {
                        config.ignore_versions.insert(v.to_string());
                    }
                }
            }
        }
    }
    config
}

/// Determines platform-specific search paths for GIMP.
///
/// Returns a vector of tuples consisting of (Path, Tag).
fn get_search_paths() -> Vec<(PathBuf, &'static str)> {
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .expect("Neither HOME nor USERPROFILE environment variable is set");
    let home_path = Path::new(&home);

    let mut search_paths = Vec::new();

    #[cfg(target_os = "windows")]
    {
        if let Ok(app_data) = env::var("APPDATA") {
            search_paths.push((PathBuf::from(app_data).join("GIMP"), "windows"));
        }
    }

    if let Ok(xdg_config) = env::var("XDG_CONFIG_HOME") {
        search_paths.push((PathBuf::from(xdg_config).join("GIMP"), "default"));
    } else {
        #[cfg(not(target_os = "windows"))]
        search_paths.push((home_path.join(".config/GIMP"), "default"));
    }

    #[cfg(target_os = "macos")]
    {
        search_paths.push((home_path.join("Library/Application Support/GIMP"), "macos"));
    }

    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        search_paths.push((home_path.join(".var/app/org.gimp.GIMP/config/GIMP"), "flatpak"));
        search_paths.push((home_path.join("snap/gimp/current/.config/GIMP"), "snap"));
        search_paths.push((home_path.join("snap/gimp/common/.config/GIMP"), "snap"));
    }

    search_paths
}

/// Checks if a version/tag combination should be included based on the configuration.
fn should_include(file_name: &str, tag: &str, config: &Config) -> bool {
    let components = get_version_components(file_name);
    
    // Parity check (even/odd) on the minor version
    if components.len() >= 2 {
        let minor = components[1];
        if config.even_only && !minor.is_multiple_of(2) {
            return false;
        }
        if config.odd_only && minor.is_multiple_of(2) {
            return false;
        }
    }

    // Apply filters (AND for only_versions and only_tags)
    if !config.only_versions.is_empty() && !config.only_versions.contains(file_name) {
        return false;
    }
    if !config.only_tags.is_empty() && !config.only_tags.contains(tag) {
        return false;
    }
    
    // Ignore filters
    if config.ignore_versions.contains(file_name) || config.ignore_tags.contains(tag) {
        return false;
    }

    true
}

/// Extracts numerical components from a version string.
fn get_version_components(version_str: &str) -> Vec<u32> {
    version_str.split('.')
        .map(|s| s.parse::<u32>().unwrap_or(0))
        .collect()
}

/// Helper function for sorting: Creates a numerical vector from the directory name.
fn get_version_vec(path: &PathBuf) -> Vec<u32> {
    let ver_str = path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    get_version_components(ver_str)
}
