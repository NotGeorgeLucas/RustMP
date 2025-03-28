use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::client::Client;
use crate::server::Server;
use crate::player::Player;
use crate::network_sync::NetworkSync;
use crate::message::ObjectType;
use bevy::prelude::Resource;



#[derive(Resource)]
pub struct GameHandle {
    client: Option<Arc<Mutex<Client>>>,
    server: Option<Arc<Mutex<Server>>>,
}

impl GameHandle {

    pub fn add_player(&mut self, mut player: Player){
        if let Some(server_arc) = &self.server {
            let server_lock = server_arc.lock().unwrap();
            

            let object_id = server_lock.gen_new_player_id();
            player.set_object_id(object_id);
            let mut message= HashMap::new();
            message.insert("goal".to_string(), ObjectType::StringMsg("add_player".to_string()));
            message.insert("id".to_string(), ObjectType::Integer(object_id));
            message.insert("player".to_string(), ObjectType::Player(player));

            if let Some(target) = server_lock.id_to_socket(-1){
                if let Err(e) = server_lock.send_message(&message, target){
                    eprintln!("Could not send message to self: {}",e);
                }
            }

        } else if let Some(client_arc) = &self.client {
            let client_lock = client_arc.lock().unwrap();
            
            player.set_object_id(client_lock.initial_add_player(player).expect("Could not add the current player"));
        }
    }

    fn launch_server(&mut self, self_mutex: Arc<Mutex<Self>>) -> Result<(), std::io::Error> {
        let server = Arc::new(Mutex::new(Server::new(self_mutex)?));
        server.lock().unwrap().start(Arc::clone(&server));
        self.server = Some(server);
        Ok(())
    }

    fn launch_client(&mut self, server_ip: String, self_mutex: Arc<Mutex<Self>>) -> Result<(), String> {
        if server_ip.is_empty() {
            return Err("No IP address provided".to_string());
        }

        let client = Arc::new(Mutex::new(Client::new(server_ip,self_mutex).unwrap()));
        client.lock().unwrap().start(Arc::clone(&client));
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