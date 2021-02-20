#![forbid(unsafe_code, future_incompatible, rust_2018_idioms)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs, missing_doc_code_examples, unreachable_pub)]

//! Async library for the Open Sound Control (OSC) protocol
//!
//! # Examples
//!
//! ```
//! # #[async_std::main]
//! # async fn main() -> async_osc::Result<()> {
//! use async_std::stream::StreamExt;
//! use async_osc::{prelude::*, OscSocket, OscPacket, OscType, Error, Result};
//!
//! let mut socket = OscSocket::bind("localhost:5050").await?;
//!
//! // Open a second socket to send a test message.
//! async_std::task::spawn(async move {
//!     let socket = OscSocket::bind("localhost:0").await?;
//!     socket.connect("localhost:5050").await?;
//!     socket.send(("/volume", (0.9f32,))).await?;
//!     Ok::<(), Error>(())
//! });
//!
//! // Listen for incoming packets on the first socket.
//! while let Some(packet) = socket.next().await {
//!     let (packet, peer_addr) = packet?;
//!     eprintln!("Receive from {}: {:?}", peer_addr, packet);
//!     match packet {
//!         OscPacket::Bundle(_) => {}
//!         OscPacket::Message(message) => match message.as_tuple() {
//!             ("/volume", &[OscType::Float(vol)]) => {
//!                 eprintln!("Set volume: {}", vol);
//!                 // Do something with the received data.
//!                 // Here, just quit the doctest.
//!                 assert_eq!(vol, 0.9f32);
//!                 return Ok(())
//!             }
//!             _ => {}
//!         },
//!     }
//! }
//! # Ok(())
//! # }
//! // tbi
//! ```

/// Re-export the main OSC types from the [`rosc`] crate.
pub mod rosc {
    pub use ::rosc::{OscBundle, OscMessage, OscPacket, OscType};
}

pub use crate::rosc::*;

mod error;
mod message;
mod osc;
mod udp;

pub use error::{Error, Result};
pub use osc::{OscSender, OscSocket};
// pub use udp::*;

/// Prelude with extensions to [`rosc`] types.
///
/// It is recommended to import everything from this module whenever working with these types.
/// See [`preulude::OscMessageExt`] for details.
pub mod prelude {
    pub use crate::message::{
        IntoOscArgs, IntoOscMessage, IntoOscPacket, OscMessageExt, OscPacketExt,
    };
}
