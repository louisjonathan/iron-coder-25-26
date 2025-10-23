use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
struct CargoConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    build: Option<BuildConfig>,
    
    #[serde(rename = "target")]
    #[serde(skip_serializing_if = "Option::is_none")]
    targets: Option<HashMap<String, TargetConfig>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    unstable: Option<UnstableConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BuildConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    target: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TargetConfig {
    #[serde(rename = "PATH")]
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    runner: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UnstableConfig {
    #[serde(rename = "build-std")]
    #[serde(skip_serializing_if = "Option::is_none")]
    build_std: Option<Vec<String>>,
}

fn update_config_toml_simple(
    project_path: &Path,
    ide_installation_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = project_path.join(".cargo").join("config.toml");
    
    // Determine toolchain path
    let toolchain_subdir = get_platform_toolchain_dir();
    let toolchain_bin = ide_installation_path
        .join("toolchain")
        .join(toolchain_subdir)
        .join("bin");
    
    let path_separator = if cfg!(windows) { ";" } else { ":" };
    let path_value = format!("{}{}$PATH", toolchain_bin.display(), path_separator);
    
    // Read existing or create new
    let mut config: CargoConfig = if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        toml::from_str(&content)?
    } else {
        CargoConfig {
            build: Some(BuildConfig {
                target: Some("avr-atmega328p.json".to_string()),
            }),
            targets: None,
            unstable: Some(UnstableConfig {
                build_std: Some(vec!["core".to_string()]),
            }),
        }
    };
    
    // Update target config
    let target_key = "cfg(target_arch = \"avr\")".to_string();
    let mut targets = config.targets.unwrap_or_default();
    
    let target_config = targets.entry(target_key).or_insert(TargetConfig {
        path: None,
        runner: Some("ravedude".to_string()),
    });
    
    target_config.path = Some(path_value);
    config.targets = Some(targets);
    
    // Write back
    fs::create_dir_all(config_path.parent().unwrap())?;
    let toml_string = toml::to_string_pretty(&config)?;
    fs::write(&config_path, toml_string)?;
    
    Ok(())
}

fn get_platform_toolchain_dir() -> &'static str {
    if cfg!(target_os = "linux") {
        "linux-x86_64"
    } else if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "macos-aarch64"
        } else {
            "macos-x86_64"
        }
    } else if cfg!(target_os = "windows") {
        "windows-x86_64"
    } else {
        panic!("Unsupported platform")
    }
}