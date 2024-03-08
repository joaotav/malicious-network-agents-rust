use serde::{Deserialize, Serialize};
/// Represents an instance of `Agent` in a format that can be shared with
/// other participants of the game.
///
/// `AgentConfig` contains information regarding an agent's `agent_id`, `address`,
///  `port` and `public_key`, which are necessary for communication with other participants of
/// the game. `AgentConfig` omits `Agent.value`, which should be obtainable only
/// by directly querying each instance of `Agent`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentConfig {
    /// The agent's unique ID
    agent_id: usize,
    /// The network address that can be used to communicate with this agent.
    address: String,
    /// The network port that can be used to communicate with this agent.
    port: usize,
    /// The agent's public key encoded as base64. Used for signature verification.
    public_key: String,
}

impl AgentConfig {
    /// Returns a new instance of `AgentConfig` initialized with the values from `agent_id`
    /// `address`, `port` and `public_key`.
    pub fn new(agent_id: usize, address: &str, port: usize, public_key: &str) -> Self {
        Self {
            agent_id,
            address: address.to_owned(),
            port,
            public_key: public_key.to_owned(),
        }
    }

    /// Returns an agent's network address.
    pub fn get_address(&self) -> &str {
        &self.address
    }

    /// Returns an agent's unique ID.
    pub fn get_id(&self) -> usize {
        self.agent_id
    }

    /// Returns the port at which an agent is listening on `Agent.address`.
    pub fn get_port(&self) -> usize {
        self.port
    }

    /// Returns an agent's base64-encoded public key. Can be used for message authentication.
    pub fn get_public_key(&self) -> &str {
        &self.public_key
    }
}
