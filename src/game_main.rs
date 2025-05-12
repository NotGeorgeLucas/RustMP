use macroquad::prelude::*;
use macroquad_tiled as tiled;
use macroquad_platformer::*;
use rust_mp::message::{ObjectType, RpcCallContainer};
use rust_mp::{player::*, PLAYER_SIZE_DATA};
use rust_mp::game_handle::GameHandle;
use std::sync::{Arc,Mutex};


#[macroquad::main("Platformer")]
async fn main() {
    let tileset = load_texture("assets/tileset.png").await.unwrap();
    tileset.set_filter(FilterMode::Nearest);

    let character_textures = CharacterTextures::load().await;
    let player_size_data = &*PLAYER_SIZE_DATA;
    let animation_frames = CharacterAnimationFrames::new();
   

    let tiled_map_json = load_string("assets/map.json").await.unwrap();
    let tiled_map = tiled::load_map(&tiled_map_json, &[("tileset.png", tileset)], &[]).unwrap();
    let mut static_colliders = vec![];
    for (_x, _y, tile) in tiled_map.tiles("main layer", None) {
        static_colliders.push(if tile.is_some() {
            Tile::Solid
        } else {
            Tile::Empty
        });
    }

    let world = Arc::new(Mutex::new(World::new()));
    world.lock().unwrap().add_static_tiled_layer(static_colliders, 8., 8., 40, 1);
    
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: game_main <is_server> <ip:port>");
        std::process::exit(1);
    }

    let is_server = args[1].parse::<bool>().unwrap_or(false);
    let ip_string = args[2].clone();
    
    let game_handle = if is_server {
        GameHandle::construct_server(Arc::clone(&world))
    } else {
        GameHandle::construct_client(ip_string,Arc::clone(&world))
       
    };
    


    {
        let player = Player::construct_from_wrapper(
            DataWrapper {
                state: PlayerState::Idle,
                owner_id: 0,
                object_id: -1,
                character_type: CharacterType::Witch,
                position_data: (15.0, 15.0),
                speed_data: (0.0, 0.0),
            },
            &mut world.lock().unwrap(),
            &player_size_data,
        );
        game_handle.lock().unwrap().add_player(player);
    }


    let mut frame_timer = 0.0;
    let camera = Camera2D::from_display_rect(Rect::new(0.0, 152.0, 320.0, -152.0));
   
    
    let mut test_counter: i32 = 0;
    loop {
        clear_background(BLACK);

        set_camera(&camera);

        tiled_map.draw_tiles("main layer", Rect::new(0.0, 0.0, 320.0, 152.0), None);

        let mut game_handle_lock = game_handle.lock().unwrap();
        let wrapper_map_mutex = game_handle_lock.get_player_wrapper_map();
        let mut wrapper_map = wrapper_map_mutex.lock().unwrap();

        for (player_index, player) in wrapper_map.iter_mut() {
            player.handle(
                &mut world.lock().unwrap(),
                &mut frame_timer,
                game_handle_lock.get_personal_id(),
                player.wrapper.character_type,
                &animation_frames,
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