pub mod client;
pub mod server;
pub mod message;
pub mod network_sync;
pub mod player;
pub mod game_handle;
pub mod rpcexample;
pub mod rpc_funcs;

use std::collections::HashMap;
use std::sync::Arc;
use once_cell::sync::Lazy;
use crate::player::PlayerSizeData;
use crate::rpc_funcs::{RpcCallable,NoParamFn,IntParamFn};
use crate::rpcexample::*;


pub static PLAYER_SIZE_DATA: Lazy<PlayerSizeData> = Lazy::new(|| {player::load_player_size_data()});

pub static RPC_FN_TABLE: Lazy<HashMap<&'static str, Arc<dyn RpcCallable>>> = Lazy::new(|| {
    let mut map: HashMap<&'static str, Arc<dyn RpcCallable>> = HashMap::new();
    map.insert("test_rpc_no_param", Arc::new(NoParamFn(test_rpc_no_param)));
    map.insert("test_rpc_params", Arc::new(IntParamFn(test_rpc_params)));
    map
});

pub const SERVER_PORT:u16 = 13882;
pub const CLIENT_PORT:u16 = 28831;