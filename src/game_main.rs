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
    const _FRAME_WIDTH: f32 = 512.0;
    const _FRAME_HEIGHT: f32 = 512.0;
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
        let player = Player {
            collider: world.lock().unwrap().add_actor(vec2(15.0, 15.0), 16, 16,),
            speed: vec2(0., 0.),
            wrapper: DataWrapper{state: PlayerState::Idle, owner_id: 0, object_id:-1},
        };
        game_handle.lock().unwrap().add_player(player);
    }

    
    let player_textures = PlayerTextures {
        run: &run_texture,
        idle: &idle_texture,
        jump: &jump_texture,
        attack1: &attack1_texture,
        attack2: &attack2_texture,
        world: &world,
    };    



    let camera = Camera2D::from_display_rect(Rect::new(0.0, 152.0, 320.0, -152.0));

    loop {
        clear_background(BLACK);

        set_camera(&camera);

        tiled_map.draw_tiles("main layer", Rect::new(0.0, 0.0, 320.0, 152.0), None);

        let mut game_handle_lock = game_handle.lock().unwrap();
        let wrapper_map_mutex = game_handle_lock.get_player_wrapper_map();
        let mut wrapper_map = wrapper_map_mutex.lock().unwrap();
        for (_, player) in wrapper_map.iter_mut(){
            player.handle(&mut world.lock().unwrap(), &mut current_frame, &mut frame_timer, game_handle_lock.get_personal_id());
            player.render(current_frame, &player_textures, vec2(100.0, 100.0), 1042.0);
        }
        drop(game_handle_lock);
        

        next_frame().await;
    }
}