use serde::{Serialize,Deserialize};
use macroquad::prelude::*;
use macroquad_platformer::*;
use std::sync::{Arc,Mutex};
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
    pub owner_id: i32,
    pub object_id: i32,
    //character_type: CharacterType,
}
#[derive(Debug, Clone, Copy)]
pub struct Player {
    pub collider: Actor,
    pub speed: Vec2,
    pub wrapper: DataWrapper,
}


pub struct PlayerTextures<'a> {
    pub run: &'a Texture2D,
    pub idle: &'a Texture2D,
    pub jump: &'a Texture2D,
    pub attack1: &'a Texture2D,
    pub attack2: &'a Texture2D,
    pub world: &'a Arc<Mutex<World>>,
}




impl Player {
    pub fn move_player(&mut self, world: &mut World, current_frame: &mut usize, frame_timer: &mut f32) {
        let pos = world.actor_pos(self.collider);
        let on_ground = world.collide_check(self.collider, pos + vec2(0., 1.));

        if !on_ground {
            self.speed.y += 500. * get_frame_time();
        }

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

        if is_key_pressed(KeyCode::Space) && on_ground && self.wrapper.state != PlayerState::Attack1 {
            self.speed.y = -120.;
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

    pub fn handle(&mut self, world: &mut World, current_frame: &mut usize, frame_timer: &mut f32) {
        self.move_player(world, current_frame, frame_timer);

        let pos = world.actor_pos(self.collider);
        let on_ground = world.collide_check(self.collider, pos + vec2(0., 1.));
        let moving = self.speed.x.abs() > 0.0;

        if self.wrapper.state != PlayerState::Attack1 && self.wrapper.state != PlayerState::Attack2 {
            self.wrapper.state = if !on_ground {
                PlayerState::Jumping
            } else if moving {
                PlayerState::Running
            } else {
                PlayerState::Idle
            };
        }

        *frame_timer += get_frame_time();
        if *frame_timer >= 0.1 {
            *frame_timer = 0.0;
            *current_frame += 1;

            match self.wrapper.state {
                PlayerState::Running => *current_frame %= 8,
                PlayerState::Idle => *current_frame %= 8,
                PlayerState::Jumping => *current_frame %= 2,
                PlayerState::Attack1 => {
                    *current_frame %= 8;
                    if *current_frame == 0 {
                        self.wrapper.state = if !on_ground {
                            PlayerState::Jumping
                        } else if moving {
                            PlayerState::Running
                        } else {
                            PlayerState::Idle
                        };
                    }
                }
                PlayerState::Attack2 => {
                    *current_frame %= 8;
                    if *current_frame == 0 {
                        self.wrapper.state = if !on_ground {
                            PlayerState::Jumping
                        } else if moving {
                            PlayerState::Running
                        } else {
                            PlayerState::Idle
                        };
                    }
                }
            }
        }
    }

    pub fn render(&self, current_frame: usize, textures: &PlayerTextures, player_size: Vec2, frame_width: f32) {
        let pos = textures.world.lock().unwrap().actor_pos(self.collider);

        let texture = match self.wrapper.state {
            PlayerState::Running => &textures.run,
            PlayerState::Idle => &textures.idle,
            PlayerState::Jumping => &textures.jump,
            PlayerState::Attack1 => &textures.attack1,
            PlayerState::Attack2 => &textures.attack2,
        };

        let src_rect = Rect::new(current_frame as f32 * frame_width, 0.0, frame_width, 1042.0);
        let dest_rect = Rect::new(
            pos.x - (player_size.x - 16.0) / 2.0,
            pos.y - player_size.y + 50.0,
            player_size.x,
            player_size.y,
        );

        draw_texture_ex(
            texture,
            dest_rect.x,
            dest_rect.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(dest_rect.w, dest_rect.h)),
                source: Some(src_rect),
                flip_x: self.speed.x < 0.0,
                ..Default::default()
            },
        );
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
