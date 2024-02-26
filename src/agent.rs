use rand::{seq::SliceRandom, thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::agent_config::AgentConfig;

static AGENT_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);
static BASE_PORT: AtomicUsize = AtomicUsize::new(5_000);
const AGENT_ADDR: &str = "127.0.0.1";


/// Represents an agent in the Liars Lie game.
///
/// Each `Agent` has an unique identifier `agent_id`, a value `value` to report when
/// queried, and a network `address` and `port` used for communication with clients and 
/// other Agents. Agents can be instantiated as either honest or liars.
///
/// # Examples
///
/// ```
/// let honest_agent = Agent::new_honest(10);
/// let liar = Agent::new_liar(10, 99); 
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Agent {
    /// An identifier for each instance of Agent.
    agent_id: usize,
    /// A value to be reported by the agent when queried.
    value: u64,
    /// The network address in which the agent listens when deployed.
    address: String,
    /// The network port in which the agent listens when deployed.
    port: usize,
}

impl Agent {
    /// Returns a new honest instance of `Agent` with the `value` field set to the value
    /// received as argument. Each new instance is assigned an unique `agent_id` 
    /// and `port`.
    pub fn new_honest(value: u64) -> Self {
        let agent_id = Self::get_new_id();
        let address = AGENT_ADDR.to_owned();
        let port = Self::get_new_port();
        Agent {
            agent_id,
            value,
            address,
            port,
        }
    }

    /// Returns a new liar instance of `Agent` with the `value` field set to an arbitrary
    /// value x, such that x != honest_value AND 1 <= x <= max_value. Each new instance
    /// is assigned an unique `agent_id` and `port`.
    pub fn new_liar(honest_value: u64, max_value: u64) -> Self {
        let agent_id = Self::get_new_id();
        let value = Self::get_liar_value(honest_value, max_value);
        let address = AGENT_ADDR.to_owned();
        let port = Self::get_new_port();
        Agent {
            agent_id,
            value,
            address,
            port,
        }
    }

    /// Receives an instance of `Agent` to generate a new instance of `AgentConfig`, 
    /// which contains only the fields of `Agent` that can be shared with other 
    /// participants of the game.
    pub fn to_config(&self) -> AgentConfig {
        AgentConfig::new(self.agent_id, &self.address, self.port)
    }

    /// TODO
    fn deploy_agent(&self) {
        todo!()
    }

    /// Returns a new unique port number for the `Agent.port` field.
    fn get_new_port() -> usize {
        BASE_PORT.fetch_add(1, Ordering::Relaxed)
    }

    /// Returns a new unique ID for the `Agent.agent_id` field.
    fn get_new_id() -> usize {
        AGENT_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
    }

    /// Returns an arbitrary `liar_value`, such that `liar_value` != `honest_value` and 
    /// 1 <= `liar_value` <= `max_value`.
    fn get_liar_value(honest_value: u64, max_value: u64) -> u64 {
        let value_to_skip = honest_value;

        // Shorten the gen_range by 1 and increment by 1 if liar_value >= value_to_skip
        // This effectively skips value_to_skip and is an alternative to a "loop until different"
        // approach, which might require a theoretically unbounded number of tries
        let mut liar_value = rand::thread_rng().gen_range(1..=(max_value - 1));
        if liar_value >= value_to_skip {
            liar_value += 1;
        }
        liar_value
    }
}

// ******************************************************************************************
// ************************************* UNIT TESTS *****************************************
// ******************************************************************************************

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn liar_value_is_diff_from_honest() {
        // Must be careful when testing randomly generated values like this.
        // Even though the chance of the test failing is negligible for a 
        // high number of iterations, for applications where security is critical
        // a more robust testing strategy should be used.
        let honest_value = 5;
        let max_value = 10;
        let iter = 10_000;

        for _ in 0..iter {
            let liar_value = Agent::get_liar_value(honest_value, max_value);
            assert_ne!(liar_value, 0, "Liar value cannot be 0");
            assert_ne!(liar_value, honest_value, "Liar value must be different from honest value");
            assert!(liar_value <= max_value, "Liar value cannot be greater than max_value");
        };
    }

    #[test]
    fn gen_unique_port() {
        let first_port = Agent::get_new_port();
        for i in 1..100 {
            let new_port = Agent::get_new_port();
            assert_eq!(first_port + i, new_port);
        }
    }

    #[test]
    fn gen_unique_agent_id() {
        let first_id = Agent::get_new_id();
        for i in 1..100 {
            let new_id = Agent::get_new_id();
            assert_eq!(first_id + i, new_id);
        }
    }


}