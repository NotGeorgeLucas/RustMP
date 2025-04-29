use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use crate::client::Client;
use crate::server::Server;
use crate::player::{DataWrapper, Player};
use crate::network_sync::NetworkSync;
use crate::message::ObjectType;
use colored::*;
use macroquad_platformer::World;



#[derive(Clone)]
pub struct GameHandle {
    client: Option<Arc<Mutex<Client>>>,
    server: Option<Arc<Mutex<Server>>>,
    player_wrapper_map: Arc<Mutex<HashMap<i32, Player>>>,
}

impl GameHandle {

    pub fn add_player(&mut self, mut player: Player) -> Option<i32> {
        if let Some(server_arc) = &self.server {
            let mut server_lock = server_arc.lock().unwrap();
    
            let object_id = server_lock.gen_new_player_id();
            player.set_object_id(object_id);
            
            let mut player_wrapper_map = self.player_wrapper_map.lock().unwrap();
            player_wrapper_map.insert(object_id, player);

            server_lock.add_player(player,-1);

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
                drop(client_lock);
                if new_player_id.is_some(){
                    player.set_object_id(new_player_id.unwrap());
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
                        let player = Player::construct_from_wrapper(*wrapper, &mut world);
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


    pub fn get_world(&self) -> Arc<Mutex<World>> {
        if self.server.is_some(){
            self.server.as_ref().unwrap().lock().unwrap().get_world()
        }else if self.client.is_some(){
            self.client.as_ref().unwrap().lock().unwrap().get_world()
        }else{ panic!("Game Handle has not been initialized properly"); }
    }


    pub fn get_player_wrappers(&self) -> Arc<Mutex<HashMap<i32, DataWrapper>>> {
        if self.server.is_some(){
            self.server.as_ref().unwrap().lock().unwrap().get_synced_players()
        }else if self.client.is_some(){
            self.client.as_ref().unwrap().lock().unwrap().get_synced_players()
        }else{ panic!("Game Handle has not been initialized properly"); }
    }


    pub fn get_player_wrapper_map(&mut self) -> Arc<Mutex<HashMap<i32,Player>>> {
        Arc::clone(&self.player_wrapper_map)
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
            if client.lock().unwrap().personal_id!=0 {
                println!("{}", "\n═════════════════════════════".bold().bright_cyan());
                println!("{}", "  Client is up and running!".bold().bright_green());
                println!("{}", "═════════════════════════════".bold().bright_cyan());
                break;
            }
        }
        if client.lock().unwrap().personal_id == 0{
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
        };
        let handle_mutex = Arc::new(Mutex::new(handle));
        handle_mutex.lock().unwrap().launch_server(Arc::clone(&world)).expect("Failed to launch server");
        handle_mutex
    }
}