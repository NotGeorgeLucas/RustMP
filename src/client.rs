use crate::message::{Message,ObjectType};
use crate::COMMS_PORT;
use std::collections::HashMap;
use std::net::{UdpSocket, SocketAddr};
use std::io::Result;
use std::str::FromStr;
use std::thread;
use std::sync::{Arc,Mutex};
use std::time::Duration;
use colored::*;
use crate::player::Player;


#[derive(Clone)]
pub struct Client {
    server_address: SocketAddr,
    socket: Arc<Mutex<UdpSocket>>,
    personal_id:i32,
    pub synced_players: HashMap<i32, Player>,
}



impl Client{
    pub fn new(server_address_ip: String) -> Result<Client> {
        let mut server_address = server_address_ip.clone();
        server_address = server_address;
        server_address = format!("{}:{}",server_address,COMMS_PORT);
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        
        Ok(Client{
            server_address:SocketAddr::from_str(&server_address).unwrap(),
            socket:Arc::new(Mutex::new(socket)),
            personal_id:0,
            synced_players: HashMap::new(),
        })
    }
    

    fn process_message(&mut self,message_received: &Message) -> HashMap<String,ObjectType>{

        let mut response_map = HashMap::new();
        let received_map = message_received.get_message_map();
        if received_map.contains_key("goal"){
            match received_map.get("goal"){
                Some(ObjectType::StringMsg(goal)) => match goal.as_str() {
                    "confirm connect" => {
                        if let Some(ObjectType::Integer(new_id)) = message_received.get_message_map().get("id") {
                            self.personal_id = *new_id;
                        }else{
                            eprint!("ID not a valid i32")
                        }
                    },
                    "ret_sync_players" => {
                        if let Some(ObjectType::PlayerMap(players)) = message_received.get_message_map().get("players"){
                            self.synced_players = players.clone();
                        }else{
                            eprintln!("Invalid player map return type! Resending player list request.");
                            response_map.insert(String::from("goal"), ObjectType::StringMsg(String::from("get_sync_players")));
                        }
                    },
                    _ =>{
                        println!("Unknown message type");
                    }
                }
                None =>{
                    println!("Goal field empty");
                }
                _ =>{
                    eprint!("Invalid goal type!")
                }
            }
        }
        response_map
    }

    
    fn receive_message(&mut self) -> Result<()> {
        let mut buffer = [0u8; 1024];
        let (size, sender) = self.socket.lock().unwrap().recv_from(&mut buffer)?;
    
        match bincode::deserialize::<Message>(&buffer[..size]){
            Ok(decoded) => {
                let response_map = self.process_message(&decoded);

                if !response_map.is_empty(){
                    self.send_message(&response_map)?;
                    println!("{}",format!("Sent response to {}", sender).bold().green());
                }
            }
            Err(e) =>{
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

    

    pub fn start(&self, self_mutex: Arc<Mutex<Self>>) {
        let mut_ref = Arc::clone(&self_mutex);

        let _receive_thread = thread::spawn(move || {
            loop {
                let mut locked = mut_ref.lock().unwrap();
                if let Err(e) = locked.receive_message() {
                    eprintln!("Failed to receive message: {:?}", e);
                }
            }
        });
        
        let mut connect_message = HashMap::new();
        connect_message.insert(String::from("goal"), ObjectType::StringMsg(String::from("sync")));
        for _ in [1..5]{
            if let Err(e) = self.send_message(&connect_message){
                eprintln!("Sending message failed: {:?}", e);
            }
            thread::sleep(Duration::from_secs(2));
            if self.personal_id!=0 {
                println!("{}", "═════════════════════════════".bold().bright_cyan());
                println!("{}", "  Client is up and running!".bold().bright_green());
                println!("{}", "═════════════════════════════".bold().bright_cyan());
                break;
            }
        }
        let mut getter_message = HashMap::new();
        getter_message.insert(String::from("goal"), ObjectType::StringMsg(String::from("get_sync_players")));
        if let Err(e) = self.send_message(&getter_message){
            eprintln!("Sending message failed: {:?}", e);
        }
        
    }
}