use serde::{Serialize,Deserialize};
use bevy::prelude::*;
use bevy::input::ButtonInput;
use bevy::time::Time;
use bevy::ecs::component::Component;
use crate::network_sync::NetworkSync;


#[derive(Component,Clone,Copy,Debug,Serialize,Deserialize)]
pub struct Player {
    speed: f32,
    owner_id: i32,
    object_id: i32,
}
impl Player {
    pub fn move_player(&self, keyboard_input: &Res<ButtonInput<KeyCode>>, time: &Res<Time>, transform: &mut Transform) {
        let mut direction = Vec3::ZERO;
        
        if keyboard_input.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }

        transform.translation += direction.normalize_or_zero() * self.speed * time.delta_secs();
    }

    pub fn new(speed:f32, player_id: i32) -> Self{
        Player { speed: speed, owner_id: player_id, object_id: -1 }
    }
}

impl NetworkSync for Player{
    fn get_owner(&self) -> i32{
        self.owner_id
    }

    fn set_owner(&mut self, owner_id: i32) {
        self.owner_id = owner_id;
    }

    fn get_object_id(&self) -> i32 {
        self.object_id
    }

    fn set_object_id(&mut self, object_id: i32) {
        self.object_id = object_id;
    }
}
