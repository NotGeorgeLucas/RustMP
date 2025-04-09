use crate::message::{Message,ObjectType};
use crate::network_sync::NetworkSync;
use crate::game_handle::GameHandle;
use crate::player::Player;
use crate::CLIENT_PORT;
use core::panic;
use std::collections::HashMap;
use std::net::{UdpSocket, SocketAddr};
use std::io::Result;
use std::str::FromStr;
use std::thread;
use std::sync::{Arc,Mutex};
use std::time::Duration;
use std::io::ErrorKind;
use bevy::tasks::futures_lite::io;
use colored::*;


#[derive(Clone)]
pub struct Client {
    server_address: SocketAddr,
    socket: Arc<Mutex<UdpSocket>>,
    personal_id:i32,
    synced_players: HashMap<i32, Player>,
    game_handle: Arc<Mutex<GameHandle>>,
}



impl Client{
    pub fn new(server_address_ip: String, game_handle_mutex: Arc<Mutex<GameHandle>>) -> Result<Client> {
        let mut server_address = server_address_ip.clone();
        server_address = server_address;
        let socket = UdpSocket::bind(format!("0.0.0.0:{}",CLIENT_PORT))?;
        println!("Client bound to: {:?}", socket.local_addr()?);
        socket.set_nonblocking(true)?;
        
        Ok(Client{
            server_address:SocketAddr::from_str(&server_address).unwrap(),
            socket:Arc::new(Mutex::new(socket)),
            personal_id:0,
            synced_players: HashMap::new(),
            game_handle: game_handle_mutex,
        })
    }
    

    fn process_message(&mut self,message_received: &Message) -> HashMap<String,ObjectType>{

        let mut response_map = HashMap::new();
        let received_map = message_received.get_message_map();
        println!("{:?}",received_map);
        if received_map.contains_key("goal"){
            match received_map.get("goal"){
                Some(ObjectType::StringMsg(goal)) => match goal.as_str() {
                    "confirm connect" => {
                        if let Some(ObjectType::Integer(new_id)) = received_map.get("id") {
                            self.personal_id = *new_id;
                        }else{
                            eprintln!("ID not a valid i32")
                        }
                    },
                    "ret_sync_players" => {
                        if let Some(ObjectType::PlayerMap(players)) = received_map.get("players"){
                            self.synced_players = players.clone();
                        }else{
                            eprintln!("Invalid player map return type");
                        }
                    },
                    _ =>{
                        println!("Unknown message type");
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
        if let Ok(message_struct) = Message::new(-1, message.clone()){
            let message_bytes = bincode::serialize(&message_struct).unwrap();
            
            self.socket.lock().unwrap().send_to(&message_bytes, self.server_address)?;
            println!("{}",format!("Sent packet to {}", self.server_address).bold().green());
        }else{
            eprintln!("Failed to construct message");
        }

        Ok(())
    }


    pub fn initial_add_player(&self, player: Player) -> Result<i32> {
        
        let mut message= HashMap::new();

        message.insert("goal".to_string(), ObjectType::StringMsg("get_sync_players".to_string()));
        message.insert("player".to_string(), ObjectType::Player(player));

        for _ in 1..6{
            if let Err(e) = self.send_message(&message){
                eprintln!("Could not send message to self: {}",e);
            }
            thread::sleep(Duration::from_secs(2));
            if !self.synced_players.is_empty(){
                for(id, player) in self.synced_players.iter(){
                    if player.get_owner() == self.personal_id{
                        return Ok(*id);
                    }
                }
            }
        }
        Err(io::Error::new(io::ErrorKind::Other, "Could not receive player map in 5 tries"))
    }

    
    pub fn get_synced_players(&self) -> HashMap<i32, Player> { 
        self.synced_players.clone()
    }


    pub fn start(&self, self_mutex: Arc<Mutex<Self>>) {
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
        
        let mut connect_message = HashMap::new();
        connect_message.insert(String::from("goal"), ObjectType::StringMsg(String::from("sync")));
        for _ in 1..6{
            if let Err(e) = self.send_message(&connect_message){
                eprintln!("Sending message failed: {:?}", e);
            }
            thread::sleep(Duration::from_secs(2));
            if self.personal_id!=0 {
                println!("{}", "═════════════════════════════".bold().bright_cyan());
                println!("{}", "  Client is up and running!".bold().bright_green());
                println!("{}", "═════════════════════════════".bold().bright_cyan());
                return;
            }
        }
        panic!("Could not connect to the server and receive an ID");
        
    }
}