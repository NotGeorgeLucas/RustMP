use bevy::prelude::*;
use RustMP::message::{Message,ObjectType};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use RustMP::server::Server;
use RustMP::client::Client;
use RustMP::network_sync::NetworkSync;
use RustMP::player::Player;
use bevy::input::keyboard::KeyCode;
use bevy::input::ButtonInput;
use bevy::app::Startup;
use bevy::app::Update;
use bevy::render::view::{InheritedVisibility,Visibility};

#[derive(Resource, Default)]
struct GameState;

#[derive(Resource)]
pub struct GameHandle {
    game_state: Arc<Mutex<GameState>>,
    client: Option<Arc<Mutex<Client>>>,
    server: Option<Arc<Mutex<Server>>>,
}

impl GameHandle {

    fn add_player(&mut self, player: Player){
        if let Some(server_arc) = &self.server {
            if let Ok(server) = server_arc.lock() {
                let mut message: HashMap<String, ObjectType> = HashMap::new();
                message.insert(String::from("goal"), ObjectType::StringMsg(String::from("player_join")));
                message.insert(String::from("player"), ObjectType::Player(player));
                server.tx.as_ref().unwrap().send(Message::new(-1,message).unwrap()).expect("Message failed to send");
                println!("BEEEEEEEEEEEEERs");
            } else {
                eprintln!("Failed to lock the mutex");
            }
        } else if let Some(client_arc) = &self.client{
            if let Ok(client) = client_arc.lock() {
                let mut message: HashMap<String, ObjectType> = HashMap::new();
                message.insert(String::from("goal"), ObjectType::StringMsg(String::from("player_join")));
                message.insert(String::from("player"), ObjectType::Player(player));
                client.send_message(&message).expect("Message failed to send");
                println!("BEEEEEEEEEEEEERs");
            } else {
                eprintln!("Failed to lock the mutex");
            }
        }
    }

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

    
    let game_handle: GameHandle;
    if is_server {
        game_handle = GameHandle::construct_server(Arc::clone(&game_state));
    } else {
        game_handle = GameHandle::construct_client(Arc::clone(&game_state), ip_string);
    }

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: String::from("Bevy Game"),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(GameState::default())
        .insert_resource(game_handle)
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, spawn_main_player)
        .add_systems(Update, move_player_system)
        .run();
}


fn spawn_main_player(commands: Commands, asset_server: Res<AssetServer>, mut game_handle: ResMut<GameHandle>) {
    spawn_player(commands, asset_server, 0, &mut game_handle);
}

fn spawn_player(mut commands: Commands, _asset_server: Res<AssetServer>, owner_id: i32, game_handle: &mut ResMut<GameHandle>) {
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
