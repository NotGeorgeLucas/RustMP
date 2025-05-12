use crate::network_sync::NetworkSync;
use crate::{CLIENT_PORT, PLAYER_SIZE_DATA, SERVER_PORT};
use crate::message::{Message, MotionDataContainer, ObjectType};
use crate::player::{DataWrapper, Player};
use std::collections::HashMap;
use std::net::{UdpSocket, SocketAddr};
use std::io::{Result,ErrorKind};
use std::sync::{Arc,Mutex};
use std::thread;
use std::str::FromStr;
use std::time::Duration;
use colored::*;
use macroquad::math::vec2;
use macroquad_platformer::World;

#[derive(Clone)]
pub struct Server {
    socket: Arc<Mutex<UdpSocket>>,
    user_map: HashMap<i32,SocketAddr>,
    synced_players: Arc<Mutex<HashMap<i32, DataWrapper>>>, //    🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥
    world: Arc<Mutex<World>>,
    player_map_mutex: Arc<Mutex<HashMap<i32,Player>>>,
}




impl Server {
    pub fn new(world:Arc<Mutex<World>>, players: Arc<Mutex<HashMap<i32,Player>>>) -> Result<Server> {
        let server_address = format!("0.0.0.0:{}",SERVER_PORT);
        let socket = UdpSocket::bind(server_address.clone()).unwrap();
        socket.set_nonblocking(true)?;

        Ok(Server {
            socket: Arc::new(Mutex::new(socket)),
            user_map: HashMap::new(),
            synced_players: Arc::new(Mutex::new(HashMap::new())),
            world: world,
            player_map_mutex: players,
        })
    }

    fn gen_new_id(&self) -> i32{
        let mut key:i32 = 1;
        loop {
            if !self.user_map.contains_key(&key){
                break;
            }
            key+=1;
        }
        key
    }

    pub fn gen_new_player_id(&self) -> i32{
        let mut key:i32 = 1;
        let synced_players = self.synced_players.lock().unwrap();
        loop {
            if !synced_players.contains_key(&key){
                break;
            }
            key+=1;
        }
        key
    }


    pub fn id_to_socket(&self, id: i32) -> Option<SocketAddr> {
        if let Some(socket_addr) = self.user_map.get(&id) {
            Some(*socket_addr)
        } else {
            None
        }
    }


    pub fn socket_to_id(&self, socket: SocketAddr) -> Option<i32> {
        self.user_map
            .iter()
            .find_map(|(id, addr)| if *addr == socket { Some(*id) } else { None })
    }


    pub fn get_world(&self) -> Arc<Mutex<World>> {
        Arc::clone(&self.world)
    }


    pub fn get_synced_players(&self) -> Arc<Mutex<HashMap<i32, DataWrapper>>> {
        Arc::clone(&self.synced_players)
    }


    pub fn get_user_map(&self) -> &HashMap<i32,SocketAddr> {
        &self.user_map
    }


    pub fn add_player(&mut self, mut player: Player, owner_id: i32) -> i32{
        let new_id = self.gen_new_player_id();
        player.wrapper.owner_id = owner_id;

        self.synced_players.lock().unwrap().insert(new_id, player.wrapper);
        new_id        
    }


    pub fn send_motion_update(&self, target_vector: Vec<&SocketAddr>, object_id: i32, motion_data: MotionDataContainer) {
        let mut message = HashMap::new();
        message.insert("goal".to_string(), ObjectType::StringMsg("motion_update_broadcast".to_string()));
        message.insert("object_id".to_string(), ObjectType::Integer(object_id));
        message.insert("motion_data".to_string(), ObjectType::MotionData(motion_data));

        for target in target_vector.iter(){
            if let Err(e) = self.send_message(&message, **target){
                eprintln!("Failed to send message: {}", e);
            }
        }
    }


    fn process_message(&mut self,message_received: &Message,client_address:SocketAddr) -> HashMap<String,ObjectType>{
        let mut response_map = HashMap::new();
        let received_map = message_received.get_message_map();
        println!("{:?}",received_map);
        if received_map.contains_key("goal"){
            match received_map.get("goal"){
                Some(ObjectType::StringMsg(goal)) => {
                    println!("Received message with goal: {}", goal.as_str().bold());
                    match goal.as_str() {
                        "sync" => {
                            response_map.insert(String::from("goal"), ObjectType::StringMsg(String::from("confirm connect")));
                            let new_id = self.gen_new_id();
                            self.user_map.insert(new_id,(client_address.ip(),CLIENT_PORT).into());
                            response_map.insert(String::from("id"), ObjectType::Integer(new_id));
                        },
                        "get_sync_players" => {
                            response_map.insert("goal".into(), ObjectType::StringMsg("ret_sync_players".into()));
                            response_map.insert("players".into(), ObjectType::PlayerMap(self.synced_players.lock().unwrap().clone()));
                        },
                        "add_player" => {
                            if let Some(player_obj) = received_map.get("player") {
                                match player_obj {
                                    ObjectType::Player(pl) => {
                                        let mut world = self.world.lock().unwrap();
                                        let mut pl = Player::construct_from_wrapper(*pl, &mut world, &*PLAYER_SIZE_DATA);
                                        drop(world);
                                        let new_id: i32;
                                        if let Some(client_id) = self.socket_to_id(client_address){
                                            if !self.synced_players.lock().unwrap().contains_key(&pl.get_object_id()){
                                                pl.set_owner(client_id);
                                                new_id = self.add_player(pl, client_id);
                                                let mut wrapper_map = self.player_map_mutex.lock().unwrap();
                                                wrapper_map.insert(new_id, pl);
                                                response_map.insert("goal".into(), ObjectType::StringMsg("ret_player_obj_id".into()));
                                                response_map.insert("id".into(), ObjectType::Integer(new_id));
                                            }
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
                        "object_pos_update" => {
                            if let Some(ObjectType::Integer(pl_id)) = received_map.get("object_id") {
                                if let Some(pl) = self.player_map_mutex.lock().unwrap().get_mut(pl_id) {
                                    if let Some(ObjectType::MotionData(motion_data)) = received_map.get("motion_data"){
                                        pl.wrapper.position_data = (motion_data.x, motion_data.y);
                                        pl.wrapper.speed_data = (motion_data.x_speed, motion_data.y_speed);
                                        pl.wrapper.state = motion_data.animation_state;

                                        let mut locked_world = self.world.lock().unwrap();
                                        locked_world.set_actor_position(pl.collider, vec2(motion_data.x, motion_data.y));
                                        
                                        pl.speed = vec2(motion_data.x_speed, motion_data.y_speed);



                                        let mut target_vector: Vec<&SocketAddr> = Vec::new();
                            
                                        for (_, target) in self.user_map.iter(){

                                            if target.to_string() != "127.0.0.1:13882" && *target != client_address {
                                                target_vector.push(target);
                                            }
                                        }
                                        
                                        self.send_motion_update(target_vector, *pl_id, motion_data.to_owned());

                                    } else {
                                        eprintln!("Motion Data for motion updated was not provided in the proper format");
                                    }
                                } else{
                                    eprintln!("Object with id {} not found in server's player map", pl_id);
                                }
                            }else{
                                eprintln!("object_id for motion_update_broadcast was incorrectly supplied");
                            }
                        },
                        _ =>{
                            println!("{}", "Unknown message type".red());
                        }
                    }
                }
                None =>{
                    println!("Goal field empty");
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
        
        let (size, sender) = {
            let socket = self.socket.lock().unwrap();
            match socket.recv_from(&mut buffer) {
                Ok(result) => result,
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
                let response_map = self.process_message(&decoded, sender);
                
                if !response_map.is_empty() {
                    self.send_message(&response_map, sender)?;
                    println!("Sent response: {:?}", response_map);
                }
            }
            Err(e) => {
                println!("Failed to decode message: {}", e);
            }
        }
        
        Ok(())
    }


    pub fn send_message(&self,message: &HashMap<String,ObjectType>,target:SocketAddr) -> Result<()> {
        if let Ok(message_struct) = Message::new(-1, message.clone()) {
            let message_bytes = bincode::serialize(&message_struct).unwrap();
            
            self.socket.lock().unwrap().send_to(&message_bytes, target)?;
            println!("Sent packet to {}", target);
        }else{
            eprintln!("Failed to create message: Message malformed");
        }
        
        Ok(())
    }

    pub fn start(&mut self, self_mutex: Arc<Mutex<Self>>) {
        let mut_ref = Arc::clone(&self_mutex);

        let _receive_thread = thread::spawn(move || {
            loop {
                {
                    let mut locked = mut_ref.lock().unwrap();
                    if let Err(e) = locked.receive_message() {
                        eprintln!("Failed to receive message: {:?}", e);
                    }
                }
                thread::sleep(Duration::from_millis(8));
            }
        });
        self.user_map.insert(-1, SocketAddr::from_str(format!("127.0.0.1:{}",SERVER_PORT).as_str()).unwrap());
    }
}

