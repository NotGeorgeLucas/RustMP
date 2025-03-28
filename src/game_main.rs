use bevy::prelude::*;
use RustMP::network_sync::NetworkSync;
use RustMP::player::Player;
use RustMP::game_handle::GameHandle;
use RustMP::player_spawner;
use RustMP::player_spawner::GameHandleResource;
use bevy::input::keyboard::KeyCode;
use bevy::input::ButtonInput;


fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: game_main <is_server> <ip:port>");
        std::process::exit(1);
    }

    let is_server = args[1].parse::<bool>().unwrap_or(false);
    let ip_string = args[2].clone();
    
    let game_handle = if is_server {
        GameHandle::construct_server()
    } else {
        GameHandle::construct_client(ip_string)
    };

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: String::from("Bevy Game"),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(GameHandleResource(game_handle))
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, player_spawner::spawn_main_player)
        .add_systems(Update, move_player_system)
        .run();
}


fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}

fn move_player_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,  
    time: Res<Time>,                       
    mut query: Query<(&Player, &mut Transform)>,
) {

    for (player, mut transform) in query.iter_mut() {
        if player.get_owner() == 0 {
            player.move_player(&keyboard_input, &time, &mut transform);
        }
    }
}