use bevy::prelude::*;
use std::sync::{Arc,Mutex};
use crate::game_handle::GameHandle;
use crate::player::Player;



#[derive(Resource)]
pub struct GameHandleResource(pub Arc<Mutex<GameHandle>>);

pub fn spawn_main_player(
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

pub fn spawn_player(
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
    game_handle.add_player(player,commands,_asset_server,owner_id);
}