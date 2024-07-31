use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, io};

use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;

pub fn download_image_to_file(
    url: String,
    player_id: String,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Create a blocking HTTP client
    let client = Client::new();

    // Send a GET request to the URL
    let response = client.get(url.clone()).send()?;

    // Check if the request was successful
    if !response.status().is_success() {
        return Err(format!("Failed to download image: HTTP {}", response.status()).into());
    }

    // Determine the file name and extension
    let file_name = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|ct| ct.to_str().ok())
        .and_then(|ct| ct.split('/').nth(1))
        .map(|ext| format!("{player_id}.{}", ext))
        .or_else(|| {
            Path::new(&url)
                .file_name()
                .and_then(|os_str| os_str.to_str())
                .map(String::from)
        })
        .unwrap_or_else(|| "downloaded_image.bin".to_string());

    // Create the file path
    let file_path = PathBuf::from(dbg!(format!("images/{file_name}")));

    // Create the file
    let mut dest = File::create(&file_path)?;
    dest.write_all(&response.bytes()?)?;
    Ok(file_path)
}
pub fn delete_files_in_directory<P: AsRef<Path>>(dir: P) -> io::Result<()> {
    // Read the directory contents
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            // If it's a file, delete it
            fs::remove_file(&path)?;
        } else if path.is_dir() {
            // If it's a directory, recursively delete files in it
            delete_files_in_directory(&path)?;
        }
    }
    Ok(())
}
