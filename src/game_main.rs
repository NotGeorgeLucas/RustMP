use macroquad::prelude::*;
use macroquad_tiled as tiled;
use macroquad_platformer::*;
use rust_mp::message::{ObjectType, RpcCallContainer};
use rust_mp::{player::*, PLAYER_SIZE_DATA};
use rust_mp::game_handle::GameHandle;
use std::sync::{Arc, Mutex};
use std::str::FromStr;


#[macroquad::main("Platformer")]
async fn main() {
    // Print the current directory to debug path issues
    println!("Current directory: {:?}", std::env::current_dir().unwrap());
    
    // Load the tileset with error handling
    let tileset = match load_texture("assets/tileset.png").await {
        Ok(texture) => {
            println!("Successfully loaded tileset");
            texture
        },
        Err(err) => {
            eprintln!("Failed to load tileset: {:?}", err);
            panic!("Could not load required asset");
        }
    };
    tileset.set_filter(FilterMode::Nearest);

    let character_textures = CharacterTextures::load().await;
    let player_size_data = &*PLAYER_SIZE_DATA;
    let animation_frames = CharacterAnimationFrames::new();
   
    // Load the map json with error handling
    let tiled_map_json = match load_string("assets/map.json").await {
        Ok(json) => {
            println!("Successfully loaded map.json");
            json
        },
        Err(err) => {
            eprintln!("Failed to load map.json: {:?}", err);
            panic!("Could not load required asset");
        }
    };
    
    // Try alternative paths for the background image
    let background_texture = match load_texture("assets/Elements/Loc1.png").await {
        Ok(texture) => {
            println!("Successfully loaded background from assets/Elements/Loc1.png");
            texture
        },
        Err(_) => match load_texture("Elements/Loc1.png").await {
            Ok(texture) => {
                println!("Successfully loaded background from Elements/Loc1.png");
                texture
            },
            Err(_) => match load_texture("Loc1.png").await {
                Ok(texture) => {
                    println!("Successfully loaded background from Loc1.png");
                    texture
                },
                Err(err) => {
                    eprintln!("Failed to load background image: {:?}", err);
                    // Use a placeholder texture if all attempts fail
                    println!("Using placeholder background");
                    Texture2D::from_rgba8(1, 1, &[0, 0, 0, 255])
                }
            }
        }
    };
    
    // Load the map with error handling
    let tiled_map = match tiled::load_map(
        &tiled_map_json,
        &[("tileset.png", tileset), ("assets/Elements/Loc1.png", background_texture.clone())],
        &[]
    ) {
        Ok(map) => {
            println!("Successfully loaded tiled map");
            map
        },
        Err(err) => {
            eprintln!("Failed to load tiled map: {:?}", err);
            panic!("Could not load required map data");
        }
    };
    
    let mut static_colliders = vec![];
    for (_x, _y, tile) in tiled_map.tiles("main layer", None) {
        static_colliders.push(if tile.is_some() {
            Tile::Solid
        } else {
            Tile::Empty
        });
    }
    println!("Created {} static colliders", static_colliders.len());

    let world = Arc::new(Mutex::new(World::new()));
    world.lock().unwrap().add_static_tiled_layer(static_colliders, 8., 8., 40, 1);
    
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: game_main <is_server> <ip:port> <character_type>");
        std::process::exit(1);
    }

    let is_server = args[1].parse::<bool>().unwrap_or(false);
    let ip_string = args[2].clone();
    let character_type_str = args[3].clone();

    let character_type = CharacterType::from_str(character_type_str.as_str()).unwrap();

    let game_handle = if is_server {
        GameHandle::construct_server(Arc::clone(&world))
    } else {
        GameHandle::construct_client(ip_string, Arc::clone(&world))
    };
    
    {
        let player = Player::construct_from_wrapper(
            DataWrapper {
                state: PlayerState::Idle,
                owner_id: 0,
                object_id: -1,
                character_type: character_type,
                position_data: (15.0, 15.0),
                speed_data: (0.0, 0.0),
                facing_right: true,
                
                
            },
            &mut world.lock().unwrap(),
            &player_size_data,
        );
        game_handle.lock().unwrap().add_player(player);
        println!("Added initial player");
    }

    let mut frame_timer = 0.0;
    let camera = Camera2D::from_display_rect(Rect::new(0.0, 152.0, 320.0, -152.0));

    println!("Entering game loop");
   
    
    let mut test_counter: i32 = 0;
    loop {
        clear_background(BLACK);
        let texture_size = vec2(background_texture.width(),background_texture.height());
        let screen_size = vec2(screen_width(), screen_height());
        let screen_aspect = screen_size.x / screen_size.y;
        let texture_asppect = texture_size.x / texture_size.y;
        let draw_size;
            if screen_aspect > texture_asppect {
                let scale = screen_size.y / texture_size.y;
                draw_size = texture_size * scale; 
            } else {
                let scale = screen_size.x / texture_size.x;
                draw_size = texture_size * scale;
            }
        let draw_pos = (screen_size - draw_size) / 2.0;

            draw_texture_ex(
            &background_texture,
            draw_pos.x, 
            draw_pos.y,
            WHITE,  
                DrawTextureParams {
                dest_size: Some (draw_size),
                ..Default::default()
                },
            );
        set_camera(&camera);

        tiled_map.draw_tiles("main layer", Rect::new(0.0, 0.0, 320.0, 152.0), None);

        let mut game_handle_lock = game_handle.lock().unwrap();
        let wrapper_map_mutex = game_handle_lock.get_player_wrapper_map();
        let mut wrapper_map = wrapper_map_mutex.lock().unwrap();

        let mut player_data: Vec<(i32, *mut Player)> = wrapper_map
            .iter_mut()
            .map(|(idx, player)| (*idx, player as *mut Player))
            .collect();

        for (player_index, player_ptr) in player_data.iter() {
            // Create a vector of pointers to other players
            let mut other_players_ptrs: Vec<*mut Player> = player_data
                .iter()
                .filter_map(|(idx, ptr)| {
                    if idx != player_index {
                        Some(*ptr)
                    } else {
                        None
                    }
                })
                .collect();
            
            // Convert pointers to mutable references
            // SAFETY: This is safe because:
            // 1. We ensure we're not mutably referencing the same player twice
            // 2. player_ptr points to a valid Player object from wrapper_map
            let player = unsafe { &mut **player_ptr };
            
            // Create vector of mutable references to other players
            // SAFETY: Each pointer points to a distinct Player object
            let mut other_players: Vec<&mut Player> = other_players_ptrs
                .iter_mut()
                .map(|ptr| unsafe { &mut **ptr })
                .collect();
            
            player.handle(
                &mut world.lock().unwrap(),
                &mut frame_timer,
                game_handle_lock.get_personal_id(),
                player.wrapper.character_type,
                &animation_frames,
                &mut other_players,
            );
            
            if player.speed_updated {
                player.speed_updated = false;
                game_handle_lock.send_motion_update(*player_index, player.wrapper.generate_motion_data());
            }
            
            let character_type = player.wrapper.character_type;
            let frame_size = match character_type {
                CharacterType::Witcher => &player_size_data.witcher.idle.size_frame,
                CharacterType::Witch => &player_size_data.witch.idle.size_frame,
            };
            let player_size = vec2(frame_size.width / 10.0, frame_size.height / 10.0);
            
            player.render(
                &character_textures,
                player_size,
                character_type,
                &world.lock().unwrap(),
                &player_size_data,
            );
        }


        if test_counter >= 500 {
            game_handle_lock.send_rpc(RpcCallContainer{
                function_name: "test_rpc_no_param".to_string(),
                params: vec![]
            });
            
            game_handle_lock.send_rpc(RpcCallContainer{
                function_name: "test_rpc_params".to_string(),
                params: vec![ObjectType::Integer(12345)]
            });

            test_counter = 0;
        }
        test_counter+=1;

        next_frame().await;
    }
}