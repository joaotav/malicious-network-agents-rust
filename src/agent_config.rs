use serde::{Deserialize, Serialize};

/// Represents an instance of `Agent` in a format that can be shared with 
/// other participants of the game.
///
/// `AgentConfig` contains information regarding an agent's `agent_id`, `address` 
/// and `port`, which are necessary for communication with other participants of 
/// the game. `AgentConfig` omits `Agent.value`, which should be obtainable only 
/// by directly querying each instance of `Agent`.
///
/// # Example
/// ``` 
/// let config = AgentConfig::new(agent_id: 1, address: "127.0.0.1", port: 6060);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentConfig {
    /// The agent's unique ID
    agent_id: usize,
    /// The network address that can be used to communicate with this agent.
    address: String,
    /// The network port that can be used to communicate with this agent.
    port: usize,
}


impl AgentConfig {
    /// Returns a new instance of `AgentConfig` initialized with the values from `agent_id`
    /// `address` and `port`.
    pub fn new(agent_id: usize, address: &str, port: usize) -> Self {
        Self {
            agent_id,
            address: address.to_owned(),
            port,
        }
    }
}
