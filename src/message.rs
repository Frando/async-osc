use rosc::{OscBundle, OscMessage, OscPacket, OscType};

/// Extension methods for the [`rosc::OscMessage`] type.
pub trait OscMessageExt {
    /// Create a new OscMessage from an address and args.
    /// The args can either be specified as a `Vec<[OscType]`, or as a tuple of regular Rust types
    /// that can be converted into [`OscType`].
    fn new<T>(addr: impl ToString, args: T) -> Self
    where
        T: IntoOscArgs;

    /// Returns `true` if the address starts with the given prefix.
    ///
    /// Returns `false` otherwise.
    fn starts_with(&self, prefix: &str) -> bool;

    /// Get a reference to the message in tuple form.
    ///
    /// This is useful for pattern matching. Example:
    ///
    /// ```no_run
    /// # use async_osc::{*, prelude::*};
    /// let message = OscMessage::new("/foo", vec![
    ///     OscType::Float(1.0), OscType::String("bar".into())
    /// ]);
    ///
    /// match message.as_tuple() {
    ///     ("foo", &[OscType::Float(val), OscType::String(ref text)]) => {
    ///         eprintln!("Got foo message with args: {}, {}", val, text);
    ///     },
    ///     _ => {}
    /// }
    /// ```
    fn as_tuple(&self) -> (&str, &[OscType]);
}

impl OscMessageExt for OscMessage {
    fn new<T>(addr: impl ToString, args: T) -> Self
    where
        T: IntoOscArgs,
    {
        let args = args.into_osc_args();
        let addr = addr.to_string();
        OscMessage { addr, args }
    }

    fn starts_with(&self, prefix: &str) -> bool {
        self.addr.starts_with(prefix)
    }

    fn as_tuple(&self) -> (&str, &[OscType]) {
        (self.addr.as_str(), &self.args[..])
    }
}

/// Extension methods for the [`rosc::OscMessage`] type.
pub trait OscPacketExt {
    /// Return `Some(&message)` if the packet is 'OscPacket::Message`.
    ///
    /// Return None otherwise.
    fn message(&self) -> Option<&OscMessage>;

    /// Return `Some(message)` if the packet is 'OscPacket::Message`.
    ///
    /// Return None otherwise.
    fn into_message(self) -> Option<OscMessage>;
}

impl OscPacketExt for OscPacket {
    fn message(&self) -> Option<&OscMessage> {
        match self {
            OscPacket::Message(message) => Some(message),
            _ => None,
        }
    }
    fn into_message(self) -> Option<OscMessage> {
        match self {
            OscPacket::Message(message) => Some(message),
            _ => None,
        }
    }
}

/// Helper trait to convert types into `Vec<[OscType]>`
pub trait IntoOscArgs {
    /// Convert self to OSC args.
    fn into_osc_args(self) -> Vec<OscType>;
}

impl<T> IntoOscArgs for Vec<T>
where
    T: Into<OscType>,
{
    fn into_osc_args(self) -> Vec<OscType> {
        let args: Vec<OscType> = self.into_iter().map(|a| a.into()).collect();
        args
    }
}

// We cannot implement IntoOscArgs for T because it conflicts
// with the impl for Vec<T> above.
// TODO: Find out if there is a solution.
// impl<T> IntoOscArgs for T
// where
//     T: Into<OscType>,
// {
//     fn into_osc_args(self) -> Vec<OscType> {
//         vec![self.into()]
//     }
// }

impl<T1> IntoOscArgs for (T1,)
where
    T1: Into<OscType>,
{
    fn into_osc_args(self) -> Vec<OscType> {
        vec![self.0.into()]
    }
}

impl<T1, T2> IntoOscArgs for (T1, T2)
where
    T1: Into<OscType>,
    T2: Into<OscType>,
{
    fn into_osc_args(self) -> Vec<OscType> {
        vec![self.0.into(), self.1.into()]
    }
}

impl<T1, T2, T3> IntoOscArgs for (T1, T2, T3)
where
    T1: Into<OscType>,
    T2: Into<OscType>,
    T3: Into<OscType>,
{
    fn into_osc_args(self) -> Vec<OscType> {
        vec![self.0.into(), self.1.into(), self.2.into()]
    }
}

impl IntoOscArgs for OscType {
    fn into_osc_args(self) -> Vec<OscType> {
        vec![self]
    }
}

/// Helper trait to convert [`OscMessage`] and [`OscBundle`] into [`OscPacket`].
pub trait IntoOscPacket {
    /// Convert into [`OscPacket`].
    fn into_osc_packet(self) -> OscPacket;
}

impl IntoOscPacket for OscMessage {
    fn into_osc_packet(self) -> OscPacket {
        OscPacket::Message(self)
    }
}

impl IntoOscPacket for OscBundle {
    fn into_osc_packet(self) -> OscPacket {
        OscPacket::Bundle(self)
    }
}

impl IntoOscPacket for OscPacket {
    fn into_osc_packet(self) -> OscPacket {
        self
    }
}

impl<T> IntoOscPacket for T
where
    T: IntoOscMessage,
{
    fn into_osc_packet(self) -> OscPacket {
        OscPacket::Message(self.into_osc_message())
    }
}

/// Helper trait to convert a `(impl ToString, impl IntoOscArgs)` tuple into [`OscMessage`].
pub trait IntoOscMessage {
    /// Convert to [`OscMessage`].
    fn into_osc_message(self) -> OscMessage;
}

impl<S, A> IntoOscMessage for (S, A)
where
    S: ToString,
    A: IntoOscArgs,
{
    fn into_osc_message(self) -> OscMessage {
        OscMessage::new(self.0, self.1)
    }
}
