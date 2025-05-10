use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::player::DataWrapper;

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct MotionDataContainer{
    pub x: f32,
    pub y: f32,
    pub x_speed: f32,
    pub y_speed: f32,
}

impl MotionDataContainer {
    pub fn new(x: f32, y: f32, x_speed: f32, y_speed: f32) -> Self {
        MotionDataContainer { x, y, x_speed, y_speed }
    }
}


#[derive(Serialize, Deserialize,Debug,Clone)]
pub enum ObjectType{
    StringMsg(String),
    Integer(i32),
    PlayerMap(HashMap<i32, DataWrapper>),
    Player(DataWrapper),
    MotionData(MotionDataContainer),
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
