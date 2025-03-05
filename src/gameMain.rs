use macroquad::prelude::*;

struct Player {
    x: f32,
    y: f32,
    size: f32,
    speed: f32,
    speed_mod: f32,
}

impl Player {
    fn new() -> Self {
        Self {
            x: screen_width() / 2.0,
            y: screen_height() / 2.0,
            size: 50.0,
            speed: 250.0,
            speed_mod: 1_f32/((2_f32).sqrt()),
        }
    }


    
    fn update(&mut self) {
        let mut dx:f32= 0.0;
        let mut dy:f32= 0.0;
        if is_key_down(KeyCode::W) {
            dy -= 1.0;
        }
        if is_key_down(KeyCode::A) {
            dx-=1.0;
        }
        if is_key_down(KeyCode::S) {
            dy+=1.0;
        }
        if is_key_down(KeyCode::D) {
            dx+=1.0;
        }
        let mut diagonal_modifier:f32 = 1.0;
        if (dx!=0.0)&&(dy!=0.0) {
            diagonal_modifier = self.speed_mod;
        }
        let delta_time = get_frame_time();
        self.x+=dx*diagonal_modifier*self.speed*delta_time;
        self.y+=dy*diagonal_modifier*self.speed*delta_time;
    }

    fn draw(&self) {
        draw_rectangle(self.x, self.y, self.size, self.size, RED);
    }
}

#[macroquad::main("Basic 2D Game")]
async fn main() {
    let mut player = Player::new();
    
    loop {
        clear_background(BLACK);
        
        player.update();
        player.draw();
        
        next_frame().await;
    }
}
