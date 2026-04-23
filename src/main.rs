use std::path::{Path, PathBuf};
use std::env;
use std::fs;
use std::collections::HashSet;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Handle help and version before anything else
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("Usage:\n  gimp-dir-getter [OPTION…] ");
        println!("\nExample usage:");
        println!("  gimp-dir-getter --only=3.0,flatpak");
        println!("  gimp-dir-getter --ignore=macos --ignore=3.99");
        println!("\nOptions:");
        println!("  --even-versions           Only show even minor versions.");
        println!("  --odd-versions            Only show odd minor versions.");
        println!("  --only=<V|TAG>,<V|TAG>    Only include specific versions AND tags.");
        println!("                            Tags: default, flatpak, snap, macos, windows");
        println!("  --ignore=<V|TAG>,<V|TAG>  Exclude specific versions or tags.");
        println!("  -h, --help                Show this help message");
        println!("  -v, --version             Show program version");
        return;
    }

    if args.iter().any(|arg| arg == "-v" || arg == "--version") {
        println!("gimp-dir-getter {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let mut only_versions = HashSet::new();
    let mut only_tags = HashSet::new();
    let mut ignore_versions = HashSet::new();
    let mut ignore_tags = HashSet::new();
    let mut even_only = false;
    let mut odd_only = false;

    let tags_list = ["default", "flatpak", "snap", "macos", "windows"];

    // Parse arguments
    for arg in args.iter().skip(1) {
        if arg == "--even-versions" {
            even_only = true;
        } else if arg == "--odd-versions" {
            odd_only = true;
        } else if let Some(val) = arg.strip_prefix("--only=") {
            for v in val.split(',') {
                if !v.is_empty() {
                    if tags_list.contains(&v) {
                        only_tags.insert(v.to_string());
                    } else {
                        only_versions.insert(v.to_string());
                    }
                }
            }
        } else if let Some(val) = arg.strip_prefix("--ignore=") {
            for v in val.split(',') {
                if !v.is_empty() {
                    if tags_list.contains(&v) {
                        ignore_tags.insert(v.to_string());
                    } else {
                        ignore_versions.insert(v.to_string());
                    }
                }
            }
        }
    }

    // Get the user's home directory from the environment variable
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .expect("Neither HOME nor USERPROFILE environment variable is set");
    let home_path = Path::new(&home);

    // Typical paths to check for GIMP configurations (Path, Tag)
    let mut search_paths = Vec::new();

    // Windows specific path
    #[cfg(target_os = "windows")]
    {
        if let Ok(app_data) = env::var("APPDATA") {
            search_paths.push((PathBuf::from(app_data).join("GIMP"), "windows"));
        }
    }

    // Check XDG_CONFIG_HOME first
    if let Ok(xdg_config) = env::var("XDG_CONFIG_HOME") {
        search_paths.push((PathBuf::from(xdg_config).join("GIMP"), "default"));
    } else {
        // XDG default path (Linux/Unix)
        #[cfg(not(target_os = "windows"))]
        search_paths.push((home_path.join(".config/GIMP"), "default"));
    }

    #[cfg(target_os = "macos")]
    {
        // macOS default path
        search_paths.push((home_path.join("Library/Application Support/GIMP"), "macos"));
    }

    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        // Flatpak path
        search_paths.push((home_path.join(".var/app/org.gimp.GIMP/config/GIMP"), "flatpak"));
        // Snap paths
        search_paths.push((home_path.join("snap/gimp/current/.config/GIMP"), "snap"));
        search_paths.push((home_path.join("snap/gimp/common/.config/GIMP"), "snap"));
    }

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
                        // Extract components for parity check
                        let components: Vec<u32> = file_name.split('.')
                            .map(|s| s.parse::<u32>().unwrap_or(0))
                            .collect();
                        
                        // Parity check (even/odd) on the minor version
                        if components.len() >= 2 {
                            let minor = components[1];
                            if even_only && !minor.is_multiple_of(2) {
                                continue;
                            }
                            if odd_only && minor.is_multiple_of(2) {
                                continue;
                            }
                        }

                        // Apply filters
                        // 1. Only Version filter (AND)
                        if !only_versions.is_empty() && !only_versions.contains(file_name) {
                            continue;
                        }
                        // 2. Only Tag filter (AND)
                        if !only_tags.is_empty() && !only_tags.contains(tag) {
                            continue;
                        }
                        // 3. Ignore Version filter
                        if ignore_versions.contains(file_name) {
                            continue;
                        }
                        // 4. Ignore Tag filter
                        if ignore_tags.contains(tag) {
                            continue;
                        }
                        
                        found_paths.push(path);
                    }
                }
            }
        }
    }

    // Sort paths by parent directory first, then numerically by version components
    found_paths.sort_by(|a, b| {
        let get_version = |p: &PathBuf| {
            p.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .split('.')
                .map(|s| s.parse::<u32>().unwrap_or(0))
                .collect::<Vec<u32>>()
        };
        
        // Compare parent directories first
        let parent_a = a.parent();
        let parent_b = b.parent();
        
        parent_a.cmp(&parent_b).then_with(|| get_version(a).cmp(&get_version(b)))
    });

    for path in &found_paths {
        println!("{}", path.display());
    }

    if found_paths.is_empty() {
        std::process::exit(1);
    }
}
