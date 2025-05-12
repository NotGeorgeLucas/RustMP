use crate::RPC_FN_TABLE;
use crate::message::{ObjectType, RpcCallContainer};



pub trait RpcCallable: Send + Sync {
    fn call(&self, args: Vec<ObjectType>);
}

pub struct NoParamFn(pub fn());

impl RpcCallable for NoParamFn {
    fn call(&self, _args: Vec<ObjectType>) {
        (self.0)();
    }
}

pub struct IntParamFn(pub fn(i32));

impl RpcCallable for IntParamFn {
    fn call(&self, args: Vec<ObjectType>) {
        if let Some(ObjectType::Integer(val)) = args.get(0) {
            (self.0)(*val);
        } else {
            eprintln!("Expected Integer argument for i32 RPC function");
        }
    }
}

pub fn invoke_rpc(call: &RpcCallContainer) {
    if let Some(func) = RPC_FN_TABLE.get(call.function_name.as_str()) {
        func.call(call.params.clone());
    } else {
        eprintln!("Unknown RPC function: {}", call.function_name);
    }
}