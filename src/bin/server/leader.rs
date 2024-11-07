use tokio::sync::Mutex;
use std::sync::Arc;
use std::collections::HashMap;


pub struct Leader {
    pub id: Option<u32>,
    pub is_leader: bool,
}

pub async fn elect_leader(servers: Arc<Mutex<HashMap<u32, u32>>>)->std::io::Result<(Option<Leader>)>{
    let db = db.lock();
    let mut min_key: Option<u32> = None;
    let mut min_value: Option<u32> = None;

    for (&key, &value) in db.await.iter() {
        if min_value.is_none() || value < min_value.unwrap() {
            min_value = Some(value);
            min_key = Some(key);
        }
    }
    Leader{
        id: key,
        is_leader: true
    }
    
}