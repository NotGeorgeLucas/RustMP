use macroquad::prelude::*;

use macroquad_tiled as tiled;

use macroquad_platformer::*;
use rust_mp::player::*;
use rust_mp::game_handle::GameHandle;
use std::sync::{Arc,Mutex};

#[macroquad::main("Platformer")]
async fn main() {
    let tileset = load_texture("examples/tileset.png").await.unwrap();
    tileset.set_filter(FilterMode::Nearest);
    let run_texture = load_texture("examples/Run.png").await.unwrap();
    run_texture.set_filter(FilterMode::Nearest);
    let idle_texture = load_texture("examples/Idle.png").await.unwrap();
    idle_texture.set_filter(FilterMode::Nearest);
    let jump_texture = load_texture("examples/Jump.png").await.unwrap();
    jump_texture.set_filter(FilterMode::Nearest);
    let attack1_texture = load_texture("examples/Attack1.png").await.unwrap();
    attack1_texture.set_filter(FilterMode::Nearest);
    let attack2_texture = load_texture("examples/Attack2.png").await.unwrap();
    attack2_texture.set_filter(FilterMode::Nearest);
    // let death_texture = load_texture("examples/Death.png").await.unwrap();
    // death_texture.set_filter(FilterMode::Nearest);
    // let take_hit_texture = load_texture("examples/Take_hit.png").await.unwrap();
    // take_hit_texture.set_filter(FilterMode::Nearest);
    const FRAME_WIDTH: f32 = 512.0;
    const FRAME_HEIGHT: f32 = 512.0;
    let total_frames = 8;
    let total_idle_frames = 8;
    let total_jump_frames = 2; 
    let frame_duration = 0.1;
    //let frame_death = 7;
    let frame_attack1 = 8;
    let frame_attack2 = 8;
    // let take_hit =3;
    
    // переменные, которые меняются каждый кадр
    let mut current_frame = 0;
    let mut frame_timer = 0.0;

    let tiled_map_json = load_string("examples/map.json").await.unwrap();
    let tiled_map = tiled::load_map(&tiled_map_json, &[("tileset.png", tileset)], &[]).unwrap();
    let mut static_colliders = vec![];
    for (_x, _y, tile) in tiled_map.tiles("main layer", None) {
        static_colliders.push(if tile.is_some() {
            Tile::Solid
        } else {
            Tile::Empty
        });
    }

    let mut world = Arc::new(Mutex::new(World::new()));
    world.lock().unwrap().add_static_tiled_layer(static_colliders, 8., 8., 40, 1);

    let mut player = Player {
        collider: world.lock().unwrap().add_actor(vec2(15.0, 15.0), 16, 16,),
        speed: vec2(0., 0.),
        wrapper: DataWrapper{state: PlayerState::Idle, owner_id: 0, object_id:-1,},
    };


    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: game_main <is_server> <ip:port>");
        std::process::exit(1);
    }

    let is_server = args[1].parse::<bool>().unwrap_or(false);
    let ip_string = args[2].clone();
    
    let game_handle = if is_server {
        GameHandle::construct_server(world)
    } else {
        GameHandle::construct_client(ip_string,world)
    };


    /*let mut platform = Platform {
        collider: world.add_solid(vec2(170.0, 130.0), 32, 8),
        speed: 50.,
    };*/

    let camera = Camera2D::from_display_rect(Rect::new(0.0, 152.0, 320.0, -152.0));

    loop {
        clear_background(BLACK);

        set_camera(&camera);

        tiled_map.draw_tiles("main layer", Rect::new(0.0, 0.0, 320.0, 152.0), None);

        // draw platform
        /*{
            let pos = world.solid_pos(platform.collider);
            tiled_map.spr_ex(
                "tileset",
                Rect::new(6.0 * 8.0, 0.0, 32.0, 8.0),
                Rect::new(pos.x, pos.y, 32.0, 8.0),
            )
        }*/
        
        player.move_player(&mut world.lock().unwrap(), &mut current_frame, &mut frame_timer);
        
        {
            let pos = world.lock().unwrap().actor_pos(player.collider);
            let on_ground = world.lock().unwrap().collide_check(player.collider, pos + vec2(0., 1.));
            let moving = player.speed.x.abs() > 0.0;
            
            // Меняем состояние только если игрок НЕ в состоянии атаки
            // Это важно, чтобы анимация атаки проигрывалась полностью
            if player.wrapper.state != PlayerState::Attack1 && player.wrapper.state != PlayerState::Attack2 {
                if !on_ground {
                    player.wrapper.state = PlayerState::Jumping;
                } else if moving {
                    player.wrapper.state = PlayerState::Running;
                } else {
                    player.wrapper.state = PlayerState::Idle;
                }
            }

            frame_timer += get_frame_time();
            if frame_timer >= frame_duration {
                frame_timer = 0.0;
                match player.wrapper.state {
                    PlayerState::Running => current_frame = (current_frame + 1) % total_frames,
                    PlayerState::Idle => current_frame = (current_frame + 1) % total_idle_frames,
                    PlayerState::Jumping => current_frame = (current_frame + 1) % total_jump_frames,
                    PlayerState::Attack1 => {
                        current_frame = (current_frame + 1) % frame_attack1;
                        
                        if current_frame == 0 {
                          
                            if !on_ground {
                                player.wrapper.state = PlayerState::Jumping;
                            } else if moving {
                                player.wrapper.state = PlayerState::Running;
                            } else {
                                player.wrapper.state = PlayerState::Idle;
                            }
                        }
                    },
                    PlayerState::Attack2 => {
                        current_frame = (current_frame + 1) % frame_attack2;
                        
                        if current_frame == 0 {
                            if !on_ground {
                                player.wrapper.state = PlayerState::Jumping;
                            } else if moving {
                                player.wrapper.state = PlayerState::Running;
                            } else {
                                player.wrapper.state = PlayerState::Idle;
                            }
                        }
                    },
                    
                }
            }

            let player_width = 100.0;  
            let player_height = 100.0; 
            
           
            let (texture_to_use, frame_width) = match player.wrapper.state {
                PlayerState::Running => (&run_texture, 1042.0),
                PlayerState::Idle => (&idle_texture, 1042.0),
                PlayerState::Jumping => (&jump_texture, 1042.0), 
                PlayerState::Attack1 => (&attack1_texture, 1042.0),
                PlayerState::Attack2 => (&attack2_texture, 1042.0),
                // PlayerState::Death => (&death_texture, 1042.0),
            };
            
            // Calculate source rectangle based on the active animation
            let src_rect = Rect::new(
                current_frame as f32 * frame_width, 
                0.0,                               
                frame_width,                
                1042.0,             
            );
            
         
            let dest_rect = Rect::new(
                pos.x - (player_width - 16.0) / 2.0,  
                pos.y - player_height + 50.0,         
                player_width,
                player_height
            );

            draw_texture_ex(
                texture_to_use,
                dest_rect.x,
                dest_rect.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(dest_rect.w, dest_rect.h)),
                    source: Some(src_rect),
                    flip_x: player.speed.x < 0.0,
                    ..Default::default()
                },
            );
            
           
           // draw_rectangle_lines(pos.x, pos.y, 16.0, 16.0, 2.0, RED);
            
            // Для отладки: отображение текущего состояния и кадра
           // draw_text(&format!("State: {:?}, Frame: {}", player.wrapper.state, current_frame), 
             //   10.0, 20.0, 20.0, YELLOW);
        }




    

    



    }
}