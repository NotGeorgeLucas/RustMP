use bevy::prelude::*;
use std::sync::{Arc, Mutex};
use RustMP::network_sync::NetworkSync;
use RustMP::player::Player;
use RustMP::game_handle::GameHandle;
use bevy::input::keyboard::KeyCode;
use bevy::input::ButtonInput;

#[derive(Resource)]
struct GameHandleResource(Arc<Mutex<GameHandle>>);

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
        .add_systems(Startup, spawn_main_player)
        .add_systems(Update, move_player_system)
        .run();
}

fn spawn_main_player(
    mut commands: Commands, 
    asset_server: Res<AssetServer>, 
    game_handle_res: ResMut<GameHandleResource>
) {
    spawn_player(
        &mut commands, 
        &asset_server, 
        0, 
        &mut game_handle_res.0.lock().unwrap()
    );
}

fn spawn_player(
    commands: &mut Commands, 
    _asset_server: &Res<AssetServer>, 
    owner_id: i32, 
    game_handle: &mut GameHandle
) {
    let player = Player::new(200.0, owner_id);
    commands.spawn((
        Sprite {
            color: Color::srgb(0.3, 0.5, 0.8),
            custom_size: Some(Vec2::new(50.0, 50.0)),
            ..Default::default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        player.clone(),
    ));
    game_handle.add_player(player);
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