use anyhow::{bail, Context};
use std::collections::HashMap;
use std::fs;
use std::ops::Deref;
use std::sync::Arc;
use text_colorizer::Colorize;
use tokio::io;
use tokio::net::TcpStream;
use tokio::spawn;

use crate::agent_config::AgentConfig;
use crate::keys::Keys;
use crate::message::Message;
use crate::network_utils::*;
use crate::packet::Packet;
/// Represents a game client.
///
/// `Client`s are responsible for communicating with deployed agents
/// and querying for their individual values to determine the network value.
#[derive(Debug, PartialEq)]
pub struct Client {
    /// The client's Ed25519 key pair. Used for message authentication.
    keys: Keys,
    /// A vector containing information that allows the client to communicate with agents.
    peers: Vec<AgentConfig>,
}

impl Client {
    /// Returns a new instance of `Client` with a key pair for message signing
    /// and an empty `peers` Vec.
    pub fn new() -> Self {
        Client {
            keys: Keys::new_key_pair(),
            peers: Vec::new(),
        }
    }

    /// Returns the client's keypair for message signing.
    pub fn get_keys(&self) -> &Keys {
        &self.keys
    }

    /// Returns the client's list of peers.
    pub fn get_peers(&self) -> &Vec<AgentConfig> {
        &self.peers
    }

    /// Attempts to read the `AgentConfig` data from the `agents.config` file
    /// and return it if the read operation succeeds.
    pub fn read_agent_config() -> Result<String, io::Error> {
        let config = fs::read_to_string("agents.config")?;
        Ok(config)
    }

    /// Receives a string slice containing the data read from `agents.config`
    /// and attempts to deserialize and store it in Client.peers
    pub fn store_agent_config(&mut self, agent_config: &str) -> Result<(), serde_json::Error> {
        self.peers = serde_json::from_str(&agent_config)?;
        Ok(())
    }

    /// Reads agent configuration from a file and stores it in an instance of `Client`.
    pub fn load_agent_config(&mut self) -> anyhow::Result<()> {
        let agent_config = Self::read_agent_config()?;
        self.store_agent_config(&agent_config)?;
        Ok(())
    }

    /// Receives a `MsgSendValue` from an agent and verifies if it has been correctly signed by the
    /// agent to whom the client has sent a `MsgQueryValue`.
    fn handle_msg_send_value(
        message_bytes: &[u8],
        signature: &Option<Vec<u8>>,
        public_key: &str,
        agent_value: u64,
        agent_id: usize,
    ) -> anyhow::Result<u64> {
        // `MsgSendValue` must be signed by the sender. Check if the message is accompanied by
        // a signature and verify it if so.
        if let Some(signature) = signature {
            Keys::verify(message_bytes, signature, public_key)?;
        } else {
            bail!(
                "[!] error: MsgSendValue requires a signature, but the received packet contains None\n"
            );
        }

        Ok(agent_value)
    }

    /// Receives the values reported by the game's agents and infers the network value from them.
    pub fn infer_network_value(agent_values: &Vec<u64>) -> Option<Vec<u64>> {
        let mut values_count = HashMap::new();

        // Count the number of occurrences of each different value returned by the agents
        for &value in agent_values {
            *values_count.entry(value).or_insert(0) += 1;
        }

        // Return the maximum number of occurrences out of all the values
        let max_count = match values_count.values().max() {
            Some(max_count) => *max_count,
            None => return None,
        };

        // Get all the values whose occurrence is equal to the max number of occurrences.
        // Different values may be tied with the most number of occurrences, in which case
        // all of them will be returned as the network value.
        let network_value = values_count
            .into_iter()
            .filter(|&(_, value_count)| value_count == max_count)
            .map(|(value, _)| value)
            .collect();

        Some(network_value)
    }

    /// Prints the network value inferred after playing a round of the game.
    /// Will print multiple values if there was no majority consensus on a single
    /// network value.
    pub fn print_network_value(network_value: &Option<Vec<u64>>) {
        match network_value {
            Some(network_value) => match network_value.len() {
                // If a single value has the majority of votes
                1 => println!(
                    "{} {}\n",
                    "[+] The network value is:".bold(),
                    network_value[0]
                ),

                // If different values are tied for the majority of votes
                _ => {
                    let values: Vec<String> = network_value
                        .iter()
                        .map(|value| value.to_string())
                        .collect();

                    println!(
                        "{}",
                        "[+] Unable to determine a single network value.".bold()
                    );
                    println!(
                        "{} {}\n",
                        "[+] The following values are tied:".bold(),
                        values.join(", ")
                    );
                }
            },

            // If no valid votes were received from the agents
            None => {
                println!("{}",
                    "[+] Unable to determine the network value because no valid replies were received.\n".bold()
                );
            }
        }
    }

    /// Queries an individual agent for its value by sending a `MsgQueryValue`.
    async fn send_msg_query_value(
        client_keys: &Keys,
        socket: &mut TcpStream,
        agent_pubkey: &str,
    ) -> anyhow::Result<u64> {
        let message = Message::build_msg_query_value()
            .context("[!] error: failed to build MsgQueryValue\n")?;

        // Compute the signature of the serialized message
        // NOTE: For messages composed by large amounts of data, signing the whole message incurs
        // a significant overhead. Ideally, the hash of  the message should be signed instead.
        // Here, given the small sizes of messages, we sign the whole message for simplicity's sake.
        let message_signature = client_keys.sign(&message)?;

        // Build a packet with the message and message signature
        let packet = Packet::build_packet(message, Some(message_signature))
            .context("[!] error: failed to build packet\n")?;

        send_packet(&packet, socket).await?;

        let reply = recv_packet(socket).await?;

        let reply_packet = Packet::unpack(&reply)?;

        match Message::deserialize_message(&reply_packet.message) {
            Ok(Message::MsgSendValue { value, agent_id }) => Self::handle_msg_send_value(
                &reply_packet.message,
                &reply_packet.msg_sig,
                agent_pubkey,
                value,
                agent_id,
            ),
            Ok(other) => bail!("[!] error: expected MsgSendValue, received {:?}\n", other),
            Err(e) => bail!("[!] error: unable to decode message - {}\n", e),
        }
    }

    /// Builds and sends a MsgKillAgent to an active agent. This message does not expect a reply.
    async fn send_msg_kill_agent(
        agent_id: usize,
        client_keys: &Keys,
        socket: &mut TcpStream,
    ) -> anyhow::Result<()> {
        let message = Message::build_msg_kill_agent(agent_id)
            .context("[!] error: failed to build MsgKillAgent\n")?;

        let message_signature = client_keys.sign(&message)?;

        let packet = Packet::build_packet(message, Some(message_signature))
            .context("[!] error: failed to build packet\n")?;

        match send_packet(&packet, socket).await {
            Ok(()) => Ok(()),
            Err(e) => bail!("[!] error: unable to reach agent {} - {}", agent_id, e),
        }
    }

    /// Plays a standard round of the game. The game client connects to the agents loaded
    /// from the `agents.config` file, queries them individually for their values and
    /// returns a Vec<u64> containing all valid agent replies. A reply is valid iff
    /// the received message is not corrupted and it has been signed by the agent to which
    /// the query was sent.
    pub async fn play_standard(&self) -> anyhow::Result<Vec<u64>> {
        let mut agent_conn_handles = Vec::new();
        let mut agent_values = Vec::new();
        let keys = Arc::new(self.keys.clone());

        for peer in &self.peers {
            let address = peer.get_address();
            let port = peer.get_port();
            let mut socket = match connect(address, port).await {
                Ok(socket) => socket,
                Err(e) => {
                    println!(
                        "[!] error: failed to connect to (Agent ID: {} - {}:{}) - {}\n",
                        peer.get_id(),
                        address,
                        port,
                        e
                    );
                    continue;
                }
            };

            let agent_pubkey = peer.get_public_key().to_owned();
            let client_keys = keys.clone();
            let handle = spawn(async move {
                Self::send_msg_query_value(&client_keys, &mut socket, &agent_pubkey).await
            });
            agent_conn_handles.push(handle);
        }

        for handle in agent_conn_handles {
            match handle.await {
                Ok(Ok(agent_value)) => {
                    agent_values.push(agent_value);
                }
                Ok(Err(e)) => println!("{}", e),
                Err(e) => println!("[!] error: task panicked - {}\n", e),
            }
        }

        Ok(agent_values)
    }

    /// Connects to `address`:`port` and sends a MsgKillAgent addressed to `agent_id`.
    pub async fn kill_agent(
        &self,
        agent_id: usize,
        address: &str,
        port: usize,
    ) -> anyhow::Result<String> {
        let mut socket = match connect(address, port).await {
            Ok(socket) => socket,
            Err(e) => {
                bail!(
                    "[!] error: failed to connect to {}:{} - {}\n",
                    address,
                    port,
                    e
                )
            }
        };

        let client_keys = self.keys.clone();

        let handle =
            spawn(
                async move { Self::send_msg_kill_agent(agent_id, &client_keys, &mut socket).await },
            );

        match handle.await {
            Ok(Ok(())) => Ok(format!(
                "{} (Agent ID: {} - {}:{})\n",
                "[+] Killed agent".bold(),
                agent_id,
                address,
                port
            )),
            Ok(Err(e)) => Err(e),
            Err(e) => Err(e.into()),
        }
    }
}
