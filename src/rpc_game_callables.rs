use crate::player::{ Player, PlayerState};

pub fn animation_force(player: &mut Player, animation_state: PlayerState) {
    player.wrapper.state = animation_state;
    player.current_frame = 0;
    player.attack_frame = 0;
}