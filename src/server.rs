use crate::COMMS_PORT;
use crate::message::{Message,ObjectType};
use crate::player::Player;
use std::collections::HashMap;
use std::net::{UdpSocket, SocketAddr};
use std::io::Result;
use std::sync::{Arc,Mutex};
use std::thread;
use std::str::FromStr;
use std::sync::mpsc::{self, Receiver,Sender};
use colored::*;


pub struct Server {
    socket: Arc<Mutex<UdpSocket>>,
    user_map: HashMap<i32,SocketAddr>,
    pub synced_players: HashMap<i32, Player>, //    🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥
    pub tx: Option<Sender<Message>>,
}




impl Server {
    pub fn new() -> Result<Server> {
        let server_address = format!("0.0.0.0:{}",COMMS_PORT);
        let socket = UdpSocket::bind(server_address.clone()).unwrap();

        Ok(Server {
            socket: Arc::new(Mutex::new(socket)),
            user_map: HashMap::new(),
            synced_players: HashMap::new(),
            tx: None
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
        loop {
            if !self.synced_players.contains_key(&key){
                break;
            }
            key+=1;
        }
        key
    }

    fn process_message(&mut self,message_received: &Message,client_address:SocketAddr) -> HashMap<String,ObjectType>{
        let mut response_map = HashMap::new();
        let received_map = message_received.get_message_map();
        println!("{:?}",received_map);
        if received_map.contains_key("goal"){
            match received_map.get("goal"){
                Some(ObjectType::StringMsg(goal)) => match goal.as_str() {
                    "sync" => {
                        response_map.insert(String::from("goal"), ObjectType::StringMsg(String::from("confirm connect")));
                        let new_id = self.gen_new_id();
                        self.user_map.insert(new_id, client_address);
                        response_map.insert(String::from("id"), ObjectType::Integer(new_id));
                    },
                    "get_sync_objects" => {
                        response_map.insert(String::from("goal"), ObjectType::StringMsg(String::from("ret_sync_players")));
                        response_map.insert(String::from("players"), ObjectType::PlayerMap(self.synced_players.clone()));
                    },
                    "player_join" => {
                        if let Some(ObjectType::Player(player)) = message_received.get_message_map().get("player"){
                            let player = player.clone();
                            let new_key = self.gen_new_player_id();
                            self.synced_players.insert(new_key, player);
                        }else{
                            eprintln!("Invalid message received for player_join");
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
                    eprintln!("Invalid goal type!")
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
                let response_map = self.process_message(&decoded,sender);
                if !response_map.is_empty(){
                    self.send_message(&response_map, sender)?;
                    println!("Sent response: {:?}",response_map);
                }
            }
            Err(e) =>{
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

    pub fn id_to_addr(&self,id: i32) -> SocketAddr{
        *self.user_map.get(&id).unwrap()
    }

    pub fn start(&mut self, self_mutex: Arc<Mutex<Self>>) {
        let mut_ref = Arc::clone(&self_mutex);
        let worker_clone: Arc<Mutex<Self>> = Arc::clone(&mut_ref);
        let (tx,rx): (Sender<Message>, Receiver<Message>)= mpsc::channel();
        self.tx = Some(tx);
        let rx_clone:Arc<Mutex<Receiver<Message>>> = Arc::clone(&Arc::new(Mutex::new(rx)));

        let _receive_thread = thread::spawn(move || {
            loop {
                let mut locked = worker_clone.lock().unwrap();
                let rx_locked = rx_clone.lock().unwrap();

                match rx_locked.try_recv() {
                    Ok(msg) => {
                        if let Err(e) = locked.send_message(&msg.get_message_map().clone(), locked.id_to_addr(-1)){
                            eprintln!("Failed to send message: {}",e);
                        }
                    }
                    Err(_) => eprintln!("Failed in receiving inner channel communication message"),
                }

                if let Err(e) = locked.receive_message() {
                    eprintln!("Failed to receive message: {:?}", e);
                }
            }
        });
        println!("{}", "═════════════════════════════".bold().bright_cyan());
        println!("{}", "  Server is up and running!".bold().bright_green());
        println!("{}", "═════════════════════════════".bold().bright_cyan());
        self.user_map.insert(-1, SocketAddr::from_str(format!("127.0.0.1:{}",COMMS_PORT).as_str()).unwrap());
    }
}

