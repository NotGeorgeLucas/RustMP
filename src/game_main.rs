use bevy::prelude::*;
use bevy::input::ButtonInput;
use std::sync::{Arc, Mutex};
use RustMP::server::Server;
use RustMP::client::Client;



#[derive(Resource)]
struct GameWindow {
    width: f32,
    height: f32,
    title: String,
}


#[derive(Resource, Default)]
struct GameState {
}


pub struct GameHandle {
    game_state: Arc<Mutex<GameState>>,
    client: Option<Arc<Mutex<Client>>>,
    server: Option<Arc<Mutex<Server>>>,
}

impl GameHandle {
    fn launch_server(&mut self) -> Result<(),std::io::Error>{
        self.server = Some(Arc::new(Mutex::new(Server::new().unwrap())));

        if let Some(server) = self.server.take() {
            let server_clone = Arc::clone(&server);
            server.lock().unwrap().start(server_clone);
        }
        Ok(())
    }


    fn launch_client(&mut self,server_ip: String) -> Result<(), String> {
        if server_ip.is_empty() {
            return Err("No IP address provided".to_string());
        }

        self.client = Some(Arc::new(Mutex::new(Client::new(server_ip).unwrap())));

        if let Some(client) = self.client.take() {
            let client_clone = Arc::clone(&client);
            client.lock().unwrap().start(client_clone);
        }
        Ok(())
    }

    fn construct_client(game_state: Arc<Mutex<GameState>>, server_ip: String) -> Self {
        let mut handle:GameHandle = GameHandle {
            game_state: game_state,
            client: None,
            server: None,
        };
        handle.launch_client(server_ip).expect("Failed to launch client");
        handle
    }

    fn construct_server(game_state: Arc<Mutex<GameState>>) -> Self {
        let mut handle:GameHandle = GameHandle {
            game_state: game_state,
            client: None,
            server: None,
        };
        handle.launch_server().expect("Failed to launch client");
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
    let ip_string = &args[2];

    let game_state = Arc::new(Mutex::new(GameState::default()));
    let game_state_clone = game_state.clone();
    
    let window = GameWindow {
        width: 800.0,
        height: 600.0,
        title: String::from("Bevy Game"),
    };
    
    if is_server{
        GameHandle::construct_server(game_state_clone);
    }else{
        GameHandle::construct_client(game_state_clone, ip_string.clone());
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
        .add_systems(Startup, spawn_player)
        .run();
    
    
}


#[derive(Component)]
struct Player {
    speed: f32,
}

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Sprite {
            color: Color::rgb(0.3, 0.5, 0.8),
            custom_size: Some(Vec2::new(50.0, 50.0)),
            ..Default::default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        Player { speed: 5.0 },
    ));
}

fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Player, &mut Transform)>,
) {
    for (player, mut transform) in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::KeyW) {
            transform.translation.y += player.speed;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            transform.translation.y -= player.speed;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            transform.translation.x += player.speed;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            transform.translation.x -= player.speed;
        }
    }
}