use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Result};

use crate::agent_config::AgentConfig;

/// Represents a game client.
///
/// `Client`s are responsible for communicating with deployed agents
/// and querying for their individual values to determine the network value.
#[derive(Debug)]
pub struct Client {
    peer_addresses: Vec<AgentConfig>,
}

impl Client {
    /// Returns a new instance of `Client` with an empty `peer_addresses` Vec
    pub fn new() -> Self {
        Client {
            peer_addresses: Vec::new(),
        }
    }

    /// Attempts to read the `AgentConfig` data from the `agents.config` file
    /// and return it if the read operation succeeds.
    pub fn read_agent_config() -> Result<String> {
        let config = fs::read_to_string("agents.config")?;
        Ok(config)
    }

    /// Receives a string slice containing the data read from `agents.config`
    //  and attempts to deserialize and store it in Client.peer_addresses
    pub fn load_agent_config(&mut self, agent_config: &str) -> Result<()> {
        self.peer_addresses = serde_json::from_str(&agent_config)?;
        Ok(())
    }
}
