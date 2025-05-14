
use macroquad::{color::WHITE, math::{vec2, Rect}, texture::{draw_texture_ex, DrawTextureParams, Texture2D}};

use crate::player::Player;

#[derive(Debug, Clone, Copy)]
pub struct Spikes {
    position_x: f32,
    position_y: f32,
    width: f32,
    height: f32,
    pub time_to_live: f32,
    owner_object_id: i32,
    damage: i32,
}

impl Spikes {

    fn get_attack_zone(&self) -> Rect {
        let collider_pos = (self.position_x, self.position_y);
        let attack_width = self.width; 
        let attack_height = self.height; 

        Rect::new(collider_pos.0, collider_pos.1, attack_width, attack_height)
    }

    fn check_attack_collision(&self, target: &Player) -> bool {
        let attack_zone = self.get_attack_zone();
        let target_collider = Rect::new(
            target.wrapper.position_data.0,
            target.wrapper.position_data.1,
            32.0, 
            64.0, 
        );

        attack_zone.overlaps(&target_collider)
    }

    pub fn new(x: f32, y: f32, width: f32, height: f32, ttl: f32, owner_object_id: i32, damage: i32) -> Self {
        Self {
            position_x: x,
            position_y: y,
            width,
            height,
            time_to_live: ttl,
            owner_object_id: owner_object_id,
            damage: damage,
        }
    }

    pub fn handle(&mut self, frame_time: f32, other_players: &mut Vec<&mut Player>) {
        self.time_to_live -= frame_time;

        for target in other_players.iter_mut(){
            if target.wrapper.object_id != self.owner_object_id && !target.is_dead && Self::check_attack_collision(self, target){

                target.take_damage(self.damage);
            
            }
        }        
    }

    pub fn render(&self, src_rect: Rect, texture: &Texture2D, facing_right: bool) {
        
        let position = vec2(self.position_x, self.position_y); // Assuming `self.position` exists

        // Calculate destination size based on source rect (can scale if needed)
        let dest_size = vec2(self.width, self.height); 

        // Draw the specific frame from the character's texture
        draw_texture_ex(
            texture,
            position.x,
            position.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(dest_size),
                source: Some(src_rect),
                flip_x: !facing_right, // or true if mirroring is needed
                ..Default::default()
            },
        );
    }
}