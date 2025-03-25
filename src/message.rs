use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::player::Player;

#[derive(Serialize, Deserialize,Debug,Clone)]
pub enum ObjectType{
    StringMsg(String),
    Integer(i32),
    PlayerMap(HashMap<i32, Player>)
}

#[derive(Serialize, Deserialize,Debug,Clone)]
pub struct Message {
    target: i32,
    message_map: HashMap<String, ObjectType>,
}

impl Message {
    pub fn new(target: i32, message: HashMap<String, ObjectType>) -> Result<Message, Box<dyn std::error::Error>> {
        Ok(Message {
            target,
            message_map: message,
        })
    }

    pub fn get_message_map(&self) -> &HashMap<String,ObjectType>{
        &self.message_map
    }
}
