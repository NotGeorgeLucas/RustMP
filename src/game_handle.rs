use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use crate::client::Client;
use crate::server::Server;
use crate::player::Player;
use crate::network_sync::NetworkSync;
use crate::message::ObjectType;
use crate::player_spawner;
use bevy::prelude::{Resource,AssetServer,Res,Commands};
use colored::*;




#[derive(Resource)]
pub struct GameHandle {
    client: Option<Arc<Mutex<Client>>>,
    server: Option<Arc<Mutex<Server>>>,
}

impl GameHandle {

    pub fn add_player(
        &mut self,
        mut player: Player,
        commands: &mut Commands, 
        _asset_server: &Res<AssetServer>, 
        owner_id: i32,
    ) {
        if let Some(server_arc) = &self.server {
            let server_lock = server_arc.lock().unwrap();
    
            let object_id = server_lock.gen_new_player_id();
            player.set_object_id(object_id);
            let mut message = HashMap::new();
            message.insert("goal".to_string(), ObjectType::StringMsg("add_player".to_string()));
            message.insert("id".to_string(), ObjectType::Integer(object_id));
            message.insert("player".to_string(), ObjectType::Player(player));
    
            if let Some(target) = server_lock.id_to_socket(-1) {
                if let Err(e) = server_lock.send_message(&message, target) {
                    eprintln!("Could not send message to self: {}", e);
                }
            }
        } else if let Some(client_arc) = &self.client {
            let mut found_id = None;

            for _ in 1..6 {
                {
                    let client_lock = client_arc.lock().unwrap();

                    let mut message = HashMap::new();
                    message.insert("goal".to_string(), ObjectType::StringMsg("get_sync_players".to_string()));
                    message.insert("player".to_string(), ObjectType::Player(player.clone()));

                    if let Err(e) = client_lock.send_to_receive_thread(message) {
                        eprintln!("Could not send message to self: {}", e);
                    }
                }

                thread::sleep(Duration::from_secs(2));

                let client_lock = client_arc.lock().unwrap();
                println!("Player map from game handle pov: {:?}",client_lock.get_synced_players());
                if !client_lock.get_synced_players().is_empty() {
                    for (id, synced_player) in client_lock.get_synced_players().iter() {
                        if synced_player.get_owner() == 0 {
                            found_id = Some(*id);
                            break;
                        }
                    }
                }

                if found_id.is_some() {
                    break;
                }
            }

            let own_id = found_id.expect("Could not add the current player");
            player.set_object_id(own_id);

            let player_map = {
                let client_lock = client_arc.lock().unwrap();
                client_lock.get_synced_players()
            };

            for (_, synced_player) in player_map.iter() {
                if synced_player.get_object_id() != own_id {
                    player_spawner::spawn_player(commands, _asset_server, -1, self);
                }
            }
        }
    }

    fn launch_server(&mut self, self_mutex: Arc<Mutex<Self>>) -> Result<(), std::io::Error> {
        let server = Arc::new(Mutex::new(Server::new(self_mutex)?));
        server.lock().unwrap().start(Arc::clone(&server));
        self.server = Some(server);
        
        println!("{}", "═════════════════════════════".bold().bright_cyan());
        println!("{}", "  Server is up and running!".bold().bright_green());
        println!("{}", "═════════════════════════════".bold().bright_cyan());

        Ok(())
    }

    fn launch_client(&mut self, server_ip: String, self_mutex: Arc<Mutex<Self>>) -> Result<(), String> {
        if server_ip.is_empty() {
            return Err("No IP address provided".to_string());
        }
        
        let client = Arc::new(Mutex::new(Client::new(server_ip,self_mutex).unwrap()));
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
                println!("{}", "═════════════════════════════".bold().bright_cyan());
                println!("{}", "  Client is up and running!".bold().bright_green());
                println!("{}", "═════════════════════════════".bold().bright_cyan());
                break;
            }
        }
        if client.lock().unwrap().personal_id == 0{
            panic!("Could not connect to the server and receive an ID");
        }
        self.client = Some(client);
        Ok(())
    }


    pub fn construct_client(server_ip: String) -> Arc<Mutex<Self>> {
        let handle = GameHandle {
            client: None,
            server: None,
        };
        let handle_mutex = Arc::new(Mutex::new(handle));
        handle_mutex.lock().unwrap().launch_client(server_ip,Arc::clone(&handle_mutex)).expect("Failed to launch client");
        handle_mutex
    }


    pub fn construct_server() -> Arc<Mutex<Self>> {
        let handle = GameHandle {
            client: None,
            server: None,
        };
        let handle_mutex = Arc::new(Mutex::new(handle));
        handle_mutex.lock().unwrap().launch_server(Arc::clone(&handle_mutex)).expect("Failed to launch server");
        handle_mutex
    }
}