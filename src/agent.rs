use anyhow::Context;
use rand::Rng;
use std::sync::atomic::{AtomicUsize, Ordering};
use text_colorizer::Colorize;
use tokio::net::{TcpListener, TcpStream};
use tokio::spawn;
use tokio::sync::oneshot;

use crate::agent_config::AgentConfig;
use crate::keys::Keys;
use crate::message::Message;
use crate::network_utils::*;
use crate::packet::Packet;

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
#[derive(Debug, Clone, PartialEq)]
pub struct Agent {
    /// An identifier for each instance of Agent.
    agent_id: usize,
    /// A value to be reported by the agent when queried.
    value: u64,
    /// The network address in which the agent listens when deployed.
    address: String,
    /// The network port in which the agent listens when deployed.
    port: usize,
    /// The agent's private and public keys for signing messages.
    keys: Keys,
    /// The game client's base64-encoded public key. Used to authenticate received messages.
    game_client_pubkey: String,
}

impl Agent {
    /// Returns a new honest instance of `Agent` with the `value` field set to the value
    /// received as argument. Each new instance is assigned an unique `agent_id`
    /// and `port`.
    pub fn new_honest(value: u64, game_client_pubkey: String) -> Self {
        let agent_id = Self::get_new_id();
        let address = AGENT_ADDR.to_owned();
        let port = Self::get_new_port();
        let keys = Keys::new_key_pair();
        Agent {
            agent_id,
            value,
            address,
            port,
            keys,
            game_client_pubkey,
        }
    }

    /// Returns a new liar instance of `Agent` with the `value` field set to an arbitrary
    /// value x, such that x != honest_value AND 1 <= x <= max_value. Each new instance
    /// is assigned an unique `agent_id` and `port`.
    pub fn new_liar(honest_value: u64, max_value: u64, game_client_pubkey: String) -> Self {
        let agent_id = Self::get_new_id();
        let value = Self::get_liar_value(honest_value, max_value);
        let address = AGENT_ADDR.to_owned();
        let port = Self::get_new_port();
        let keys = Keys::new_key_pair();
        Agent {
            agent_id,
            value,
            address,
            port,
            keys,
            game_client_pubkey,
        }
    }

    /// Receives an instance of `Agent` to generate a new instance of `AgentConfig`,
    /// which contains only the fields of `Agent` that can be shared with other
    /// participants of the game.
    pub fn to_config(&self) -> AgentConfig {
        AgentConfig::new(
            self.agent_id,
            &self.address,
            self.port,
            &self.keys.get_public_key(),
        )
    }

    /// TODO!
    fn authenticate_message(
        message_bytes: &[u8],
        msg_sig: Option<&[u8]>,
        public_key: &str,
    ) -> anyhow::Result<()> {
        todo!();
    }

    /// Builds and sends a `MsgSendValue` packet as a response to a `MsgQueryValue` request.
    async fn handle_msg_query_value(&self, socket: &mut TcpStream) -> anyhow::Result<()> {
        // Build a MsgSendValue to send as a reply to MsgQueryValue
        let reply = Message::build_msg_send_value(self.value, self.agent_id)?;

        // Generate a signature of the message
        let reply_sig = self.keys.sign(&reply)?;

        // Build a packet containing the message and the message signature
        let reply_packet = Packet::build_packet(reply, Some(reply_sig))?;

        send_packet(&reply_packet, socket).await?;

        Ok(())
    }

    /// TODO!
    fn handle_msg_send_value(&self) {
        todo!();
    }

    /// TODO!
    fn handle_msg_kill_agent(&self) {
        todo!();
    }

    /// TODO!
    fn handle_msg_shutdown(&self) {
        todo!();
    }

    /// Receives a packet and executes the logic required by the message contained within.
    async fn packet_handler(
        &self,
        packet_bytes: &[u8],
        socket: &mut TcpStream,
    ) -> anyhow::Result<()> {
        let packet = Packet::unpack(packet_bytes).context("error: unable to decode packet\n")?;
        let message = Message::deserialize_message(&packet.message);

        match message {
            Ok(Message::MsgQueryValue) => {
                self.handle_msg_query_value(socket).await?;
            }
            Ok(Message::MsgSendValue { value, agent_id }) => todo!(),
            Ok(Message::MsgKillAgent { agent_id }) => todo!(),
            Ok(Message::MsgShutdown { agent_id }) => todo!(),
            Err(e) => println!("error: unable to decode message - {}\n", e),
        }

        Ok(())
    }

    /// Processes incoming packets from an active TCP connection. This function reads packets from
    /// a `TcpStream` and handles them using internal packet handling logic.
    async fn connection_handler(&self, socket: &mut TcpStream) -> anyhow::Result<()> {
        let packet_bytes = recv_packet(socket).await?;
        self.packet_handler(&packet_bytes, socket).await?;
        Ok(())
    }

    /// Spawns a task to execute an instance of `Agent` and listen for incoming communication
    /// requests. The agent is bound to a network address specified by the fields `Agent.address`
    /// and `Agent.port`.
    pub async fn start_agent(&self, ready_signal: oneshot::Sender<()>) {
        let listener = TcpListener::bind(format!("{}:{}", self.address, self.port)).await;
        let listener = match listener {
            Ok(listener) => listener,
            Err(e) => {
                println!(
                    "error: failed to bind agent {} to address {}:{} - {}\n",
                    self.agent_id, self.address, self.port, e
                );
                return;
            }
        };

        println!(
            "{} (Agent ID: {} - Listening on: {}:{})\n",
            "[+] Spawned agent".bold(),
            self.agent_id,
            self.address,
            self.port
        );

        // Send a signal back to the calling function to inform that the agent has been spawned and
        // execution may continue
        ready_signal.send(());

        // Awaits for incoming connection requests
        loop {
            let (mut socket, _) = match listener.accept().await {
                Ok((socket, addr)) => (socket, addr),
                Err(e) => {
                    println!("error: couldn't accept connection: {:?}\n", e);
                    continue;
                }
            };

            // Cloning can be expensive, however, given that instances of `Agent`
            // do not contain large amounts of data, using it here allows us to
            // avoid the extra complexity of having to manage lifetimes.
            let agent = self.clone();
            spawn(async move {
                agent.connection_handler(&mut socket).await;
            });
        }
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
            assert_ne!(
                liar_value, honest_value,
                "Liar value must be different from honest value"
            );
            assert!(
                liar_value <= max_value,
                "Liar value cannot be greater than max_value"
            );
        }
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
