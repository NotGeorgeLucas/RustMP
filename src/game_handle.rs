use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use crate::client::Client;
use crate::server::Server;
use crate::player::{DataWrapper, Player};
use crate::network_sync::NetworkSync;
use crate::message::{MotionDataContainer, ObjectType, RpcCallContainer};
use crate::PLAYER_SIZE_DATA;
use colored::*;
use macroquad_platformer::World;



#[derive(Clone)]
pub struct GameHandle {
    client: Option<Arc<Mutex<Client>>>,
    server: Option<Arc<Mutex<Server>>>,
    player_wrapper_map: Arc<Mutex<HashMap<i32, Player>>>,
    personal_id: i32,
}

impl GameHandle {

    pub fn add_player(&mut self, mut player: Player) -> Option<i32> {
        if let Some(server_arc) = &self.server {
            let mut server_lock = server_arc.lock().unwrap();
    
            let object_id = server_lock.gen_new_player_id();
            player.set_object_id(object_id);
            player.set_owner(self.personal_id);
            
            let mut player_wrapper_map = self.player_wrapper_map.lock().unwrap();
            player_wrapper_map.insert(object_id, player);

            server_lock.add_player(player,self.personal_id);

            return Some(player.get_object_id());

        } else if let Some(client_arc) = &self.client {

            for _ in 1..6 {
                {
                    let client_lock = client_arc.lock().unwrap();

                    let mut message = HashMap::new();
                    message.insert("goal".to_string(), ObjectType::StringMsg("add_player".to_string()));
                    message.insert("player".to_string(), ObjectType::Player(player.wrapper));

                    if let Err(e) = client_lock.send_to_receive_thread(message) {
                        eprintln!("Could not send message: {}", e);
                    }
                }

                thread::sleep(Duration::from_millis(200));

                let client_lock = client_arc.lock().unwrap();
                let new_player_id = client_lock.get_new_player_id();
                let synced_players = client_lock.get_synced_players();
                drop(client_lock);
                if new_player_id.is_some(){
                    player.set_object_id(new_player_id.unwrap());
                    synced_players.lock().unwrap().insert(new_player_id.unwrap(), player.wrapper);    

                    player.set_owner(self.personal_id);

                    let mut player_wrapper_map = self.player_wrapper_map.lock().unwrap();
                    player_wrapper_map.insert(new_player_id.unwrap(), player);
                    return Some(player.get_object_id());
                }
            }

        }else{ panic!("Game Handle has not been initialized properly"); }
        None
    }


    pub fn request_synced_players(&mut self) {
        if let Some(client_arc) = &self.client {
            for _ in 1..6 {
                {
                    let client_lock = client_arc.lock().unwrap();

                    let mut message = HashMap::new();
                    message.insert("goal".to_string(), ObjectType::StringMsg("get_sync_players".to_string()));

                    if let Err(e) = client_lock.send_to_receive_thread(message) {
                        eprintln!("Could not send message: {}", e);
                    }
                }

                thread::sleep(Duration::from_millis(200));

                let client_lock = client_arc.lock().unwrap();
                let synced_players_mutex = client_lock.get_synced_players();
                let player_map = synced_players_mutex.lock().unwrap();
                drop(client_lock);
                if !player_map.is_empty() {
                    let world = self.get_world();
                    let mut world = world.lock().unwrap();
                    for (_, wrapper) in player_map.iter(){
                        let player = Player::construct_from_wrapper(*wrapper, &mut world, &*PLAYER_SIZE_DATA);
                        let mut player_wrapper_map = self.player_wrapper_map.lock().unwrap();
                        player_wrapper_map.insert(player.get_object_id(), player);
                    }
                    break;
                }
            }
            
        } else{
            eprintln!("Cannot request players when running as the server, or client was not initialized correctly!");
        }
    }


    pub fn send_motion_update(&self, object_id: i32, motion_data: MotionDataContainer) {
        if self.server.is_some(){
            let server_locked = self.server.as_ref().unwrap().lock().unwrap();

            if let Some(wrapper) = server_locked.get_synced_players().lock().unwrap().get_mut(&object_id){
                wrapper.position_data = (motion_data.x, motion_data.y);
                wrapper.speed_data = (motion_data.x_speed, motion_data.y_speed);
                let socket_map = server_locked.get_user_map();
                let mut target_vector: Vec<&SocketAddr> = Vec::new();
    
                for (_, target) in socket_map.iter(){
                    
                    if target.to_string() != "127.0.0.1:13882" {
                        target_vector.push(target);
                    }
                }
                server_locked.send_motion_update(target_vector, object_id, motion_data);
            }else{
                eprintln!("No object with ID {} found inside server's synced players", object_id);
            }
            
        } else if self.client.is_some(){
            let client_locked = self.client.as_ref().unwrap().lock().unwrap();
            
            if let Some(wrapper) = client_locked.get_synced_players().lock().unwrap().get_mut(&object_id){
                wrapper.position_data = (motion_data.x, motion_data.y);
                wrapper.speed_data = (motion_data.x_speed, motion_data.y_speed);
                
                let mut message: HashMap<String, ObjectType> = HashMap::new();

                message.insert("goal".to_string(), ObjectType::StringMsg("object_pos_update".to_string()));
                message.insert("object_id".to_string(), ObjectType::Integer(wrapper.object_id));
                message.insert("motion_data".to_string(), ObjectType::MotionData(motion_data));

                if let Err(e) = client_locked.send_to_receive_thread(message) {
                    eprintln!("Failed to send message: {}", e);
                }
            }else{
                eprintln!("No object with ID {} found inside client's synced players", object_id);
            }
        }else{ panic!("Game Handle has not been initialized properly"); }
    }


    pub fn send_rpc(&self, call_container: RpcCallContainer) {
        if self.server.is_some(){
            let server_locked = self.server.as_ref().unwrap().lock().unwrap();

            let socket_map = server_locked.get_user_map();
            let mut target_vector: Vec<&SocketAddr> = Vec::new();

            for (_, target) in socket_map.iter(){
                
                if target.to_string() != "127.0.0.1:13882" {
                    target_vector.push(target);
                }
            }
            server_locked.send_rpc(target_vector, call_container);
        }else if self.client.is_some(){
            let client_locked = self.client.as_ref().unwrap().lock().unwrap();

            let mut message: HashMap<String, ObjectType> = HashMap::new();

            message.insert("goal".to_string(), ObjectType::StringMsg("rpc_call".to_string()));
            message.insert("rpc_data".to_string(), ObjectType::RpcCall(call_container));

            if let Err(e) = client_locked.send_to_receive_thread(message) {
                eprintln!("Failed to send message: {}", e);
            }

        }else{ panic!("Game Handle has not been initialized properly"); }
    }


    pub fn get_world(&self) -> Arc<Mutex<World>> {
        if self.server.is_some(){
            self.server.as_ref().unwrap().lock().unwrap().get_world()
        }else if self.client.is_some(){
            self.client.as_ref().unwrap().lock().unwrap().get_world()
        }else{ panic!("Game Handle has not been initialized properly"); }
    }


    pub fn get_network_wrappers(&self) -> Arc<Mutex<HashMap<i32, DataWrapper>>> {
        if self.server.is_some(){
            self.server.as_ref().unwrap().lock().unwrap().get_synced_players()
        }else if self.client.is_some(){
            self.client.as_ref().unwrap().lock().unwrap().get_synced_players()
        }else{ panic!("Game Handle has not been initialized properly"); }
    }


    pub fn get_player_wrapper_map(&mut self) -> Arc<Mutex<HashMap<i32,Player>>> {
        Arc::clone(&self.player_wrapper_map)
    }


    pub fn get_personal_id(&self) -> i32{
        self.personal_id
    }


    fn launch_server(&mut self, world: Arc<Mutex<World>>) -> Result<(), std::io::Error> {
        let server = Arc::new(Mutex::new(Server::new(world, Arc::clone(&self.player_wrapper_map))?));
        server.lock().unwrap().start(Arc::clone(&server));
        self.server = Some(server);
        
        println!("{}", "\n═════════════════════════════".bold().bright_cyan());
        println!("{}", "  Server is up and running!".bold().bright_green());
        println!("{}", "═════════════════════════════".bold().bright_cyan());

        Ok(())
    }


    fn launch_client(&mut self, server_ip: String, world: Arc<Mutex<World>>) -> Result<(), String> {
        if server_ip.is_empty() {
            return Err("No IP address provided".to_string());
        }
        
        let client = Arc::new(Mutex::new(Client::new(server_ip, world,Arc::clone(&self.player_wrapper_map)).unwrap()));
        {
            client.lock().unwrap().start(Arc::clone(&client));
        }
        for _ in 1..6{
            {
                
                let mut connect_message = HashMap::new();
                connect_message.insert(String::from("goal"), ObjectType::StringMsg(String::from("sync")));

                let _ = client.lock().unwrap().send_to_receive_thread(connect_message);
                                
            }
            thread::sleep(Duration::from_millis(200));
            let new_id = client.lock().unwrap().get_personal_id();
            if new_id!=0 {
                self.personal_id = new_id;
                println!("{}", "\n═════════════════════════════".bold().bright_cyan());
                println!("{}", "  Client is up and running!".bold().bright_green());
                println!("{}", "═════════════════════════════".bold().bright_cyan());
                break;
            }
        }
        if client.lock().unwrap().get_personal_id() == 0{
            panic!("Could not connect to the server and receive an ID");
        }
        self.client = Some(client);
        self.request_synced_players();
        Ok(())
    }

    
    pub fn construct_client(server_ip: String, world: Arc<Mutex<World>>) -> Arc<Mutex<Self>> {
        let handle = GameHandle {
            client: None,
            server: None,
            player_wrapper_map: Arc::new(Mutex::new(HashMap::new())),
            personal_id: 0,
        };
        let handle_mutex = Arc::new(Mutex::new(handle));
        handle_mutex.lock().unwrap().launch_client(server_ip,world).expect("Failed to launch client");
        handle_mutex
    }


    pub fn construct_server(world: Arc<Mutex<World>>) -> Arc<Mutex<Self>> {
        let handle = GameHandle {
            client: None,
            server: None,
            player_wrapper_map: Arc::new(Mutex::new(HashMap::new())),
            personal_id: 0,
        };
        let handle_mutex = Arc::new(Mutex::new(handle));
        handle_mutex.lock().unwrap().launch_server(Arc::clone(&world)).expect("Failed to launch server");
        handle_mutex
    }
}