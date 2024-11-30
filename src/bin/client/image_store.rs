use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Serialize, Deserialize)]
struct ImageData {
    image: String,
    views: u32,
}

pub fn create_json_for_images(path: &str, output_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut image_data = Vec::new();
    let valid_extensions = ["jpg", "jpeg", "png"];

    for entry in WalkDir::new(path) {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
                if valid_extensions.contains(&extension.to_lowercase().as_str()) {
                    if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                        image_data.push(ImageData {
                            image: file_name.to_string(),
                            views: 5,
                        });
                    }
                }
            }
        }
    }
    let json_data = serde_json::to_string_pretty(&image_data)?;
    let mut file = File::create(output_file)?;
    file.write_all(json_data.as_bytes())?;
    Ok(())
}



fn update_views_for_image(json_file: &str, image_name: &str, new_views: u32) {
    let file_content = match fs::read_to_string(json_file) {
        Ok(content) => content,
        Err(_) => {
            println!("Could not read the JSON file.");
            return;
        }
    };

    let mut image_data: Vec<ImageData> = match serde_json::from_str(&file_content) {
        Ok(data) => data,
        Err(_) => {
            println!("Failed to parse.");
            return;
        }
    };

    let mut found = false;
    for entry in &mut image_data {
        if entry.image == image_name {
            entry.views = new_views;
            found = true;
            break;
        }
    }

    if !found {
        println!("Image '{}' not found. No changes made.", image_name);
        return;
    }

    let updated_json = match serde_json::to_string_pretty(&image_data) {
        Ok(json) => json,
        Err(_) => {
            println!("Failed to serialize updated JSON.");
            return;
        }
    };

    if let Err(_) = fs::write(json_file, updated_json) {
        println!("Failed to write to the JSON file.");
        return;
    }

    println!("Updated views for '{}' to {}.", image_name, new_views);
}

fn add_image_to_json(json_file: &str, image_name: &str, views: u32) {
    let mut image_data: Vec<ImageData> = match fs::read_to_string(json_file) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(data) => data,
            Err(_) => {
                println!("Failed to parse JSON. Starting with a new list.");
                Vec::new()
            }
        },
        Err(_) => {
            println!("JSON file not found. Starting with a new list.");
            Vec::new()
        }
    };
    if image_data.iter().any(|entry| entry.image == image_name) {
        println!("Image '{}' already exists. No changes made.", image_name);
        return;
    }
    image_data.push(ImageData {
        image: image_name.to_string(),
        views,
    });
    let updated_json = match serde_json::to_string_pretty(&image_data) {
        Ok(json) => json,
        Err(_) => {
            println!("Failed to serialize updated JSON.");
            return;
        }
    };
    if let Err(_) = fs::write(json_file, updated_json) {
        println!("Failed to write to the JSON file.");
        return;
    }
}


pub fn display_images_data(json_file: &str) {
    let file_content = match fs::read_to_string(json_file) {
        Ok(content) => content,
        Err(_) => {
            println!("JSON file not found or could not be read.");
            return;
        }
    };
    let image_data: Vec<ImageData> = match serde_json::from_str(&file_content) {
        Ok(data) => data,
        Err(_) => {
            println!("Failed to parse JSON. The file might be corrupted.");
            return;
        }
    };
    for entry in image_data {
        println!("{}", entry.image);
    }
}

fn update_image_json(path: &str, json_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file_content = match fs::read_to_string(json_file) {
        Ok(content) => content,
        Err(_) => {
            println!("JSON file not found or could not be read.");
            return Ok(());
        }
    };
    let mut image_data: Vec<ImageData> = match serde_json::from_str(&file_content) {
        Ok(data) => data,
        Err(_) => {
            println!("Failed to parse JSON. Starting with a new list.");
            Vec::new()
        }
    };
    let mut current_images = Vec::new();
    let valid_extensions = ["jpg", "jpeg", "png"];
    for entry in WalkDir::new(path) {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
                if valid_extensions.contains(&extension.to_lowercase().as_str()) {
                    if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                        current_images.push(file_name.to_string());
                    }
                }
            }
        }
    }
    image_data.retain(|entry| current_images.contains(&entry.image));
    for image in current_images {
        if !image_data.iter().any(|entry| entry.image == image) {
            image_data.push(ImageData {
                image: image.clone(),
                views: 5, // New images start with 5 views
            });
        }
    }
    let updated_json = match serde_json::to_string_pretty(&image_data) {
        Ok(json) => json,
        Err(_) => {
            println!("Failed to serialize updated JSON.");
            return Ok(());
        }
    };

    fs::write(json_file, updated_json)?;
    Ok(())
}