
use bevy::prelude::*;
use bevy::input::ButtonInput;

fn start() {
    App::new() 
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, spawn_player)
        .add_systems(Update, move_player)
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