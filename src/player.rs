use serde::{Serialize,Deserialize};
use macroquad::prelude::*;
use macroquad_tiled as tiled;
use macroquad_platformer::*;
use crate::network_sync::NetworkSync;


#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
enum PlayerState {
    Idle,
    Running,
    Jumping,
    Attack1,
    Attack2
    //Death
    //Take_hit
}
#[derive(Debug, PartialEq, Clone, Copy)]
enum  CharacterType {
    Withest,
    //With,
   // Withest2,
    //Mag
    
}
#[derive(Debug, Clone)]
pub struct Player {
    pub collider: Actor,
    pub speed: Vec2,
    pub state: PlayerState,
    pub  owner_id: i32,
    pub  object_id: i32,
    //character_type: CharacterType,

}

impl Player {
    pub fn move_player(&mut self, world: &mut World, current_frame: &mut usize, frame_timer: &mut f32) {
        let pos = world.actor_pos(self.collider);
        let on_ground = world.collide_check(self.collider, pos + vec2(0., 1.));
    
        if !on_ground {
            self.speed.y += 500. * get_frame_time();
        }
    
        // Обрабатываем движение, если не атакуем
        if self.state != PlayerState::Attack1 && self.state != PlayerState::Attack2 {
            if is_key_down(KeyCode::D) {
                self.speed.x = 100.0;
            } else if is_key_down(KeyCode::A) {
                self.speed.x = -100.0;
            } else {
                self.speed.x = 0.;
            }
        } else {
            
            self.speed.x = 0.0;
        }
    
       
        if is_key_pressed(KeyCode::Space) {
            if on_ground && self.state != PlayerState::Attack1 {
                self.speed.y = -120.;
            }
        }
        
       
        if is_key_pressed(KeyCode::F) && on_ground && self.state != PlayerState::Attack1 {
            self.state = PlayerState::Attack1;
            *current_frame = 0; 
            *frame_timer = 0.0; 
        } 
        if is_key_pressed(KeyCode::G) && on_ground && self.state != PlayerState::Attack2 {
            self.state = PlayerState::Attack2;
            *current_frame = 0; 
            *frame_timer = 0.0; 
        }
    
        
        world.move_h(self.collider, self.speed.x * get_frame_time());
        world.move_v(self.collider, self.speed.y * get_frame_time());
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
