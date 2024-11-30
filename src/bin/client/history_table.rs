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

const HISTORY_FILE_PATH: &str = "history_table_client.json";

pub fn add_to_history(my_username: &str, requester_username: &str, image_name: &str) -> io::Result<()> {
    let mut history = read_history_table()?;
    let new_entry = HistoryEntry::new(my_username, requester_username, image_name);
    history.push(new_entry);

    save_history_table(&history)
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

fn read_history_table() -> io::Result<Vec<HistoryEntry>> {
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

fn save_history_table(history: &[HistoryEntry]) -> io::Result<()> {
    let serialized = serde_json::to_string_pretty(history)?;
    let mut file = fs::File::create(HISTORY_FILE_PATH)?;
    file.write_all(serialized.as_bytes())?;
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

