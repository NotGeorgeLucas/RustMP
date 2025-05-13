use crate::RPC_FN_TABLE;
use crate::message::{ObjectType, RpcCallContainer};
use crate::player::{Player,PlayerState};

pub enum RuntimeParams{
    None,
    Player,
}
pub trait RpcCallable: Send + Sync {
    fn call(&self, params: Vec<ObjectType>, runtime_args: &mut [RuntimeArg]);
    fn get_runtime_params(&self) -> RuntimeParams;
}

pub struct NoParamFn(pub fn ());

impl RpcCallable for NoParamFn {
    fn call(&self, _params: Vec<ObjectType>, _runtime_args: &mut [RuntimeArg]) {
        (self.0)();
    }
    fn get_runtime_params(&self) -> RuntimeParams {
        RuntimeParams::None
    }
}

pub struct IntParamFn(pub fn(i32));

impl RpcCallable for IntParamFn {
    fn call(&self, params: Vec<ObjectType>, _runtime_args: &mut [RuntimeArg]) {
        if let Some(ObjectType::Integer(val)) = params.get(0) {
            (self.0)(*val);
        } else {
            eprintln!("Expected Integer argument for i32 RPC function");
        }
    }
    fn get_runtime_params(&self) -> RuntimeParams {
        RuntimeParams::None
    }
}



pub fn invoke_rpc(call: &RpcCallContainer, runtime_args: &mut [RuntimeArg]) {
    if let Some(func) = RPC_FN_TABLE.get(call.function_name.as_str()) {
        func.call(call.params.clone(), runtime_args);
    } else {
        eprintln!("Unknown RPC function: {}", call.function_name);
    }
}


pub struct PlayerStateFn(pub fn(&mut Player, PlayerState));

#[allow(irrefutable_let_patterns)]
impl RpcCallable for PlayerStateFn {
    fn call(&self, params: Vec<ObjectType>, runtime_args: &mut [RuntimeArg]) {
        if runtime_args.is_empty() {
            eprintln!("No runtime arguments passed (expected Player)");
            return;
        }

        
        let mut_arg = &mut runtime_args[0];
        
        let player = if let RuntimeArg::Player(ref mut player) = mut_arg {
            player
        } else {
            eprintln!("First runtime argument was not a Player");
            return;
        };

        if let Some(ObjectType::AnimationState(state)) = params.get(0) {
            (self.0)(player, *state);
        } else {
            eprintln!("Expected AnimationState as first parameter");
        }
    }
    fn get_runtime_params(&self) -> RuntimeParams {
        RuntimeParams::Player
    }
}


pub enum RuntimeArg<'a> {
    Player(&'a mut Player),
}

#[allow(unreachable_patterns)]
impl<'a> RuntimeArg<'a> {
    pub fn as_player_mut(&mut self) -> Option<&mut Player> {
        match self {
            RuntimeArg::Player(p) => Some(*p),
            _ => None,
        }
    }
}