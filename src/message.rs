use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize,Debug)]
pub struct Message {
    target: i32,
    message_map: HashMap<String, String>,
}

impl Message {
    pub fn new(target: i32, message: HashMap<String, String>) -> Result<Message, Box<dyn std::error::Error>> {
        Ok(Message {
            target,
            message_map: message,
        })
    }

    pub fn get_message_map(&self) -> &HashMap<String,String>{
        &self.message_map
    }
}
