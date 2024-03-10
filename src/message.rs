use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};

use crate::agent_config::AgentConfig;
use crate::packet::Packet;

/// Represents actions used by the game client and agents to communicate among themselves.
#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    /// Used to request the receiving agent's value. Should expect a `MsgSendValue` as a reply.
    MsgQueryValue,
    /// Used by an agent to send its value as a reply to a `MsgQueryValue`.
    MsgSendValue { agent_id: usize, value: u64 },
    /// Used by the game's client to kill an active agent.
    MsgKillAgent { agent_id: usize },
    /// Used by the game's client to request an agent to query other agents' values.
    MsgFetchValues {
        agent_id: usize,
        peer_addresses: Vec<AgentConfig>,
    },
    /// Used by agents to forward other agents' values to the game's client.
    MsgFwdValues {
        agent_id: usize,
        peer_values: Vec<Packet>,
    },
}
// NOTE: It would be an improvement to include nonces in messages in order to prevent replay attacks.

impl Message {
    /// Builds and returns a `MsgQueryValue` serialized into binary format using bincode.
    /// Takes no parameters.
    pub fn build_msg_query_value() -> Result<Vec<u8>, bincode::Error> {
        let message = Message::MsgQueryValue.serialize_message()?;
        Ok(message)
    }

    /// Builds a `MsgFetchValues` containing a target agent ID `agent_id` and a list of
    /// peer_addresses as a `Vec<AgentConfig>`. Returns the message serialized into binary format
    /// using bincode.
    pub fn build_msg_fetch_values(
        agent_id: usize,
        peers: &Vec<AgentConfig>,
    ) -> Result<Vec<u8>, bincode::Error> {
        let message = Message::MsgFetchValues {
            agent_id,
            peer_addresses: peers.to_vec(),
        }
        .serialize_message()?;
        Ok(message)
    }

    /// Builds a `MsgFwdValues` containing the sending agent's ID `agent_id` and a `Vec<Packet>`
    /// containing the replies, received from other agents, to be forwarded. Returns the message
    /// serialized into binary format using bincode.
    pub fn build_msg_fwd_values(
        agent_id: usize,
        peer_replies: &Vec<Packet>,
    ) -> Result<Vec<u8>, bincode::Error> {
        let message = Message::MsgFwdValues {
            agent_id,
            peer_values: peer_replies.to_vec(),
        }
        .serialize_message()?;
        Ok(message)
    }

    /// Builds a `MsgSendValue` containing `value` and `agent_id` and returns it serialized into
    /// binary format.
    pub fn build_msg_send_value(value: u64, agent_id: usize) -> Result<Vec<u8>, bincode::Error> {
        let message = Message::MsgSendValue { value, agent_id }.serialize_message()?;
        Ok(message)
    }

    /// Builds a `MsgKillAgent` containing the identifier of the agent to be killed, `agent_id`.
    /// Returns the message serialized into binary format.
    pub fn build_msg_kill_agent(agent_id: usize) -> Result<Vec<u8>, bincode::Error> {
        let message = Message::MsgKillAgent { agent_id }.serialize_message()?;
        Ok(message)
    }

    /// Serializes a variant of `Message` into binary format using bincode.
    pub fn serialize_message(&self) -> Result<Vec<u8>, bincode::Error> {
        serialize(&self)
    }

    /// Deserializes `message_bytes` from binary format into a variant of `Message`. Returns
    /// `bincode::Error` if the format of `message_bytes` is invalid.
    pub fn deserialize_message(message_bytes: &[u8]) -> Result<Message, bincode::Error> {
        deserialize(message_bytes)
    }
}
