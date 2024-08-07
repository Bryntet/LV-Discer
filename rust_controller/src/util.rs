use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::{fs, io};

use itertools::Itertools;
use reqwest::blocking::Client;

pub fn download_image_to_file(
    url: String,
    img_location: String,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Create the file path
    let file_path = PathBuf::from(img_location);

    // Create the file
    let mut dest = File::create_new(&file_path)?;
    // Create a blocking HTTP client
    let client = Client::new();

    // Send a GET request to the URL
    let response = client.get(url.clone()).send()?;

    // Check if the request was successful
    if !response.status().is_success() {
        return Err(format!("Failed to download image: HTTP {}", response.status()).into());
    }
    dest.write_all(&response.bytes()?)?;
    webp_to_png(&file_path);
    Ok(file_path.with_extension("png"))
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

pub fn webp_to_png<P: AsRef<Path>>(dir: P) {
    use libwebp::WebPDecodeRGBA;
    let test: Vec<u8> = File::open(&dir).unwrap().bytes().try_collect().unwrap();
    let (width, height, buf) = WebPDecodeRGBA(&test).unwrap();

    lodepng::encode32_file(
        dir.as_ref().with_extension("png"),
        &buf,
        width as usize,
        height as usize,
    )
    .unwrap();
}
