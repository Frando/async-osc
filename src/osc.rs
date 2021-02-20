use async_std::net::{ToSocketAddrs, UdpSocket};
use async_std::stream::Stream;
use futures_lite::ready;
use rosc::OscPacket;
use std::io;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use crate::error::Error;
use crate::prelude::IntoOscPacket;
use crate::udp::UdpSocketStream;

/// A UDP socket to send and receive OSC messages.
#[derive(Debug)]
pub struct OscSocket {
    socket: UdpSocketStream,
}

impl OscSocket {
    /// Creates a new OSC socket from a [`async_std::net::UdpSocket`].
    pub fn new(socket: UdpSocket) -> Self {
        let socket = UdpSocketStream::new(socket);
        Self { socket }
    }

    /// Creates an OSC socket from the given address.
    ///
    /// Binding with a port number of 0 will request that the OS assigns a port to this socket.
    /// The port allocated can be queried via [`local_addr`] method.
    ///
    /// [`local_addr`]: #method.local_addr
    pub async fn bind<A: ToSocketAddrs>(addr: A) -> Result<Self, Error> {
        let socket = UdpSocket::bind(addr).await?;
        Ok(Self::new(socket))
    }

    /// Connects the UDP socket to a remote address.
    ///
    /// When connected, only messages from this address will be received and the [`send`] method
    /// will use the specified address for sending.
    ///
    /// [`send`]: #method.send
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main() -> async_osc::Result<()> { async_std::task::block_on(async {
    /// #
    /// use async_osc::{prelude::*, OscSocket, OscMessage};
    ///
    /// let socket = OscSocket::bind("127.0.0.1:0").await?;
    /// socket.connect("127.0.0.1:8080").await?;
    /// #
    /// # Ok(()) }) }
    /// ```
    pub async fn connect<A: ToSocketAddrs>(&self, addrs: A) -> Result<(), Error> {
        self.socket().connect(addrs).await?;
        Ok(())
    }

    /// Sends an OSC packet on the socket to the given address.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main() -> async_osc::Result<()> { async_std::task::block_on(async {
    /// #
    /// use async_osc::{prelude::*, OscSocket, OscMessage};
    ///
    /// let socket = OscSocket::bind("127.0.0.1:0").await?;
    /// let addr = "127.0.0.1:5010";
    /// let message = OscMessage::new("/volume", (0.8,));
    /// socket.send_to(message, &addr).await?;
    /// #
    /// # Ok(()) }) }
    /// ```
    pub async fn send_to<A: ToSocketAddrs, P: IntoOscPacket>(
        &self,
        packet: P,
        addrs: A,
    ) -> Result<(), Error> {
        let buf = rosc::encoder::encode(&packet.into_osc_packet())?;
        let n = self.socket().send_to(&buf[..], addrs).await?;
        check_len(&buf[..], n)
    }

    /// Sends a packet on the socket to the remote address to which it is connected.
    ///
    /// The [`connect`] method will connect this socket to a remote address.
    /// This method will fail if the socket is not connected.
    ///
    /// [`connect`]: #method.connect
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main() -> async_osc::Result<()> { async_std::task::block_on(async {
    /// #
    /// use async_osc::{prelude::*, OscSocket, OscMessage};
    ///
    /// let socket = OscSocket::bind("127.0.0.1:34254").await?;
    /// socket.connect("127.0.0.1:8080").await?;
    /// socket.send(("/volume", (1.0f32,))).await?;
    /// #
    /// # Ok(()) }) }
    /// ```
    pub async fn send<P: IntoOscPacket>(&self, packet: P) -> Result<(), Error> {
        let buf = rosc::encoder::encode(&packet.into_osc_packet())?;
        let n = self.socket().send(&buf[..]).await?;
        check_len(&buf[..], n)
    }

    /// Create a standalone sender for this socket.
    ///
    /// The sender can be moved to other threads or tasks.
    pub fn sender(&self) -> OscSender {
        OscSender::new(self.socket.clone_inner())
    }

    /// Get a reference to the underling [`UdpSocket`].
    pub fn socket(&self) -> &UdpSocket {
        self.socket.get_ref()
    }

    /// Returns the local address that this socket is bound to.
    ///
    /// This can be useful, for example, when binding to port 0 to figure out which port was
    /// actually bound.
    pub fn local_addr(&self) -> Result<SocketAddr, Error> {
        let addr = self.socket().local_addr()?;
        Ok(addr)
    }
}

impl Stream for OscSocket {
    type Item = Result<(OscPacket, SocketAddr), Error>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let packet = ready!(Pin::new(&mut self.socket).poll_next(cx));
        let message = match packet {
            None => None,
            Some(packet) => Some(match packet {
                Err(err) => Err(err.into()),
                Ok((buf, peer_addr)) => rosc::decoder::decode(&buf[..])
                    .map_err(|e| e.into())
                    .map(|p| (p, peer_addr)),
            }),
        };
        Poll::Ready(message)
    }
}

/// A sender to send messages over an OSC socket.
///
/// See [`OscSocket::sender`].
#[derive(Clone, Debug)]
pub struct OscSender {
    socket: Arc<UdpSocket>,
}

impl OscSender {
    fn new(socket: Arc<UdpSocket>) -> Self {
        Self { socket }
    }

    /// Sends an OSC packet on the socket to the given address.
    ///
    /// See [`OscSocket::send_to`].
    pub async fn send_to<A: ToSocketAddrs, P: IntoOscPacket>(
        &self,
        packet: P,
        addrs: A,
    ) -> Result<(), Error> {
        let buf = rosc::encoder::encode(&packet.into_osc_packet())?;
        let n = self.socket().send_to(&buf[..], addrs).await?;
        check_len(&buf[..], n)
    }

    /// Sends an OSC packet on the connected socket.
    ///
    /// See [`OscSocket::send`].
    pub async fn send<P: IntoOscPacket>(&self, packet: P) -> Result<(), Error> {
        let buf = rosc::encoder::encode(&packet.into_osc_packet())?;
        let n = self.socket().send(&buf[..]).await?;
        check_len(&buf[..], n)
    }

    /// Get a reference to the underling [`UdpSocket`].
    pub fn socket(&self) -> &UdpSocket {
        &*self.socket
    }
}

fn check_len(buf: &[u8], len: usize) -> Result<(), Error> {
    if len != buf.len() {
        Err(io::Error::new(io::ErrorKind::Interrupted, "UDP packet not fully sent").into())
    } else {
        Ok(())
    }
}
