use crate::game_handle::GameHandle;
use crate::player::{DataWrapper, Player,PlayerState};



pub fn spawn_main_player(game_handle: &mut GameHandle) {
    spawn_player(game_handle, DataWrapper { state: PlayerState::Idle, owner_id: 0, object_id: -1 });
}

pub fn spawn_player(
    game_handle: &mut GameHandle,
    wrapper: DataWrapper,
) {
    let world = game_handle.get_world();
    let mut locked_world = world.lock().unwrap();
    let player = Player::construct_from_wrapper(wrapper,&mut locked_world);

    if player.wrapper.owner_id == 0 {
        game_handle.add_player(player);
    }
}