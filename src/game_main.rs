use bevy::prelude::*;
use std::sync::{Arc, Mutex};
use RustMP::server::Server;
use RustMP::client::Client;
use bevy::input::keyboard::KeyCode;
use bevy::input::ButtonInput;
use bevy::app::Startup;
use bevy::app::Update;
use bevy::render::view::{InheritedVisibility,Visibility};

#[derive(Resource)]
struct GameWindow {
    width: f32,
    height: f32,
    title: String,
}

#[derive(Resource, Default)]
struct GameState;

pub struct GameHandle {
    game_state: Arc<Mutex<GameState>>,
    client: Option<Arc<Mutex<Client>>>,
    server: Option<Arc<Mutex<Server>>>,
}

impl GameHandle {
    fn launch_server(&mut self) -> Result<(), std::io::Error> {
        let server = Arc::new(Mutex::new(Server::new().unwrap()));
        server.lock().unwrap().start(Arc::clone(&server));
        self.server = Some(server);
        Ok(())
    }

    fn launch_client(&mut self, server_ip: String) -> Result<(), String> {
        if server_ip.is_empty() {
            return Err("No IP address provided".to_string());
        }

        let client = Arc::new(Mutex::new(Client::new(server_ip).unwrap()));
        client.lock().unwrap().start(Arc::clone(&client));
        self.client = Some(client);
        Ok(())
    }

    fn construct_client(game_state: Arc<Mutex<GameState>>, server_ip: String) -> Self {
        let mut handle = GameHandle {
            game_state,
            client: None,
            server: None,
        };
        handle.launch_client(server_ip).expect("Failed to launch client");
        handle
    }

    fn construct_server(game_state: Arc<Mutex<GameState>>) -> Self {
        let mut handle = GameHandle {
            game_state,
            client: None,
            server: None,
        };
        handle.launch_server().expect("Failed to launch server");
        handle
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: game_main <is_server> <ip:port>");
        std::process::exit(1);
    }

    let is_server = args[1].parse::<bool>().unwrap_or(false);
    let ip_string = args[2].clone();

    let game_state = Arc::new(Mutex::new(GameState::default()));

    let window = GameWindow {
        width: 800.0,
        height: 600.0,
        title: String::from("Bevy Game"),
    };

    if is_server {
        GameHandle::construct_server(Arc::clone(&game_state));
    } else {
        GameHandle::construct_client(Arc::clone(&game_state), ip_string);
    }

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: window.title.clone(),
                resolution: (window.width, window.height).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(window)
        .insert_resource(GameState::default())
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, spawn_player)
        .add_systems(Update, move_player)
        .run();
}

#[derive(Component)]
struct Player {
    speed: f32,
}

fn spawn_player(mut commands: Commands, _asset_server: Res<AssetServer>) {
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
        Player { speed: 200.0 },
    ));
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}

fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,  
    time: Res<Time>,                      
    mut query: Query<(&Player, &mut Transform)>,
) {
    for (player, mut transform) in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }

        transform.translation += direction.normalize_or_zero() * player.speed * time.delta_secs();
    }
}
