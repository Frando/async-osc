use async_osc::{prelude::*, Error, OscPacket, OscSocket, OscType, Result};
use async_std::stream::StreamExt;

#[async_std::main]
async fn main() -> Result<()> {
    let mut socket = OscSocket::bind("localhost:5050").await?;

    // Open a second socket to send a test message.
    async_std::task::spawn(async move {
        let socket = OscSocket::bind("localhost:0").await?;
        socket.connect("localhost:5050").await?;
        socket
            .send(("/volume", (0.9f32, "foo".to_string())))
            .await?;
        Ok::<(), Error>(())
    });

    // Listen for incoming packets on the first socket.
    while let Some(packet) = socket.next().await {
        let (packet, peer_addr) = packet?;
        eprintln!("Receive from {}: {:?}", peer_addr, packet);
        match packet {
            OscPacket::Bundle(_) => {}
            OscPacket::Message(message) => match &message.as_tuple() {
                ("/volume", &[OscType::Float(vol), OscType::String(ref s)]) => {
                    eprintln!("Set volume: {} {}", vol, s);
                }
                _ => {}
            },
        }
    }
    Ok(())
}
