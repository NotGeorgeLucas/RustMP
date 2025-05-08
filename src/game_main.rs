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

    let character_textures = CharacterTextures::load().await;
    let player_size_data = load_player_size_data();
    let animation_frames = CharacterAnimationFrames::new();
   

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
        let player = Player::construct_with_size(
            DataWrapper {
                state: PlayerState::Idle,
                owner_id: 0,
                object_id: -1,
                character_type: CharacterType::Witch,
            },
            &mut world.lock().unwrap(),
            &player_size_data,
        );
        game_handle.lock().unwrap().add_player(player);
    }
    let mut current_frame = 0;
    let mut frame_timer = 0.0;
    let camera = Camera2D::from_display_rect(Rect::new(0.0, 152.0, 320.0, -152.0));
   



    

    loop {
        clear_background(BLACK);

        set_camera(&camera);

        tiled_map.draw_tiles("main layer", Rect::new(0.0, 0.0, 320.0, 152.0), None);

        let mut game_handle_lock = game_handle.lock().unwrap();
        let wrapper_map_mutex = game_handle_lock.get_player_wrapper_map();
        let mut wrapper_map = wrapper_map_mutex.lock().unwrap();

        for (_, player) in wrapper_map.iter_mut() {
            player.handle(
                &mut world.lock().unwrap(),
                &mut current_frame,
                &mut frame_timer,
                game_handle_lock.get_personal_id(),
                player.wrapper.character_type,
                &animation_frames,
            );

            let character_type = player.wrapper.character_type;
            let frame_size = match character_type {
                CharacterType::Withest => player_size_data.witcher.idle.size_frame.clone(),
                CharacterType::Witch => player_size_data.witch.idle.size_frame.clone(),
            };

            let player_size = vec2(frame_size.width / 10.0, frame_size.height / 10.0);

            player.render(
                current_frame,
                &character_textures,
                player_size,
                character_type,
                &world.lock().unwrap(),
                &player_size_data,
            );
        }

        next_frame().await;
    }
}