use crate::COMMS_PORT;
use std::collections::HashMap;
use std::net::{UdpSocket, SocketAddr};
use std::io::Result;
use std::sync::{Arc,Mutex};
use std::thread;
use std::str::FromStr;

pub struct Server {
    socket: UdpSocket,
    user_map: HashMap<i32,SocketAddr>,
}


fn receive_message(socket: &UdpSocket) -> Result<()> {
    let mut buffer = [0u8; 1024];
    let (size, sender) = socket.recv_from(&mut buffer)?;

    println!("Received packet from {}: {}", sender, String::from_utf8_lossy(&buffer[..size]));
    Ok(())
}


impl Server {
    pub fn new() -> Result<Server> {
        let server_address = format!("127.0.0.1:{}",COMMS_PORT);
        let socket = UdpSocket::bind(server_address.clone())?;

        Ok(Server {
            socket: socket,
            user_map: HashMap::new(),
        })
    }

    pub fn send_message(&self, message: &[u8], client_address: SocketAddr) -> Result<()> {
        self.socket.send_to(message, client_address)?;
        println!("Sent packet to {}", client_address);
        Ok(())
    }

    pub fn start(&mut self) {
        let arc_socket = Arc::new(Mutex::new(self.socket.try_clone().expect("Failed to clone socket")));
        let socket_clone = Arc::clone(&arc_socket);
        let _receive_thread = thread::spawn(move || {
            let socket_copy = socket_clone.lock().unwrap();
            loop {
                receive_message(&socket_copy).expect("Failed to receive message");
            }
        });
        
        self.user_map.insert(-1, SocketAddr::from_str("127.0.0.1:").unwrap());
    }
}

