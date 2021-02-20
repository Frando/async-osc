#![allow(unreachable_pub)]

use async_std::net::UdpSocket;
use async_std::stream::Stream;
use futures_lite::future::Future;
use futures_lite::ready;
use std::fmt;
use std::io;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

pub(crate) type RecvFut =
    Pin<Box<dyn Future<Output = io::Result<(Vec<u8>, usize, SocketAddr)>> + Send + Sync>>;

pub(crate) struct UdpSocketStream {
    pub(crate) socket: Arc<UdpSocket>,
    fut: Option<RecvFut>,
    buf: Option<Vec<u8>>,
}

// TODO: Decide if Clone shold be enabled.
// I'm not sure about the behavior of polling from different clones.
impl Clone for UdpSocketStream {
    fn clone(&self) -> Self {
        Self::from_arc(self.socket.clone())
    }
}

impl fmt::Debug for UdpSocketStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UdpSocketStream")
            .field("socket", &*self.socket)
            .finish()
    }
}

impl UdpSocketStream {
    pub fn new(socket: UdpSocket) -> Self {
        let socket = Arc::new(socket);
        Self::from_arc(socket)
    }

    pub fn from_arc(socket: Arc<UdpSocket>) -> Self {
        let buf = vec![0u8; 1024 * 64];
        Self {
            socket,
            fut: None,
            buf: Some(buf),
        }
    }

    pub fn get_ref(&self) -> &UdpSocket {
        &self.socket
    }

    pub fn clone_inner(&self) -> Arc<UdpSocket> {
        self.socket.clone()
    }
}

impl Stream for UdpSocketStream {
    type Item = io::Result<(Vec<u8>, SocketAddr)>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            if self.fut.is_none() {
                let buf = self.buf.take().unwrap();
                let fut = recv_next(self.socket.clone(), buf);
                self.fut = Some(Box::pin(fut));
            }

            if let Some(f) = &mut self.fut {
                let res = ready!(f.as_mut().poll(cx));
                self.fut = None;
                return match res {
                    Err(e) => Poll::Ready(Some(Err(e))),
                    Ok((buf, n, addr)) => {
                        let res_buf = buf[..n].to_vec();
                        self.buf = Some(buf);
                        Poll::Ready(Some(Ok((res_buf, addr))))
                    }
                };
            }
        }
    }
}

async fn recv_next(
    socket: Arc<UdpSocket>,
    mut buf: Vec<u8>,
) -> io::Result<(Vec<u8>, usize, SocketAddr)> {
    // let mut buf = vec![0u8; 1024];
    let res = socket.recv_from(&mut buf).await;
    match res {
        Err(e) => Err(e),
        Ok((n, addr)) => Ok((buf, n, addr)),
    }
}
