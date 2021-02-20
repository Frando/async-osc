use async_osc::prelude::*;
use async_osc::{OscMessage, OscPacket, OscSocket, OscType, Result};
use async_std::stream::StreamExt;
use async_std::task::{self, JoinHandle};

#[async_std::test]
async fn connect_send_recv() -> Result<()> {
    let mut socket1 = OscSocket::bind("localhost:0").await?;
    let mut socket2 = OscSocket::bind("localhost:0").await?;
    let addr1 = socket1.socket().local_addr()?;
    let addr2 = socket2.socket().local_addr()?;

    let task: JoinHandle<Result<()>> = task::spawn(async move {
        if let Some(packet) = socket2.next().await {
            let (packet, peer_addr) = packet?;
            let message = packet.message().unwrap();
            assert_eq!(peer_addr, addr1);
            assert_eq!(&message.addr, "/glitch");
            assert_eq!(
                &message.args,
                &[OscType::Float(0.17), OscType::String("ultra".to_string())]
            );
            let reply = ("/ack", (1,));
            socket2.send_to(reply, peer_addr).await?;
        }
        Ok(())
    });

    socket1.connect(addr2).await?;
    socket1.send(("/glitch", (0.17f32, "ultra"))).await?;

    if let Some(Ok((OscPacket::Message(message), peer_addr))) = socket1.next().await {
        assert_eq!(message, OscMessage::new("/ack", (1,)));
        assert_eq!(peer_addr, addr2);
    }

    task.await?;

    Ok(())
}
