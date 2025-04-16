use serde::{Serialize,Deserialize};
use macroquad::prelude::*;
use macroquad_tiled as tiled;
use macroquad_platformer::*;
use crate::network_sync::NetworkSync;


#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum PlayerState {
    Idle,
    Running,
    Jumping,
    Attack1,
    Attack2
    //Death
    //Take_hit
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum  CharacterType {
    Withest,
    //With,
   // Withest2,
    //Mag
    
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub struct DataWrapper{
    pub state: PlayerState,
    pub  owner_id: i32,
    pub  object_id: i32,
    //character_type: CharacterType,
}
#[derive(Debug, Clone, Copy)]
pub struct Player {
    pub collider: Actor,
    pub speed: Vec2,
    pub wrapper: DataWrapper,

}

impl Player {
    pub fn move_player(&mut self, world: &mut World, current_frame: &mut usize, frame_timer: &mut f32) {
        let pos = world.actor_pos(self.collider);
        let on_ground = world.collide_check(self.collider, pos + vec2(0., 1.));
    
        if !on_ground {
            self.speed.y += 500. * get_frame_time();
        }
    
        // Обрабатываем движение, если не атакуем
        if self.wrapper.state != PlayerState::Attack1 && self.wrapper.state != PlayerState::Attack2 {
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
            if on_ground && self.wrapper.state != PlayerState::Attack1 {
                self.speed.y = -120.;
            }
        }
        
       
        if is_key_pressed(KeyCode::F) && on_ground && self.wrapper.state != PlayerState::Attack1 {
            self.wrapper.state = PlayerState::Attack1;
            *current_frame = 0; 
            *frame_timer = 0.0; 
        } 
        if is_key_pressed(KeyCode::G) && on_ground && self.wrapper.state != PlayerState::Attack2 {
            self.wrapper.state = PlayerState::Attack2;
            *current_frame = 0; 
            *frame_timer = 0.0; 
        }
    
        
        world.move_h(self.collider, self.speed.x * get_frame_time());
        world.move_v(self.collider, self.speed.y * get_frame_time());
    }


    pub fn construct_from_wrapper(wrapper: DataWrapper, world: &mut World) -> Player{
        Player {
            collider: world.add_actor(vec2(15.0, 15.0), 16, 16,),
            speed: vec2(0., 0.),
            wrapper: wrapper
        }
    }
}

impl NetworkSync for Player{
    fn get_owner(&self) -> i32{
        self.wrapper.owner_id
    }

    fn set_owner(&mut self, owner_id: i32) {
        self.wrapper.owner_id = owner_id;
    }

    fn get_object_id(&self) -> i32 {
        self.wrapper.object_id
    }

    fn set_object_id(&mut self, object_id: i32) {
        self.wrapper.object_id = object_id;
    }
}
