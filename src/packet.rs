use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};

/// Encapsulates message data to be sent between the game's client and agents.
///
/// A `Packet` contains a field `message`, which specifies a request or a response, and
/// an optional field `msg_sig` which contains a signature of `message` by the sender.
#[derive(Debug, Serialize, Deserialize)]
pub struct Packet {
    /// A message containing the data to be sent.
    pub message: Vec<u8>,
    /// An optional signature of the message for authentication purposes.
    pub msg_sig: Option<Vec<u8>>,
}

impl Packet {
    pub fn new(message: Vec<u8>, msg_sig: Option<Vec<u8>>) -> Self {
        Packet { message, msg_sig }
    }

    /// Builds a new instance of `Packet`, containing a message `message` and an optional message
    /// signature `msg_sig`, and returns it serialized into binary format.
    pub fn build_packet(
        message: Vec<u8>,
        msg_sig: Option<Vec<u8>>,
    ) -> Result<Vec<u8>, bincode::Error> {
        let packet = Self::new(message, msg_sig);
        serialize(&packet)
    }

    /// Receives a byte array `data`, expected to be in binary format, and attempts to deserialize
    /// it into an instance of `Packet`. Returns `bincode::Error` if the format of `data` is invalid.
    pub fn unpack(data: &[u8]) -> Result<Self, bincode::Error> {
        deserialize(data)
    }
}
