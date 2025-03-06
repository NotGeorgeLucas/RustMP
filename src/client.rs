use std::net::{UdpSocket, SocketAddr};
use std::io::Result;
use std::str::FromStr;
use std::thread;
use std::sync::{Arc,Mutex};



pub struct Client{
    server_address: SocketAddr,
    socket: UdpSocket,
}


fn receive_message(socket: &UdpSocket) -> Result<()> {
    let mut buffer = [0u8; 1024];
    let (size, sender) = socket.recv_from(&mut buffer)?;
    println!("Received packet from {}: {}", sender, String::from_utf8_lossy(&buffer[..size]));
    Ok(())
}


impl Client{
    pub fn new(server_address_ip: String) -> Result<Client> {
        let mut server_address = server_address_ip.clone();
        server_address = server_address;
        server_address.push_str(":34254");
        let socket = UdpSocket::bind("0.0.0.0:0")?;

        Ok(Client{
            server_address:SocketAddr::from_str(&server_address).unwrap(),
            socket:socket
        })
    }


    pub fn send_message(&self,message: &[u8]) -> Result<()> {
        self.socket.send_to(message, self.server_address)?;
        println!("Sent packet to {}", self.server_address);
        Ok(())
    }

    

    pub fn start(&self) {
        let arc_socket = Arc::new(Mutex::new(self.socket.try_clone().expect("Failed to clone socket")));
        let socket_clone = Arc::clone(&arc_socket);
        
        let _receive_thread = thread::spawn(move || {
            let socket_copy = socket_clone.lock().unwrap();
            loop {
                receive_message(&socket_copy).expect("Failed to receive message");
            }
        });

        receive_message(&self.socket).expect("Failed to receive message");

        let reply_message = b"Hello from server!";
        self.send_message(reply_message).expect("Failed to send message");
    }
}