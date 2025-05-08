pub mod client;
pub mod server;
pub mod message;
pub mod network_sync;
pub mod player;
pub mod game_handle;

use once_cell::sync::Lazy;
use crate::player::PlayerSizeData;
pub static PLAYER_SIZE_DATA: Lazy<PlayerSizeData> = Lazy::new(|| {player::load_player_size_data()});

pub const SERVER_PORT:u16 = 13882;
pub const CLIENT_PORT:u16 = 28831;