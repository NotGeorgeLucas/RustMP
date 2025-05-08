use serde::{Serialize,Deserialize};
use macroquad::prelude::*;
use macroquad_platformer::*;
use std::sync::{Arc,Mutex};
use crate::network_sync::NetworkSync;
use serde_json;

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

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum CharacterType {
    Withest,
    Witch,
}

#[derive(Debug, Clone, Serialize, Deserialize,Copy)]
pub struct DataWrapper {
    pub state: PlayerState,
    pub owner_id: i32,
    pub object_id: i32,
    pub character_type: CharacterType,
}

#[derive(Debug, Clone, Copy)]
pub struct Player {
    pub collider: Actor,
    pub speed: Vec2,
    pub wrapper: DataWrapper,
    pub attack_frame: usize,
}

pub struct CharacterTextures {
    pub withest: PlayerTextures,
    pub witch: PlayerTextures, 
 
  
}


impl CharacterTextures {
    pub async fn load() -> Self {
        CharacterTextures {
            withest: PlayerTextures {
                run: load_texture("examples/Run.png").await.unwrap(),
                idle: load_texture("examples/Idle.png").await.unwrap(),
                jump: load_texture("examples/Jump.png").await.unwrap(),
                attack1: load_texture("examples/Attack1.png").await.unwrap(),
                attack2: load_texture("examples/Attack2.png").await.unwrap(),
            },
            witch: PlayerTextures { 
                run: load_texture("examples/W_blue/B_run.png").await.unwrap(),
                idle: load_texture("examples/W_blue/B_idle.png").await.unwrap(),
                jump: load_texture("examples/W_blue/B_idle.png").await.unwrap(),
                attack1: load_texture("examples/W_blue/Attack.png").await.unwrap(),
                attack2: load_texture("examples/W_blue/Attack.png").await.unwrap(),
            },
        }
    }
}

pub struct PlayerTextures {
    pub run: Texture2D,
    pub idle: Texture2D,
    pub jump: Texture2D,
    pub attack1: Texture2D,
    pub attack2: Texture2D,
   
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameSize {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnimationData {
    pub size_frame: FrameSize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CharacterAnimations {
    pub Attack1: AnimationData,
    pub Attack2: AnimationData,
    pub Death: AnimationData,
    pub Jump: AnimationData,
    pub Idle: AnimationData,
    pub Run: AnimationData,
    pub Fall: AnimationData,
    pub TakeHit: AnimationData,
}
#[derive(Debug, Clone, Deserialize)]
pub struct WitchAnimations {
    pub Attack: AnimationData,
    pub Death: AnimationData,
    pub Charge: AnimationData,
    pub Damage: AnimationData,
    pub Idle: AnimationData,
    pub Run: AnimationData,
}

#[derive(Deserialize)]
pub struct PlayerSizeData {
    pub witcher: CharacterAnimations,
    pub witch: CharacterAnimations,
}
pub struct AnimationFrames {
  pub run: usize,
  pub idle: usize,
  pub jumping: usize,
    pub attack1: usize,
    pub attack2: usize,
    pub death: usize,
    pub take_hit: usize,
    pub fall: usize,  
}
pub struct CharacterAnimationFrames {
    pub witcher: AnimationFrames, // Кадры для мага
    pub witch: AnimationFrames,   // Кадры для ведьмы
}




impl CharacterAnimationFrames {
    pub fn new() -> Self {
        CharacterAnimationFrames {
            witcher: AnimationFrames {
                run: 8,
                idle: 6,
                jumping: 4,
                attack1: 10,
                attack2: 12,
                death: 7,
                take_hit: 5,
                fall: 4,
            },
            witch: AnimationFrames {
                run: 9,
                idle: 7,
                jumping: 5,
                attack1: 8,
                attack2: 9,
                death: 6,
                take_hit: 4,
                fall: 5,
            },
        }
    }
}


pub fn load_player_size_data() -> PlayerSizeData {
    let json_str = std::fs::read_to_string("examples/player_size.json")
        .expect("Failed to read player_size.json");
    
    serde_json::from_str(&json_str)
        .expect("Failed to parse player_size.json")

}




impl Player {

    pub fn construct_with_size(
        wrapper: DataWrapper,
        world: &mut World,
        player_size_data: &PlayerSizeData,
    ) -> Player {
        let (width, height) = match wrapper.character_type {
            CharacterType::Withest => {
                let size = &player_size_data.witcher.Idle.size_frame;
                ((size.width / 10.0) as i32, (size.height / 10.0) as i32)
            }
            CharacterType::Witch => {
                let size = &player_size_data.witch.Idle.size_frame;
                ((size.width / 10.0) as i32, (size.height / 10.0) as i32)
            }
        };

        Player {
            collider: world.add_actor(vec2(15.0, 15.0), width, height),
            speed: vec2(0.0, 0.0),
            wrapper,
            attack_frame: 0,
        }
    }

    pub fn construct_from_wrapper(wrapper: DataWrapper, world: &mut World) -> Player {
        Player {
            collider: world.add_actor(vec2(15.0, 15.0), 16, 16),
            speed: vec2(0.0, 0.0),
            wrapper,
            attack_frame: 0,
        }
    }
    
    pub fn process_input(&mut self, world: &mut World, current_frame: &mut usize, frame_timer: &mut f32) {
        let pos = world.actor_pos(self.collider);
        let on_ground = world.collide_check(self.collider, pos + vec2(0., 1.));

        if self.wrapper.state != PlayerState::Attack1 && self.wrapper.state != PlayerState::Attack2 {
            if is_key_down(KeyCode::D) {
                self.speed.x = 100.0;
            } else if is_key_down(KeyCode::A) {
                self.speed.x = -100.0;
            } else {
                self.speed.x = 0.0;
            }
        } else {
            self.speed.x = 0.0;
        }


        if is_key_pressed(KeyCode::Space) && on_ground && self.wrapper.state != PlayerState::Attack1 {
            self.speed.y = -120.0;
        }

        if is_key_pressed(KeyCode::F) && on_ground && self.wrapper.state != PlayerState::Attack1 {
            self.wrapper.state = PlayerState::Attack1;
            *current_frame = 0;
            *frame_timer = 0.0;
            self.attack_frame = 0;
        }

        if is_key_pressed(KeyCode::G) && on_ground && self.wrapper.state != PlayerState::Attack2 {
            self.wrapper.state = PlayerState::Attack2;
            *current_frame = 0;
            *frame_timer = 0.0;
            self.attack_frame = 0;
        }
    }


    pub fn apply_physics(&mut self, world: &mut World) {
        let dt = get_frame_time();
        let pos = world.actor_pos(self.collider);
        let on_ground = world.collide_check(self.collider, pos + vec2(0., 1.));

        if !on_ground {
            self.speed.y += 500.0 * dt;
        }

        world.move_h(self.collider, self.speed.x * dt);
        world.move_v(self.collider, self.speed.y * dt);
    }


    pub fn move_player(&mut self, world: &mut World, current_frame: &mut usize, frame_timer: &mut f32, client_id: i32) {
        if client_id == self.get_owner() {
            self.process_input(world, current_frame, frame_timer);
        }
        self.apply_physics(world);
    }

    


    pub fn handle(&mut self, world: &mut World, current_frame: &mut usize, frame_timer: &mut f32, client_id: i32, player_size_data: &PlayerSizeData, character_type: CharacterType, animation_frames: &CharacterAnimationFrames) {
        self.move_player(world, current_frame, frame_timer, client_id);
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


            let frames = match character_type {
                CharacterType::Withest => &animation_frames.witcher,
                CharacterType::Witch => &animation_frames.witch,
            };

            match self.wrapper.state {
                PlayerState::Running => {
                    *current_frame = (*current_frame + 1) % frames.run;
                },
                PlayerState::Idle => {
                    *current_frame = (*current_frame + 1) % frames.idle;
                },
                PlayerState::Jumping => {
                    *current_frame = (*current_frame + 1) % frames.jumping;
                },
                PlayerState::Attack1 => {
                   
                    if self.attack_frame < frames.attack1 - 1 {
                        
                        self.attack_frame += 1;
                        *current_frame = self.attack_frame;
                    } else {
                        
                        self.attack_frame = 0;
                        self.wrapper.state = if !on_ground {
                            PlayerState::Jumping
                        } else if moving {
                            PlayerState::Running
                        } else {
                            PlayerState::Idle
                        };
                
                        *current_frame = 0;
                    }
                },
                PlayerState::Attack2 => {
                  
                    if self.attack_frame < frames.attack2 - 1 {
                        self.attack_frame += 1;
                        *current_frame = self.attack_frame;
                    } else {
                        self.attack_frame = 0;
                        self.wrapper.state = if !on_ground {
                            PlayerState::Jumping
                        } else if moving {
                            PlayerState::Running
                        } else {
                            PlayerState::Idle
                        };
                        *current_frame = 0;
                    }
                }
            }
        }
    }

    pub fn render(&self, current_frame: usize, textures: &CharacterTextures, player_size: Vec2, frame_width: f32, character_type: CharacterType, world: &World, player_size_data: &PlayerSizeData) {
        let pos = world.actor_pos(self.collider);

        let texture = match character_type {
            CharacterType::Withest => match self.wrapper.state {
                PlayerState::Running => &textures.withest.run,
                PlayerState::Idle => &textures.withest.idle,
                PlayerState::Jumping => &textures.withest.jump,
                PlayerState::Attack1 => &textures.withest.attack1,
                PlayerState::Attack2 => &textures.withest.attack2,
            },
            CharacterType::Witch => match self.wrapper.state { 
                PlayerState::Running => &textures.witch.run,
                PlayerState::Idle => &textures.witch.idle,      
                PlayerState::Jumping => &textures.witch.jump,   
                PlayerState::Attack1 => &textures.witch.attack1, 
                PlayerState::Attack2 => &textures.witch.attack2, 
            },
        };

        let frame_size = match character_type {
            CharacterType::Withest => match self.wrapper.state {
                PlayerState::Running => &player_size_data.witcher.Run.size_frame,
                PlayerState::Idle => &player_size_data.witcher.Idle.size_frame,
                PlayerState::Jumping => &player_size_data.witcher.Jump.size_frame,
                PlayerState::Attack1 => &player_size_data.witcher.Attack1.size_frame,
                PlayerState::Attack2 => &player_size_data.witcher.Attack2.size_frame,
            },
            CharacterType::Witch => match self.wrapper.state {
                PlayerState::Running => &player_size_data.witch.Run.size_frame,
                PlayerState::Idle => &player_size_data.witch.Idle.size_frame,
                PlayerState::Jumping => &player_size_data.witch.Jump.size_frame,
                PlayerState::Attack1 => &player_size_data.witch.Attack1.size_frame,
                PlayerState::Attack2 => &player_size_data.witch.Attack2.size_frame,
            },
        };
        let frame_width = frame_size.width;
        let frame_height = frame_size.height;

        let frame_to_draw = match self.wrapper.state {
            PlayerState::Attack1 | PlayerState::Attack2 => self.attack_frame,
            _ => current_frame,
        };

        let src_rect = Rect::new(
            frame_to_draw as f32 * frame_width,
            0.0,
            frame_width,
            frame_height,
        );
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
}

impl NetworkSync for Player {
    fn get_owner(&self) -> i32 {
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