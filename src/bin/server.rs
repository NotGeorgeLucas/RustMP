use std::net::{UdpSocket, SocketAddr};
use std::io::Result;


fn send_message(socket: &UdpSocket, message: &[u8], server_address: SocketAddr) -> Result<()> {
    socket.send_to(message, server_address)?;
    println!("Sent packet to {}", server_address);
    Ok(())
}

fn receive_message(socket: &UdpSocket) -> Result<()> {
    let mut buffer = [0u8; 1024];
    let (size, sender) = socket.recv_from(&mut buffer)?;
    println!("Received packet from {}: {}", sender, String::from_utf8_lossy(&buffer[..size]));
    Ok(())
}

fn main() -> std::io::Result<()> {
    let server_address: SocketAddr = "192.168.1.100:34254".parse().expect("Invalid address");
    let socket = UdpSocket::bind("0.0.0.0:0")?;

    let message = b"Hello, server!";
    send_message(&socket, message, server_address)?;
    loop{receive_message(&socket)?;}
}
