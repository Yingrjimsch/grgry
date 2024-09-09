use colored::Colorize;
use rayon::str::Bytes;
use reqwest::Client;
use serde::Deserialize;
use zip::ZipArchive;
use std::error::Error;
use std::fs::File;
use std::io::{self, copy, Seek};
use std::path::{Path, PathBuf};
use std::{env, fs};
use flate2::read::GzDecoder;
use tar::Archive;
use sys_info;
use tokio;

#[derive(Debug, Deserialize)]
struct Asset {
    browser_download_url: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct Release {
    assets: Vec<Asset>,
}

async fn determine_asset_pattern() -> Result<(String, String), Box<dyn Error>> {
    let os = sys_info::os_type()?;
    let arch = std::env::consts::ARCH;
    println!("{} {}", os, arch);
    let os_str = match os.as_str() {
        "Linux" => "linux-gnu",
        "Darwin" => "apple-darwin",
        "Windows" => "pc-windows-msvc",
        _ => return Err("Unsupported OS".into()),
    };
    
    let arch_str = match arch {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        _ => return Err("Unsupported architecture".into()),
    };

    // Ok((arch_str.to_string(), os_str.to_string()))
    Ok(("x86_64".to_string(), "pc-windows-msvc".to_string()))
}

pub async fn download_latest_release() -> Result<(), Box<dyn Error>> {
    let client = Client::new();

    let api_url = format!("https://api.github.com/repos/Yingrjimsch/grgry/releases/latest");
    let response = client
        .get(&api_url)
        .header("User-Agent", "grgry")
        .send()
        .await?
        .json::<Release>()
        .await?;

    let asset_pattern: (String, String) = determine_asset_pattern().await?;
    let matching_asset = response.assets.iter()
        .find(|asset| asset.name.to_lowercase().contains(&asset_pattern.0.to_lowercase()) && asset.name.to_lowercase().contains(&asset_pattern.1.to_lowercase()))
        .ok_or("No matching asset found for the current system".red())?;

    let download_url = &matching_asset.browser_download_url;
    let tmp_dir = env::temp_dir();
    
    println!("Downloading from: {}", download_url);
    let response = client
        .get(download_url)
        .header("User-Agent", "grgry")
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    extract(io::Cursor::new(response), &tmp_dir)?;
    let binary_file_name = tmp_dir.join("grgry");
    self_replace::self_replace(&binary_file_name)?;


    Ok(())
}

#[cfg(target_family = "unix")]
fn extract<R: io::Read + Seek>(reader: R, target_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let tar = GzDecoder::new(reader);
    let mut archive = Archive::new(tar);

    for entry in archive.entries()? {
        let mut entry = entry?;
        let entry_path = entry.path()?.to_path_buf();
        let file_name = entry_path.file_name().ok_or("Failed to get file name")?;
        let target_path: PathBuf = target_dir.join(file_name);

        if file_name.to_string_lossy() != "grgry" {
            continue;
        }

        println!("Unpacking into {}", target_path.display());

        let _ = entry.unpack(&target_path);
    }

    Ok(())
}

#[cfg(target_family = "windows")]
fn extract<R: io::Read + Seek>(reader: R, target_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut archive = ZipArchive::new(reader)?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let entry_path = entry.mangled_name();
        let file_name = entry_path.file_name().ok_or("Failed to get file name")?;
        let target_path: PathBuf = target_dir.join(file_name);
        
        println!("{:?}", entry_path);
        if file_name.to_string_lossy() != "grgry.exe" {
            continue;
        }

        // Construct target path
        // let target_path = target_dir.join(entry_path.strip_prefix("/")?);

        // Print paths for debugging
        println!("Unpacking {} to {:?}", entry.name(), target_path);

        // Unpack the entry
        let mut outfile = File::create(&target_path)?;
        copy(&mut entry, &mut outfile)?;
    }

    Ok(())
}