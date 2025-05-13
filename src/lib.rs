pub mod client;
pub mod server;
pub mod message;
pub mod network_sync;
pub mod player;
pub mod game_handle;
pub mod rpc_game_callables;
pub mod rpc_funcs;

use std::collections::HashMap;
use std::sync::Arc;
use once_cell::sync::Lazy;
use crate::player::PlayerSizeData;
use crate::rpc_funcs::*;
use crate::rpc_game_callables::*;


pub static PLAYER_SIZE_DATA: Lazy<PlayerSizeData> = Lazy::new(|| {player::load_player_size_data()});

pub static RPC_FN_TABLE: Lazy<HashMap<&'static str, Arc<dyn RpcCallable>>> = Lazy::new(|| {
    let mut map: HashMap<&'static str, Arc<dyn RpcCallable>> = HashMap::new();
    map.insert("animation_force", Arc::new(PlayerStateFn(animation_force)));
    map
});

pub const SERVER_PORT:u16 = 13882;
pub const CLIENT_PORT:u16 = 28831;