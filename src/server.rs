use crate::game_handle::GameHandle;
use crate::COMMS_PORT;
use crate::message::{Message,ObjectType};
use crate::player::Player;
use std::collections::HashMap;
use std::net::{UdpSocket, SocketAddr};
use std::io::{Result,ErrorKind};
use std::sync::{Arc,Mutex};
use std::thread;
use std::str::FromStr;
use std::time::Duration;
use colored::*;

#[derive(Clone)]
pub struct Server {
    socket: Arc<Mutex<UdpSocket>>,
    user_map: HashMap<i32,SocketAddr>,
    synced_players: HashMap<i32, Player>, //    🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥
    game_handle: Arc<Mutex<GameHandle>>,
}




impl Server {
    pub fn new(game_handle_mutex: Arc<Mutex<GameHandle>>) -> Result<Server> {
        let server_address = format!("0.0.0.0:{}",COMMS_PORT);
        let socket = UdpSocket::bind(server_address.clone()).unwrap();
        socket.set_nonblocking(true)?;

        Ok(Server {
            socket: Arc::new(Mutex::new(socket)),
            user_map: HashMap::new(),
            synced_players: HashMap::new(),
            game_handle: game_handle_mutex,
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

    pub fn id_to_socket(&self, id: i32) -> Option<SocketAddr> {
        if let Some(socket_addr) = self.user_map.get(&id) {
            Some(*socket_addr)
        } else {
            None
        }
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
                        if let Some(new_player) = received_map.get("player"){
                            let new_id = self.gen_new_player_id();
                            match new_player {
                                ObjectType::Player(np) => {
                                    self.synced_players.insert(new_id,*np);
                                    response_map.insert(String::from("goal"), ObjectType::StringMsg(String::from("ret_sync_players")));
                                    response_map.insert(String::from("players"), ObjectType::PlayerMap(self.synced_players.clone()));
                                },
                                _ => {
                                    eprintln!("plaer field not a player struct");
                                }
                            }
                        }else{
                            eprintln!("player field not found in get_sync_objects message");
                        }
                    },
                    "add_player" => {
                        let mut new_id: Option<i32> = None;
                        let mut new_player: Option<Player> = None;
                        if let Some(id) = received_map.get("id"){
                            match id{
                                ObjectType::Integer(val) => {new_id = Some(*val);},
                                _ => {eprintln!("Object id not i32")},
                            }
                        }else{
                            eprintln!("ID field not found in message");
                        }
                        if let Some(pl) = received_map.get("player"){
                            match pl{
                                ObjectType::Player(pl_val) => {new_player = Some(*pl_val);},
                                _ => {eprintln!("Player not a valid player struct")},
                            }
                        }else{
                            eprintln!("ID field not found in message");
                        }
                        if new_id.is_some() && new_player.is_some(){
                            self.synced_players.insert(new_id.unwrap(), new_player.unwrap());
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
        println!("{}", "═════════════════════════════".bold().bright_cyan());
        println!("{}", "  Server is up and running!".bold().bright_green());
        println!("{}", "═════════════════════════════".bold().bright_cyan());
        self.user_map.insert(-1, SocketAddr::from_str(format!("127.0.0.1:{}",COMMS_PORT).as_str()).unwrap());
    }
}

