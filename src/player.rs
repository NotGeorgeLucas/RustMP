use serde::{Serialize,Deserialize};
use macroquad::prelude::*;
use macroquad_platformer::*;
use crate::{message::MotionDataContainer, network_sync::NetworkSync};
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
    Witcher,
    Witch,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub struct DataWrapper {
    pub state: PlayerState,
    pub owner_id: i32,
    pub object_id: i32,
    pub character_type: CharacterType,
    pub position_data: (f32, f32),
    pub speed_data: (f32, f32),
}

impl DataWrapper {
    pub fn generate_motion_data(&self) -> MotionDataContainer{
        MotionDataContainer::new(self.position_data.0, self.position_data.1, self.speed_data.0, self.speed_data.1)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Player {
    pub collider: Actor,
    pub speed: Vec2,
    pub wrapper: DataWrapper,
    pub attack_frame: usize,
    pub speed_updated: bool,
    pub current_frame: usize,
    pub facing_right: bool,
}

pub struct CharacterTextures {
    pub witcher: PlayerTextures,
    pub witch: PlayerTextures, 
}


impl CharacterTextures {
    pub async fn load() -> Self {
        CharacterTextures {
            witcher: PlayerTextures {
                run: load_texture("assets/Run.png").await.unwrap(),
                idle: load_texture("assets/Idle.png").await.unwrap(),
                jump: load_texture("assets/Jump.png").await.unwrap(),
                attack1: load_texture("assets/Attack1.png").await.unwrap(),
                attack2: load_texture("assets/Attack2.png").await.unwrap(),
            },
            witch: PlayerTextures { 
                run: load_texture("assets/W_blue/B_run.png").await.unwrap(),
                idle: load_texture("assets/W_blue/B_idle.png").await.unwrap(),
                jump: load_texture("assets/W_blue/B_charge.png").await.unwrap(),
                attack1: load_texture("assets/W_blue/Attack1.png").await.unwrap(),
                attack2: load_texture("assets/W_blue/Attack2.png").await.unwrap(),
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

#[derive(Debug, Clone, Deserialize)]
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
    pub attack1: AnimationData,
    pub attack2: AnimationData,
    pub death: AnimationData,
    pub jump: AnimationData,
    pub idle: AnimationData,
    pub run: AnimationData,
    pub fall: AnimationData,
    pub hit_taken: AnimationData,
}


#[derive(Debug, Clone, Deserialize)]
pub struct WitchAnimations {
    pub attack1: AnimationData,
    pub attack2: AnimationData,
    pub death: AnimationData,
    pub charge: AnimationData,
    pub damage: AnimationData,
    pub idle: AnimationData,
    pub run: AnimationData,
    pub jump: AnimationData,
    pub fall: AnimationData,
    pub hit_taken: AnimationData,
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
                idle: 8,
                jumping: 2,
                attack1: 8,
                attack2: 8,
                death: 7,
                take_hit: 3,
                fall: 2,
            },
            witch: AnimationFrames {
                run: 8,
                idle: 6,
                jumping: 5,
                attack1: 9,
                attack2: 9,
                death: 12,
                take_hit: 3,
                fall: 5,
            },
        }
    }
}


pub fn load_player_size_data() -> PlayerSizeData {
    let json_str = std::fs::read_to_string("assets/player_size.json")
        .expect("Failed to read player_size.json");
    
    serde_json::from_str(&json_str)
        .expect("Failed to parse player_size.json")

}




impl Player {
    
    pub fn construct_from_wrapper(wrapper: DataWrapper, world: &mut World, player_size_data: &PlayerSizeData) -> Player {
        let (width, height) = match wrapper.character_type {
            CharacterType::Witcher => {
                (32,64)
            }
            CharacterType::Witch => {
                let size = &player_size_data.witch.idle.size_frame;
                ((size.width / 10.0) as i32, (size.height / 10.0) as i32)
            }
        };
        let position = if wrapper.character_type == CharacterType::Witcher {
        
        vec2(25.0, 25.0)
    } else {
        
        vec2(wrapper.position_data.0, wrapper.position_data.1)
    };

        Player {
            collider: world.add_actor(position, width, height),
            speed: vec2(wrapper.speed_data.0, wrapper.speed_data.1),
            wrapper,
            attack_frame: 0,
            speed_updated: false,
            current_frame: 0,
            facing_right: true,
        }
    }
    
    pub fn process_input(&mut self, world: &mut World, frame_timer: &mut f32) {
        let pos = world.actor_pos(self.collider);
        let on_ground = world.collide_check(self.collider, pos + vec2(0., 1.));

        if self.wrapper.state != PlayerState::Attack1 && self.wrapper.state != PlayerState::Attack2 {
            if is_key_down(KeyCode::D) {
                self.speed.x = 100.0;
                self.facing_right = true;
            } else if is_key_down(KeyCode::A) {
                self.speed.x = -100.0;
                self.facing_right = false;
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
            self.current_frame = 0;
            *frame_timer = 0.0;
            self.attack_frame = 0;
        }

        if is_key_pressed(KeyCode::G) && on_ground && self.wrapper.state != PlayerState::Attack2 {
            self.wrapper.state = PlayerState::Attack2;
            self.current_frame = 0;
            *frame_timer = 0.0;
            self.attack_frame = 0;
             if self.facing_right {
            self.speed.x = 100.0; 
        } else {
            self.speed.x = -100.0; 
        }
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


    pub fn move_player(&mut self, world: &mut World, frame_timer: &mut f32, client_id: i32) {
        let old_vel = self.speed;
        if client_id == self.get_owner() {
            self.process_input(world, frame_timer);
        }
        self.apply_physics(world);
        if client_id == self.get_owner() {
            let new_vel = self.speed;
            let new_pos = world.actor_pos(self.collider);
            self.wrapper.position_data = (new_pos.x,new_pos.y);
            self.wrapper.speed_data = (new_vel.x, new_vel.y);
            if new_vel!=old_vel {
                self.speed_updated = true;
            }
        }
    }

    


    pub fn handle(&mut self, world: &mut World, frame_timer: &mut f32, client_id: i32, character_type: CharacterType, animation_frames: &CharacterAnimationFrames) {
        self.move_player(world, frame_timer, client_id);
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
            self.current_frame += 1;


            let frames = match character_type {
                CharacterType::Witcher => &animation_frames.witcher,
                CharacterType::Witch => &animation_frames.witch,
            };

            match self.wrapper.state {
                PlayerState::Running => {
                    self.current_frame = (self.current_frame + 1) % frames.run;
                },
                PlayerState::Idle => {
                    self.current_frame = (self.current_frame + 1) % frames.idle;
                },
                PlayerState::Jumping => {
                    self.current_frame = (self.current_frame + 1) % frames.jumping;
                },
                PlayerState::Attack1 => {
                   
                    if self.attack_frame < frames.attack1 - 1 {
                        
                        self.attack_frame += 1;
                        self.current_frame = self.attack_frame;
                    } else {
                        
                        self.attack_frame = 0;
                        self.wrapper.state = if !on_ground {
                            PlayerState::Jumping
                        } else if moving {
                            PlayerState::Running
                        } else {
                            PlayerState::Idle
                        };
                
                        self.current_frame = 0;
                    }
                },
                PlayerState::Attack2 => {
                  
                    if self.attack_frame < frames.attack2 - 1 {
                        self.attack_frame += 1;
                        self.current_frame = self.attack_frame;
                    } else {
                        self.attack_frame = 0;
                        self.wrapper.state = if !on_ground {
                            PlayerState::Jumping
                        } else if moving {
                            PlayerState::Running
                        } else {
                            PlayerState::Idle
                        };
                        self.current_frame = 0;
                    }
                }
            }
        }
    }

    pub fn render(&self, textures: &CharacterTextures, player_size: Vec2, character_type: CharacterType, world: &World, player_size_data: &PlayerSizeData) {
        let player_width = 100.0;  
        let player_height = 100.0; 
        let witcher_width = player_width * 1.5;  
        let witcher_height = player_height * 1.5; 
        let texture = match character_type {
            CharacterType::Witcher => match self.wrapper.state {
                PlayerState::Running => &textures.witcher.run,
                PlayerState::Idle => &textures.witcher.idle,
                PlayerState::Jumping => &textures.witcher.jump,
                PlayerState::Attack1 => &textures.witcher.attack1,
                PlayerState::Attack2 => &textures.witcher.attack2,
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
            CharacterType::Witcher => match self.wrapper.state {
                PlayerState::Running => &player_size_data.witcher.run.size_frame,
                PlayerState::Idle => &player_size_data.witcher.idle.size_frame,
                PlayerState::Jumping => &player_size_data.witcher.jump.size_frame,
                PlayerState::Attack1 => &player_size_data.witcher.attack1.size_frame,
                PlayerState::Attack2 => &player_size_data.witcher.attack2.size_frame,
            },
            CharacterType::Witch => match self.wrapper.state {
                PlayerState::Running => &player_size_data.witch.run.size_frame,
                PlayerState::Idle => &player_size_data.witch.idle.size_frame,
                PlayerState::Jumping => &player_size_data.witch.jump.size_frame,
                PlayerState::Attack1 => &player_size_data.witch.attack1.size_frame,
                PlayerState::Attack2 => &player_size_data.witch.attack2.size_frame,
            },
        };
        let frame_width = frame_size.width;
        let frame_height = frame_size.height;

        let frame_to_draw = match self.wrapper.state {
            PlayerState::Attack1 | PlayerState::Attack2 => self.attack_frame,
            _ => self.current_frame,
        };

        let src_rect = match character_type{
            CharacterType::Witcher => Rect::new(
                frame_width * frame_to_draw as f32,
                0.0,
                frame_width,
                frame_height,
            ),
            CharacterType::Witch => Rect::new(
                0.0,
                frame_size.height * frame_to_draw as f32,
                frame_size.width,
                frame_size.height,
            ),
        };

        let collider_pos = world.actor_pos(self.collider);
        let mut collider_size = vec2(player_size.x, player_size.y);
        

        let scale = match character_type{
            CharacterType::Witcher => 1.0,
            CharacterType::Witch => 0.8,
        };

        collider_size = collider_size * scale;

       if character_type == CharacterType::Witch {
    if self.wrapper.state == PlayerState::Attack1 || self.wrapper.state == PlayerState::Attack2 {
        collider_size.x *= 4.0; 
    } else {
        if self.wrapper.state == PlayerState::Jumping {
        collider_size.x *= 2.0;
        }
        else{
            collider_size.x *= 1.5;
          
        }
    }
}

        let dest_rect = if character_type == CharacterType::Witch {
    Rect::new(
        collider_pos.x,
        collider_pos.y + 20.0, 
        collider_size.x,
        collider_size.y,
    )
} else {
    Rect::new(
       collider_pos.x - (witcher_width - 16.0) / 2.0,  
            collider_pos.y - 35.0,         
            witcher_width,
            witcher_height
    )
};

          /*if character_type == CharacterType::Witch {
        draw_rectangle_lines(
            collider_pos.x,
            collider_pos.y + 20.0,
            collider_size.x,
            collider_size.y,
            2.0, // Толщина линии
            RED, // Цвет линии
        );
    }*/
     if character_type == CharacterType::Witcher {
        draw_rectangle_lines(
             collider_pos.x, 
            collider_pos.y, 
            32.0, 
            64.0, 
            2.0, 
            RED
        );
    }
      
       
    
        draw_texture_ex(
            texture,
            dest_rect.x,
            dest_rect.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(dest_rect.w, dest_rect.h)),
                source: Some(src_rect),
                flip_x: !self.facing_right,
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