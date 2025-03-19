use crate::COMMS_PORT;
use crate::message::Message;
use std::collections::HashMap;
use std::net::{UdpSocket, SocketAddr};
use std::io::Result;
use std::sync::{Arc,Mutex};
use std::thread;
use std::str::FromStr;

#[derive(Clone)]
pub struct Server {
    socket: Arc<Mutex<UdpSocket>>,
    user_map: HashMap<i32,SocketAddr>,
}




impl Server {
    pub fn new() -> Result<Server> {
        let server_address = format!("0.0.0.0:{}",COMMS_PORT);
        let socket = UdpSocket::bind(server_address.clone()).unwrap();

        Ok(Server {
            socket: Arc::new(Mutex::new(socket)),
            user_map: HashMap::new(),
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


    fn process_message(&mut self,message_received: &Message,client_address:SocketAddr) -> HashMap<String,String>{
        let mut response_map = HashMap::new();
        let received_map = message_received.get_message_map();
        println!("{:?}",received_map);
        if received_map.contains_key("goal"){
            match received_map.get("goal"){
                Some(goal) => match goal.as_str() {
                    "sync" => {
                        response_map.insert(String::from("goal"), String::from("confirm connect"));
                        let new_id = self.gen_new_id();
                        self.user_map.insert(new_id, client_address);
                        response_map.insert(String::from("id"), String::from(format!("{}",new_id)));
                    },
                    _ =>{
                        println!("Unknown message type");
                    }
                }
                None =>{
                    println!("Goal field empty");
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


    pub fn send_message(&self,message: &HashMap<String,String>,target:SocketAddr) -> Result<()> {
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
                let mut locked = mut_ref.lock().unwrap();
                if let Err(e) = locked.receive_message() {
                    eprintln!("Failed to receive message: {:?}", e);
                }
            }
        });

        self.user_map.insert(-1, SocketAddr::from_str("127.0.0.1:8080").unwrap());
    }
}

