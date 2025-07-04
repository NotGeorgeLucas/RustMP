use crate::message::{Message, ObjectType};
use crate::network_sync::NetworkSync;
use crate::player::{DataWrapper,Player};
use crate::{CLIENT_PORT,RPC_FN_TABLE};
use crate::rpc_funcs::{invoke_rpc,RuntimeArg, RuntimeParams};
use crate::PLAYER_SIZE_DATA;
use std::collections::HashMap;
use std::net::{UdpSocket, SocketAddr};
use std::io::Result;
use std::str::FromStr;
use std::thread;
use std::sync::{Arc, Mutex, mpsc::{self, Sender, Receiver}};
use std::time::Duration;
use std::io::ErrorKind;
use colored::Colorize;
use macroquad::math::vec2;
use macroquad_platformer::World;

#[derive(Clone)]
pub struct Client {
    server_address: SocketAddr,
    socket: Arc<Mutex<UdpSocket>>,
    personal_id:i32,
    synced_players: Arc<Mutex<HashMap<i32, DataWrapper>>>,
    world: Arc<Mutex<World>>,
    tx: Option<Sender<HashMap<String, ObjectType>>>,
    new_player_id: Option<i32>,
    player_map_mutex: Arc<Mutex<HashMap<i32,Player>>>,
}



impl Client{
    pub fn new(server_address_ip: String, world: Arc<Mutex<World>>, players:Arc<Mutex<HashMap<i32, Player>>>) -> Result<Client> {
        let mut server_address = server_address_ip.clone();
        server_address = server_address;
        let socket = UdpSocket::bind(format!("0.0.0.0:{}",CLIENT_PORT))?;
        println!("Client bound to: {:?}", socket.local_addr()?);
        socket.set_nonblocking(true)?;
        
        Ok(Client{
            server_address:SocketAddr::from_str(&server_address).unwrap(),
            socket:Arc::new(Mutex::new(socket)),
            personal_id:0,
            synced_players: Arc::new(Mutex::new(HashMap::new())),
            world: world,
            tx: None,
            new_player_id: None,
            player_map_mutex: players,
        })
    }


    pub fn get_world(&self) -> Arc<Mutex<World>> {
        Arc::clone(&self.world)
    }


    pub fn get_personal_id(&self) -> i32 {
        self.personal_id
    }
    

    #[allow(unused_mut)]
    fn process_message(&mut self,message_received: &Message) -> HashMap<String,ObjectType>{

        let mut response_map = HashMap::new();
        let received_map = message_received.get_message_map();
        println!("{:?}",received_map);
        if received_map.contains_key("goal"){
            match received_map.get("goal"){
                Some(ObjectType::StringMsg(goal)) => {
                    println!("Received message with goal: {}", goal.as_str().bold());
                    match goal.as_str() {
                        "confirm connect" => {
                            if let Some(ObjectType::Integer(new_id)) = received_map.get("id") {
                                self.personal_id = *new_id;
                            }else{
                                eprintln!("ID not a valid i32")
                            }
                        },
                        "ret_sync_players" => {
                            if let Some(ObjectType::PlayerMap(players)) = received_map.get("players"){
                                self.synced_players = Arc::new(Mutex::new(players.clone()));
                            }else{
                                eprintln!("Invalid player map return type");
                            }
                        },
                        "ret_player_obj_id" => {
                            if let Some(ObjectType::Integer(new_player_id)) = received_map.get("id"){
                                self.new_player_id = Some(*new_player_id);
                            }else{
                                eprintln!("Invalid player id return type");
                            }
                        },
                        "add_player" => {
                            if let Some(player_obj) = received_map.get("player") {
                                match player_obj {
                                    ObjectType::Player(pl) => {
                                        let mut world = self.world.lock().unwrap();
                                        let mut pl = Player::construct_from_wrapper(*pl, &mut world, &*PLAYER_SIZE_DATA);
                                        drop(world);
                                        let object_id = pl.get_object_id();
                                        let owner_id = pl.get_owner();

                                        if !self.synced_players.lock().unwrap().contains_key(&pl.get_object_id()){
                                            pl.set_owner(owner_id);
                                            self.synced_players.lock().unwrap().insert(object_id, pl.wrapper);
                                            let mut wrapper_map = self.player_map_mutex.lock().unwrap();
                                            wrapper_map.insert(object_id, pl);                                                    
                                        }
                                        
                                    },
                                    _ => {
                                        eprintln!("player field is invalid type");
                                    }
                                }
                            } else {
                                eprintln!("Missing 'player' field in received_map");
                            }
                        },
                        "motion_update_broadcast" => {
                            if let Some(ObjectType::Integer(pl_id)) = received_map.get("object_id") {
                                if let Some(pl) = self.player_map_mutex.lock().unwrap().get_mut(pl_id) {
                                    if let Some(ObjectType::MotionData(motion_data)) = received_map.get("motion_data"){
                                        pl.wrapper.position_data = (motion_data.x, motion_data.y);
                                        pl.wrapper.speed_data = (motion_data.x_speed, motion_data.y_speed);
                                        pl.wrapper.state = motion_data.animation_state;
                                        pl.wrapper.facing_right = motion_data.facing_right;

                                        let mut locked_world = self.world.lock().unwrap();
                                        locked_world.set_actor_position(pl.collider, vec2(motion_data.x, motion_data.y));
                                        
                                        pl.speed = vec2(motion_data.x_speed, motion_data.y_speed);
                                        pl.facing_right = motion_data.facing_right;
                                    } else {
                                        eprintln!("Motion Data for motion updated was not provided in the proper format");
                                    }
                                } else{
                                    eprintln!("Object with id {} not found in client's player map", pl_id);
                                }
                            }else{
                                eprintln!("object_id for motion_update_broadcast was incorrectly supplied");
                            }
                        },
                        "rpc_call" => {
                            if let Some(ObjectType::RpcCall(rpc_data)) = received_map.get("rpc_data"){
                                if let Some(func) = RPC_FN_TABLE.get(rpc_data.function_name.as_str()) {
                                    let mut runtime_args = Vec::new();
                                    let mut rpc_data_mut = rpc_data.clone();


                                    let mut player_map_locked = self.player_map_mutex.lock().unwrap();
                                    match func.get_runtime_params() {
                                        RuntimeParams::Player => {
                                            if let Some(ObjectType::Integer(player_id)) = rpc_data_mut.params.get(0) {
                                                if let Some(player) = player_map_locked.get_mut(player_id) {
                                                    runtime_args.push(RuntimeArg::Player(player));
                                                } else {
                                                    eprintln!("Player with ID {} not found", player_id);
                                                }
                                                rpc_data_mut.params.remove(0);
                                            } else {
                                                eprintln!("Expected player_id as first parameter for Player-based function");
                                            }
                                        }
                                        RuntimeParams::None => { }
                                    }

                                    invoke_rpc(&rpc_data_mut, &mut runtime_args);
                                }
                            }
                        },
                        _ =>{
                            println!("{}", "Unknown message type!".red());
                        }
                    }
                }
                None =>{
                    eprintln!("Goal field empty");
                }
                _ =>{
                    eprintln!("Invalid goal type!")
                }
            }
        }
        response_map
    }

    
    fn receive_message(&mut self) -> Result<()> {
        let mut buffer = [0u8; 1024];
        
        let (size, _) = {
            let socket = self.socket.lock().unwrap();
            match socket.recv_from(&mut buffer) {
                Ok(result) => {
                    result
                },
                Err(e) => {
                    if e.kind() != ErrorKind::WouldBlock {
                        eprintln!("Error encountered while trying to receive message: {}", e);
                    }
                    return Ok(());
                }
            }
        };
        match bincode::deserialize::<Message>(&buffer[..size]) {
            Ok(decoded) => {
                let response_map = self.process_message(&decoded);
                
                if !response_map.is_empty() {
                    self.send_message(&response_map)?;
                    println!("Sent response: {:?}", response_map);
                }
            }
            Err(e) => {
                println!("Failed to decode message: {}", e);
            }
        }
        
        Ok(())
    }

    pub fn send_message(&self,message: &HashMap<String,ObjectType>) -> Result<()> {
        if let Ok(message_struct) = Message::new(-1, message.clone()) {
            let message_bytes = bincode::serialize(&message_struct).unwrap();
            
            self.socket.lock().unwrap().send_to(&message_bytes, self.server_address)?;
            println!("Sent packet to {}", self.server_address);
        }else{
            eprintln!("Failed to create message: Message malformed");
        }
        
        Ok(())
    }


    pub fn send_to_receive_thread(&self, msg: HashMap<String, ObjectType>) -> Result<()>{
        if let Some(tx) = &self.tx {
            if let Err(e) = tx.send(msg) {
                eprintln!("Failed to send to receive thread: {:?}", e);
            }
        } else {
            eprintln!("{}","The pipe to receive thread is none!".bold().bright_red());
        }
        Ok(())
    }

    
    pub fn get_synced_players(&self) -> Arc<Mutex<HashMap<i32, DataWrapper>>> {
        Arc::clone(&self.synced_players)
    }


    pub fn get_new_player_id(&self) -> Option<i32> {
        self.new_player_id
    }


    pub fn start(&mut self, self_mutex: Arc<Mutex<Self>>) {
        let mut_ref = Arc::clone(&self_mutex);
        let (tx, rx): (
            Sender<HashMap<String, ObjectType>>,
            Receiver<HashMap<String, ObjectType>>,
        ) = mpsc::channel();

        
        self.tx = Some(tx);
        

        let _receive_thread = thread::spawn(move || {
            loop {
                {
                    let mut locked = mut_ref.lock().unwrap();
                    if let Err(e) = locked.receive_message() {
                        eprintln!("Failed to receive message: {:?}", e);
                    }
            
                    while let Ok(msg) = rx.try_recv() {
                        if let Err(e) = locked.send_message(&msg) {
                            eprintln!("Sending message failed: {:?}", e);
                        }
                    }
                }
                thread::sleep(Duration::from_millis(8));
            }
        });
    }
    
}