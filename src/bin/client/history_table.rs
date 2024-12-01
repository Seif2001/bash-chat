use async_std::fs::File;
use async_std::prelude::*;
use serde::{Serialize, Deserialize};
use std::fs;
use std::io::{self, Write, Read};
use std::path::Path;
use std::sync::Mutex;
use std::sync::Arc;

// Define the structure of a history entry
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoryEntry {
    pub my_username: String,
    pub requester_username: String,
    pub image_name: String,
    pub is_sent: bool,
}

impl HistoryEntry {
    pub fn new(my_username: &str, requester_username: &str, image_name: &str) -> Self {
        HistoryEntry {
            my_username: my_username.to_string(),
            requester_username: requester_username.to_string(),
            image_name: image_name.to_string(),
            is_sent: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequestEntry {
    pub my_username: String,
    pub requester_username: String,
    pub image_name: String,
    pub is_approved: bool,
    pub port: u16,
}

impl RequestEntry {
    pub fn new(my_username: &str, requester_username: &str, image_name: &str,  port: u16) -> Self {
        RequestEntry {
            my_username: my_username.to_string(),
            requester_username: requester_username.to_string(),
            image_name: image_name.to_string(),
            is_approved: false,
            port: port,
        }
    }
}


const HISTORY_FILE_PATH: &str = "history_table_client.json";

pub fn add_to_history(my_username: &str, requester_username: &str, image_name: &str) -> io::Result<()> {
    let mut history = read_history_table()?;
    println!("Before new entry");
    let new_entry = HistoryEntry::new(my_username, requester_username, image_name);
    println!("After new entry");
    history.push(new_entry);
    println!("After pushing into history table");
    let _ = save_history_table(&history);
    println!("After saving into table");
    Ok(())
}

pub fn add_to_history_with_file(my_username: &str, requester_username: &str, image_name: &str, port: u16, file: &str) -> io::Result<()> {
    let mut history = read_history_table_with_file(file)?;
    println!("Before new entry");
    let new_entry = RequestEntry::new(my_username, requester_username, image_name, port);
    println!("After new entry");
    history.push(new_entry);
    println!("After pushing into history table");
    let _ = save_request_table(&history);
    println!("After saving into table");
    
    Ok(())
}


pub async fn read_and_print_unapproved_requests() {
    // Open the file current_requests.json
    let mut file = File::open("current_requests.json").await.unwrap();

    // Read the contents of the file into a string
    let mut contents = String::new();
    file.read_to_string(&mut contents).await.unwrap();

    // Parse the JSON into a vector of Request structs
    let requests: Vec<RequestEntry> = serde_json::from_str(&contents).expect("Error parsing JSON");

    // Print the image name of each request
    for request in requests {
        if request.is_approved == false {
            println!("{} requested by {}", request.image_name, request.requester_username);
        }
    }
}

pub async fn get_approved_request_by_number(number: usize) -> io::Result<(String, String, u16)> {
    // Open the file current_requests.json
    let mut file = File::open("current_requests.json").await?;

    // Read the contents of the file into a string
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;

    // Parse the JSON into a vector of Request structs
    let requests: Vec<RequestEntry> = serde_json::from_str(&contents).expect("Error parsing JSON");

    // Filter out the approved requests
    let approved_requests: Vec<RequestEntry> = requests.into_iter()
        .filter(|request| !request.is_approved)  // Keep only approved requests
        .collect();

    // Ensure that the number is within the bounds of the approved requests
    let number = number -1;
    if number < approved_requests.len() {
        
        Ok((approved_requests[number].image_name.clone(), approved_requests[number].requester_username.clone(), approved_requests[number].port))  // Return the nth approved request
    } else {
        Err(io::Error::new(io::ErrorKind::NotFound, "Request not found"))
    }
}

pub fn get_history() -> io::Result<Vec<HistoryEntry>> {
    read_history_table()
}

pub fn delete_history_entry(my_username: &str, requester_username: &str, image_name: &str) -> io::Result<()> {
    let mut history = read_history_table()?;

    history.retain(|entry| {
        !(entry.my_username == my_username &&
          entry.requester_username == requester_username &&
          entry.image_name == image_name)
    });

    save_history_table(&history)
}

pub fn read_history_table() -> io::Result<Vec<HistoryEntry>> {
    if Path::new(HISTORY_FILE_PATH).exists() {
        let mut file = fs::File::open(HISTORY_FILE_PATH)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let history: Vec<HistoryEntry> = serde_json::from_str(&content)?;
        Ok(history)
    } else {
        Ok(Vec::new())
    }
}


pub fn read_history_table_with_file(file: &str) -> io::Result<Vec<RequestEntry>> {
    if Path::new(file).exists() {
        let mut file = fs::File::open(file)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let history: Vec<RequestEntry> = serde_json::from_str(&content)?;
        Ok(history)
    } else {
        Ok(Vec::new())
    }
}

fn save_history_table(history: &[HistoryEntry]) -> io::Result<()> {
    let serialized = serde_json::to_string_pretty(history)?;
    println!("After serialized");
    let mut file = fs::File::create(HISTORY_FILE_PATH)?;
    println!("After file");
    file.write_all(serialized.as_bytes())?;
    println!("After writing into file");
    Ok(())
}

fn save_request_table(history: &[RequestEntry]) -> io::Result<()> {
    let serialized = serde_json::to_string_pretty(history)?;
    println!("After serialized");
    let mut file = fs::File::create("current_requests.json")?;
    println!("After file");
    file.write_all(serialized.as_bytes())?;
    println!("After writing into file");
    Ok(())
}


pub fn mark_as_sent(my_username: &str, requester_username: &str, image_name: &str) -> io::Result<()> {
    let mut history = read_history_table()?;
    
    for entry in history.iter_mut() {
        if entry.my_username == my_username
            && entry.requester_username == requester_username
            && entry.image_name == image_name {
            entry.is_sent = true;
            break; 
        }
    }

    save_history_table(&history)
}


pub fn get_requesters_by_image(image_name: &str) -> io::Result<Vec<String>> {
    let history = read_history_table()?;
    let requesters: Vec<String> = history.iter()
        .filter(|entry| entry.image_name == image_name)
        .map(|entry| entry.requester_username.clone())
        .collect();
    
    Ok(requesters)
}
