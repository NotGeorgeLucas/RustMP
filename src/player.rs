
use serde::{Serialize,Deserialize};
use strum_macros::{EnumString, Display};
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
    Attack2,
    Death
    //Take_hit
}

#[derive(EnumString, Display, Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
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
    pub facing_right: bool,
}

impl DataWrapper {
    pub fn generate_motion_data(&self) -> MotionDataContainer{
        MotionDataContainer::new(
            self.position_data.0,
            self.position_data.1,
            self.speed_data.0,
            self.speed_data.1,
            self.state,
            self.facing_right,
        )
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
    pub health: i32,
    pub is_dead: bool,         // Flag to track if player is dead
    pub death_frame: usize,    // Track death animation frame
    pub animation_changed: bool,
    pub invinvibility_frames: f32,
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
                attack1_1: load_texture("assets/Attack1.png").await.unwrap(),
                attack1_2: load_texture("assets/Attack1.png").await.unwrap(),
                attack2: load_texture("assets/Attack2.png").await.unwrap(),
                death: load_texture("assets/Death.png").await.unwrap(), 
            },
            witch: PlayerTextures { 
                run: load_texture("assets/W_blue/B_run.png").await.unwrap(),
                idle: load_texture("assets/W_blue/B_idle.png").await.unwrap(),
                jump: load_texture("assets/W_blue/B_charge.png").await.unwrap(),
                attack1_1: load_texture("assets/W_blue/Attack1_W.png").await.unwrap(),
                attack1_2: load_texture("assets/W_blue/Attack1_S.png").await.unwrap(),
                attack2: load_texture("assets/W_blue/Attack2.png").await.unwrap(),
                death: load_texture("assets/W_blue/B_Death.png").await.unwrap(), 
            },
        }
    }
}

pub struct PlayerTextures {
    pub run: Texture2D,
    pub idle: Texture2D,
    pub jump: Texture2D,
    pub attack1_1: Texture2D,
    pub attack1_2: Texture2D,
    pub attack2: Texture2D,
    pub death: Texture2D,    // Add death texture field
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
    pub attack1_1: AnimationData, 
    pub attack1_2: AnimationData, 
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
    pub witch: WitchAnimations,
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
                attack1: 9, // Changed from 18 to 9 to only use attack1_1
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

    pub fn get_attack_zone(&self) -> Rect {
        let collider_pos = self.wrapper.position_data;
        let attack_width = 50.0; 
        let attack_height = 30.0; 

        if self.facing_right {
            Rect::new(collider_pos.0 + 32.0, collider_pos.1, attack_width, attack_height)
        } else {
            Rect::new(collider_pos.0 - attack_width, collider_pos.1, attack_width, attack_height)
        }
    }

    pub fn check_attack_collision(attacker: &Player, target: &Player) -> bool {
        let attack_zone = attacker.get_attack_zone();
        let target_collider = Rect::new(
            target.wrapper.position_data.0,
            target.wrapper.position_data.1,
            32.0, 
            64.0, 
        );

        attack_zone.overlaps(&target_collider)
    }
    
    pub fn take_damage(&mut self, damage: i32) {
        
        if self.is_dead || self.invinvibility_frames > 0.0 {
            return;
        }
        
        self.health -= damage;
        self.invinvibility_frames = 1.5;
        if self.health <= 0 {
            self.health = 0;
            self.is_dead = true;
            self.wrapper.state = PlayerState::Death;
            self.death_frame = 0;
            self.current_frame = 0;
            self.speed = Vec2::ZERO; 
        }
    }
    
    pub fn handle_attack(&mut self, other_players: &mut Vec<&mut Player>) {
        if self.is_dead {
            return;
        }
        
        // Only check for collision during specific frames of the attack animation
        // (typically middle frames have the best hit detection)
        if (self.wrapper.state == PlayerState::Attack1 && self.attack_frame >= 3 && self.attack_frame <= 5) || 
           (self.wrapper.state == PlayerState::Attack2 && self.attack_frame >= 4 && self.attack_frame <= 6) {
            
            let damage = if self.wrapper.state == PlayerState::Attack1 { 10 } else { 15 };
            
            for target in other_players.iter_mut() {
                if target.wrapper.object_id != self.wrapper.object_id && // Don't attack self
                   !target.is_dead && // Don't attack already dead players
                   Self::check_attack_collision(self, target)
                {
                    target.take_damage(damage);
                }
            }
        }
    }
    
    pub fn construct_from_wrapper(wrapper: DataWrapper, world: &mut World, player_size_data: &PlayerSizeData) -> Player {
        let (width, height) = match wrapper.character_type {
            CharacterType::Witcher => {
                (32, 64)
            }
            CharacterType::Witch => {
                let size = &player_size_data.witch.idle.size_frame;
                ((size.width / 10.0) as i32, (size.height / 10.0) as i32)
            }
        };
        let position = if wrapper.character_type == CharacterType::Witcher {
            vec2(wrapper.position_data.0, wrapper.position_data.1)
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
            facing_right: wrapper.facing_right,
            health: 100,
            is_dead: false,
            death_frame: 0,
            animation_changed: false,
            invinvibility_frames: 0.0,
        }
    }
    
    pub fn process_input(&mut self, world: &mut World, frame_timer: &mut f32) {
        // Don't process input if player is dead
        if self.is_dead {
            return;
        }
        
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
        } else if self.wrapper.state == PlayerState::Attack2 {
            if self.facing_right {
                self.speed.x = 100.0; 
            } else {
                self.speed.x = -100.0; 
            }
        } else {
            self.speed.x = 0.0;
        }

        if is_key_pressed(KeyCode::Space) && on_ground && 
           self.wrapper.state != PlayerState::Attack1 && 
           self.wrapper.state != PlayerState::Attack2 {
            self.speed.y = -120.0;
        }
        
        if is_key_pressed(KeyCode::F) && on_ground && 
           self.wrapper.state != PlayerState::Attack1 && 
           self.wrapper.state != PlayerState::Attack2 {
            self.wrapper.state = PlayerState::Attack1;
            self.current_frame = 0;
            *frame_timer = 0.0;
            self.attack_frame = 0;
        }

        if is_key_pressed(KeyCode::G) && on_ground && 
           self.wrapper.state != PlayerState::Attack1 && 
           self.wrapper.state != PlayerState::Attack2 {
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
        // No physics for dead players
        if self.is_dead {
            return;
        }
        
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
            self.wrapper.facing_right = self.facing_right;
            let new_vel = self.speed;
            let new_pos = world.actor_pos(self.collider);
            self.wrapper.position_data = (new_pos.x, new_pos.y);
            self.wrapper.speed_data = (new_vel.x, new_vel.y);
            if new_vel != old_vel {
                self.speed_updated = true;
            }
        }
    }

    pub fn handle(
        &mut self,
        world: &mut World,
        frame_timer: &mut f32,
        client_id: i32,
        character_type: CharacterType,
        animation_frames: &CharacterAnimationFrames,
        other_players: &mut Vec<&mut Player>
    ) {
        let prev_state = self.wrapper.state;
        self.move_player(world, frame_timer, client_id);

        self.handle_attack(other_players);
        
        self.invinvibility_frames -= get_frame_time();
        if self.invinvibility_frames < 0.0 { self.invinvibility_frames = 0.0; }

        // Handle death animation separately
        if self.is_dead {
            self.handle_death_animation(frame_timer, character_type, animation_frames);
            return;
        }
        
        if prev_state != self.wrapper.state && matches!(self.wrapper.state, PlayerState::Attack1 | PlayerState::Attack2) {
            self.animation_changed = true;
        }

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
                    self.current_frame = self.current_frame % frames.run;
                }
                PlayerState::Idle => {
                    self.current_frame = self.current_frame % frames.idle;
                }
                PlayerState::Jumping => {
                    self.current_frame = self.current_frame % frames.jumping;
                }
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
                }
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
                PlayerState::Death => {
                    // This shouldn't happen as death is handled in handle_death_animation
                    // But just in case
                    self.is_dead = true;
                }
            }
        }
    }
    
    // New method to handle death animation
    fn handle_death_animation(
        &mut self, 
        frame_timer: &mut f32,
        character_type: CharacterType,
        animation_frames: &CharacterAnimationFrames
    ) {
        *frame_timer += get_frame_time();
            
            let frames = match character_type {
                CharacterType::Witcher => &animation_frames.witcher,
                CharacterType::Witch => &animation_frames.witch,
            };
            
            // If death animation isn't complete yet
            if self.death_frame < frames.death - 1 {
                self.death_frame += 1;
                self.current_frame = self.death_frame;
            }
            // Otherwise stay on the last frame of death animation
        
    }
    

    pub fn render(
        &self, 
        textures: &CharacterTextures, 
        player_size: Vec2, 
        character_type: CharacterType, 
        world: &World, 
        player_size_data: &PlayerSizeData
    ) {
        let player_width = 100.0;  
        let player_height = 100.0; 
        let witcher_width = player_width * 1.5;  
        let witcher_height = player_height * 1.5; 
        
        // Choose texture based on state and character type
        let texture = match character_type {
            CharacterType::Witcher => match self.wrapper.state {
                PlayerState::Running => &textures.witcher.run,
                PlayerState::Idle => &textures.witcher.idle,
                PlayerState::Jumping => &textures.witcher.jump,
                PlayerState::Attack1 => &textures.witcher.attack1_1,
                PlayerState::Attack2 => &textures.witcher.attack2,
                PlayerState::Death => &textures.witcher.death,
            },
            CharacterType::Witch => match self.wrapper.state { 
                PlayerState::Running => &textures.witch.run,
                PlayerState::Idle => &textures.witch.idle,      
                PlayerState::Jumping => &textures.witch.jump,   
                PlayerState::Attack1 => &textures.witch.attack1_1, 
                PlayerState::Attack2 => &textures.witch.attack2,
                PlayerState::Death => &textures.witch.death,
            },
        };

        // Get frame size from player size data
        let frame_size = match character_type {
            CharacterType::Witcher => match self.wrapper.state {
                PlayerState::Running => &player_size_data.witcher.run.size_frame,
                PlayerState::Idle => &player_size_data.witcher.idle.size_frame,
                PlayerState::Jumping => &player_size_data.witcher.jump.size_frame,
                PlayerState::Attack1 => &player_size_data.witcher.attack1.size_frame,
                PlayerState::Attack2 => &player_size_data.witcher.attack2.size_frame,
                PlayerState::Death => &player_size_data.witcher.death.size_frame,
            },
            CharacterType::Witch => match self.wrapper.state {
                PlayerState::Running => &player_size_data.witch.run.size_frame,
                PlayerState::Idle => &player_size_data.witch.idle.size_frame,
                PlayerState::Jumping => &player_size_data.witch.jump.size_frame,
                PlayerState::Attack1 => &player_size_data.witch.attack1_1.size_frame,
                PlayerState::Attack2 => &player_size_data.witch.attack2.size_frame,
                PlayerState::Death => &player_size_data.witch.death.size_frame,
            },
        };
        
        let frame_width = frame_size.width;
        let frame_height = frame_size.height;

        // Use the current frame based on animation state
        let frame_to_draw = match self.wrapper.state {
            PlayerState::Attack1 | PlayerState::Attack2 => self.attack_frame,
            PlayerState::Death => self.death_frame,
            _ => self.current_frame,
        };

        // Calculate source rectangle based on character type
        let src_rect = match character_type {
            CharacterType::Witcher => Rect::new(
                frame_width * frame_to_draw as f32,
                0.0,
                frame_width,
                frame_height,
            ),
            CharacterType::Witch => {
                Rect::new(
                    0.0,
                    frame_height * frame_to_draw as f32,
                    frame_width,
                    frame_height,
                )
            },
        };

        let collider_pos = world.actor_pos(self.collider);
        let mut collider_size = vec2(player_size.x, player_size.y);
            
        let scale = match character_type {
            CharacterType::Witcher => 1.0,
            CharacterType::Witch => 0.8,
        };
        
        collider_size = collider_size * scale;

        if character_type == CharacterType::Witch {
            if self.wrapper.state == PlayerState::Attack1 || self.wrapper.state == PlayerState::Attack2 {
                collider_size.x *= 2.0; 
            } else if self.wrapper.state == PlayerState::Death {
                collider_size.x *= 2.0;
                collider_size.y *= 1.2; // Make death animation a bit larger
            } else {
                if self.wrapper.state == PlayerState::Jumping {
                    collider_size.x *= 2.0;
                } else {
                    collider_size.x *= 1.5;
                }
            }
        }

        // Special positioning for death animation
        let dest_rect = if self.wrapper.state == PlayerState::Death {
            if character_type == CharacterType::Witch {
                Rect::new(
                    collider_pos.x - collider_size.x * 0.5, // Center the death animation
                    collider_pos.y + 20.0, 
                    collider_size.x * 0.8, // Make death animation wider
                    collider_size.y,
                )
            } else {
                Rect::new(
                    collider_pos.x - witcher_width * 0.5,
                    collider_pos.y - 15.0, // Position the death animation a bit higher
                    witcher_width * 0.8,   // Make death animation wider
                    witcher_height
                )
            }
        } else if character_type == CharacterType::Witch {
            let x_offset = if !self.facing_right && (self.wrapper.state == PlayerState::Attack1 || self.wrapper.state == PlayerState::Attack2) {
                collider_size.x - player_size.x * scale * 1.5
            } else {
                0.0
            };
            Rect::new(
                collider_pos.x - x_offset,
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
    
        // Display health bar above the player if not dead
        if !self.is_dead {
            let health_bar_width = 50.0;
            let health_bar_height = 5.0;
            let health_percent = self.health as f32 / 100.0;
            
            // Background (empty) health bar
            draw_rectangle(
                collider_pos.x - 10.0,
                collider_pos.y - 15.0,
                health_bar_width,
                health_bar_height,
                Color::new(0.3, 0.3, 0.3, 0.8),
            );
            
            // Filled health bar
            draw_rectangle(
                collider_pos.x - 10.0,
                collider_pos.y - 15.0,
                health_bar_width * health_percent,
                health_bar_height,
                Color::new(1.0 - health_percent, health_percent, 0.0, 0.8),
            );

            //Invincibility frame indicator
            if self.invinvibility_frames > 0.0 {
                draw_rectangle_lines(
                    collider_pos.x - 10.0,
                    collider_pos.y - 15.0,
                    health_bar_width * health_percent,
                    health_bar_height,
                    1.5,
                    Color::new(163.0, 213.0, 255.0, 0.8),
                );
            }
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