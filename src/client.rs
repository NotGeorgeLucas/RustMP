use crate::message::Message;
use std::collections::HashMap;
use std::net::{UdpSocket, SocketAddr};
use std::io::Result;
use std::str::FromStr;
use std::thread;
use std::sync::{Arc,Mutex};
use std::time::Duration;


#[derive(Clone)]
pub struct Client {
    server_address: SocketAddr,
    socket: Arc<Mutex<UdpSocket>>,
    personal_id:i32,
}



impl Client{
    pub fn new(server_address_ip: String) -> Result<Client> {
        let mut server_address = server_address_ip.clone();
        server_address = server_address;
        server_address.push_str(":34254");
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        
        Ok(Client{
            server_address:SocketAddr::from_str(&server_address).unwrap(),
            socket:Arc::new(Mutex::new(socket)),
            personal_id:0,
        })
    }
    

    fn process_message(&mut self,message_received: &Message) -> HashMap<String,String>{

        let mut response_map = HashMap::new();
        let received_map = message_received.get_message_map();
        if received_map.contains_key("goal"){
            match received_map.get("goal"){
                Some(goal) => match goal.as_str() {
                    "confirm connect" => {
                        self.personal_id = message_received.get_message_map().get("id").unwrap().parse().expect("Invalid i32 id");
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
                let response_map = self.process_message(&decoded);

                if !response_map.is_empty(){
                    self.send_message(&response_map)?;
                }
                println!("Sent response to {}", sender);
            }
            Err(e) =>{
                println!("Failed to decode message: {}", e);
            }
        }
        Ok(())
    }


    pub fn send_message(&self,message: &HashMap<String,String>) -> Result<()> {
        let message_struct: Message = Message::new(-1, message.clone()).expect("Message malformed");

        let message_bytes = bincode::serialize(&message_struct).unwrap();
        
        self.socket.lock().unwrap().send_to(&message_bytes, self.server_address)?;
        println!("Sent packet to {}", self.server_address);
        Ok(())
    }

    

    pub fn start(&self, self_mutex: Arc<Mutex<Self>>) {
        let mut_ref = Arc::clone(&self_mutex);

        let _receive_thread = thread::spawn(move || {
            loop {
                let mut locked = mut_ref.lock().unwrap();
                locked.receive_message().expect("Failed to receive message");
            }
        });
        
        let mut connect_message = HashMap::new();
        connect_message.insert(String::from("goal"), String::from("sync"));
        for _ in [1..5]{
            self.send_message(&connect_message).expect("Sending message failed");
            thread::sleep(Duration::from_secs(2));
            if self.personal_id!=0 {println!("Connected!"); break;}
        }
    }
}