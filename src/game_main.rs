use bevy::prelude::*;
use bevy::input::ButtonInput;
use std::sync::{Arc, Mutex};

// Define a window configuration struct
#[derive(Resource)]
struct GameWindow {
    width: f32,
    height: f32,
    title: String,
}

// Define a shared state for external communication
#[derive(Resource, Default)]
struct GameState {
    should_quit: bool,
    // Add any other state you need to control externally
}

// Create a handle struct to interact with the game
pub struct GameHandle {
    game_state: Arc<Mutex<GameState>>,
}

impl GameHandle {
    
}

pub fn start() -> GameHandle {
    // Create shared state for communication
    let game_state = Arc::new(Mutex::new(GameState::default()));
    let game_state_clone = game_state.clone();
    
    // Create window configuration
    let window = GameWindow {
        width: 800.0,
        height: 600.0,
        title: String::from("My Bevy Game"),
    };
    
    // Spawn the game in a separate thread
    std::thread::spawn(move || {
        let state_resource = GameState {
            should_quit: false,
            // Initialize other fields as needed
        };
        
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
            .insert_resource(state_resource)
            .add_systems(Startup, spawn_player)
            .add_systems(Update, (move_player, check_quit_state))
            .run();
    });
    
    // Return a handle to interact with the game
    GameHandle {
        game_state: game_state_clone,
    }
}

// System to check if we should quit
fn check_quit_state(state: Res<GameState>, mut exit: EventWriter<AppExit>) {
    if state.should_quit {
        exit.send(AppExit::Success);
    }
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