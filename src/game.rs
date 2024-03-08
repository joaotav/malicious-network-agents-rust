use std::io::{self, Write};
use text_colorizer::Colorize;
use tokio::spawn;
use tokio::sync::oneshot;

use crate::agent::{Agent, AgentStatus};
use crate::client::Client;

/// Represents the configuration for a game of Liars Lie.
///
/// # Example
/// ```
/// let mut game = Game::new();
/// ```
#[derive(Debug, PartialEq)]
pub struct Game {
    /// Represents the state of the game. Should be set to `false` if the game is
    /// not ready to be played.
    is_ready: bool,
    /// The value assigned to all honest agents.
    value: Option<u64>,
    /// The maximum value that can be assigned to a liar.
    max_value: Option<u64>,
    /// A vector to store instances of `Agent` that are deployed and ready
    /// to participate in a round of the game.
    active_agents: Vec<Agent>,
    /// The game's client. Used to communicate with agents.
    game_client: Client,
}

impl Game {
    pub fn new() -> Self {
        Game {
            is_ready: false,
            value: None,
            max_value: None,
            active_agents: Vec::new(),
            game_client: Client::new(),
        }
    }

    pub fn print_welcome() {
        println!("\n{}\n", ">>>>> Welcome to Liars Lie! <<<<<".bold().green());
        println!("{}\n", "Type 'help' for a list of commands.".bold());
    }

    fn print_started() {
        println!("{}", "The game has already been started!\n".bold().red());
    }

    fn print_not_started() {
        println!("{}", "The game has not yet been started!\n".bold().red());
    }

    fn print_ready(&self) {
        if self.value == None || self.max_value == None || self.active_agents.len() == 0 {
            panic!("Game cannot be started! Missing game values or active agents.\n");
        }
        println!("{}\n", "[+] Game is ready!".bold());
    }

    /// Resets all the fields of `Game` to their default values as specified by `Game::new()`.
    fn reset_game(&mut self) {
        *self = Game::new();
    }

    /// Attempts to write data to the `agents.config` file.
    fn write_agent_config(agents_config: &str) -> std::io::Result<()> {
        std::fs::write("agents.config", agents_config)?;
        Ok(())
    }

    /// Attempts to delete the `agents.config` file.
    fn remove_agent_config() -> std::io::Result<()> {
        std::fs::remove_file("agents.config")?;
        Ok(())
    }

    // Convert all instances of `Agent` stored in `Game.active_agents` into
    // instances of `AgentConfig` and convert them to JSON
    fn gen_agent_config(&self) -> Result<String, serde_json::Error> {
        let mut agents_config = Vec::new();
        for agent in &self.active_agents {
            agents_config.push(agent.to_config());
        }
        serde_json::to_string_pretty(&agents_config)
    }

    // Checks if the `agents.config` file exists in the current directory
    fn agent_config_exists() -> bool {
        std::path::Path::new("agents.config").is_file()
    }

    /// Calculates and returns the number of honest agents and liars in a game based on
    /// the total number of agents represented by `num_agents` and the percentage of liars
    /// represented by `liar_ratio`. `num_liars` is truncated, e.g, if `num_agents` is 6
    /// and `liar_ratio` is 0.6, therefore `num_liars` is 3.59, `num_liars` will be 3.
    fn get_agent_distribution(num_agents: u16, liar_ratio: f32) -> (u16, u16) {
        // Number of honest agents = `num_agents` * (1 - `liar_ratio`)
        // Number of liars = (`num_agents` * `liar_ratio`)
        let num_liars = ((num_agents as f32) * liar_ratio) as u16;
        let num_honest = num_agents - num_liars;
        (num_honest, num_liars)
    }

    /// Creates `num_honest` instances of honest agents and push those instances
    /// into `Game.active_agents`.
    fn add_honest_agents(&mut self, value: u64, num_honest: u16) {
        for _ in 1..=num_honest {
            self.active_agents.push(Agent::new_honest(
                value,
                self.game_client.get_keys().get_public_key().to_owned(),
            ));
        }
    }

    /// Creates `num_liars` instances of liars and push those instances
    /// into `Game.active_agents`.
    fn add_liar_agents(&mut self, value: u64, max_value: u64, num_liars: u16) {
        for _ in 1..=num_liars {
            self.active_agents.push(Agent::new_liar(
                value,
                max_value,
                self.game_client.get_keys().get_public_key().to_owned(),
            ));
        }
    }

    /// Sets the `Game.value` and `Game.max_value` fields to be used as a reference
    /// when creating new agents. Also sets the `Game.is_ready` to `true`.
    fn init_game(&mut self, value: u64, max_value: u64) {
        self.set_value(value);
        self.set_max_value(max_value);
        self.set_ready();
    }

    /// A setter method for `Game.value`. Used to store the `max_value` used for agents when
    /// the game was started.
    // May not be idiomatic Rust, see: https://www.reddit.com/r/rust/comments/d7w6n7
    fn set_value(&mut self, value: u64) {
        self.value = Some(value);
    }

    /// A setter method for `Game.max_value`. Used to store the `max_value` used for agents when
    /// the game was started.
    fn set_max_value(&mut self, max_value: u64) {
        self.max_value = Some(max_value);
    }

    /// Sets `Game.is_ready` to `true`, indicating that the game is ready to be played.
    fn set_ready(&mut self) {
        self.is_ready = true;
    }

    /// Returns the game's list of active agents.
    fn get_active_agents(&self) -> &Vec<Agent> {
        &self.active_agents
    }

    /// Asynchronously spawns tasks for the uninitialized game agents in `Game.active_agents`. Waits
    /// for the initialization of all agents before continuing execution.
    async fn start_game_agents(&mut self) {
        let mut ready_signals = Vec::new();
        let mut spawned_count = 0;
        for agent in &self.active_agents {
            if agent.get_status() == AgentStatus::Uninitialized {
                // Use a oneshot channel to wait for agents to be spawned
                let (signal_transmitter, signal_receiver) = oneshot::channel();
                let agent = agent.clone();
                spawn(async move {
                    agent.start_agent(signal_transmitter).await;
                });
                ready_signals.push(signal_receiver);
            }
        }

        // Wait for all tasks to finish their attempt at spawning an agent
        for signal_receiver in ready_signals {
            match signal_receiver.await {
                Ok(spawned_id) => {
                    if let Some(index) = self
                        .get_active_agents()
                        .iter()
                        .position(|agent| agent.get_id() == spawned_id)
                    {
                        self.active_agents[index].set_ready();
                        spawned_count += 1;
                    }
                }
                Err(e) => println!("{}", e),
            }
        }

        // If any of the new (uninitialized) agents failed to be spawned, remove them from the
        // active_agents Vec.
        self.active_agents
            .retain(|agent| agent.get_status() != AgentStatus::Uninitialized);

        println!(
            "{}{}{}\n",
            "[+] Sucessfully spawned ".bold(),
            spawned_count,
            " game agents!".bold()
        );
    }

    /// Executes the `start` command. The `start` command launches a number of independent
    /// agents and produces the `agents.config` file containing information that can be used
    /// to communicate with those agents. It then displays a message to indicate that the
    //  game is ready to be played.
    pub async fn start(&mut self, value: u64, max_value: u64, num_agents: u16, liar_ratio: f32) {
        if self.is_ready() {
            Game::print_started();
            return;
        }

        println!("{}", "[+] Starting game!\n".bold());

        let (num_honest, num_liars) = Self::get_agent_distribution(num_agents, liar_ratio);

        // NOTE: An improvement would be to shuffle the values or ids of agents in
        // active_agents to prevent honest agents and liars from being identified
        // by looking at the agents.config file. E.g, given a vector with agent_ids
        // in an increasing order, the first half of agents all have the same value (honest)
        // and the second half all have different values (liars).
        self.add_honest_agents(value, num_honest);
        self.add_liar_agents(value, max_value, num_liars);

        self.start_game_agents().await;

        let agent_config = match self.gen_agent_config() {
            Ok(agent_config) => agent_config,
            Err(e) => {
                // Should not happen unless there is an issue in the code of the application itself
                panic!("[!] error: failed to generate agent configuration - {}", e);
            }
        };

        if let Err(e) = Self::write_agent_config(&agent_config) {
            // Could not write config to a file, kill spawned agents as they will be unreachable
            for agent in &self.active_agents {
                let _ = self
                    .game_client
                    .kill_agent(agent.get_id(), agent.get_address(), agent.get_port())
                    .await;
            }
            self.reset_game();
            println!("[!] error: failed to write agents.config file - {}", e);
            return;
        }

        self.init_game(value, max_value);
        self.print_ready();
    }

    /// Executes the `play` command. The `play` command creates an instance of
    /// `Client`, which then reads the `agents.config` file to obtain information
    /// about the currently deployed agents. By using the information obtained from
    /// the file, the client must then directly query each individuaal agent for their
    /// value. After collecting the value from every agent, the client must determine
    /// the network value and print it.
    pub async fn play(&mut self) {
        if !self.is_ready() {
            Game::print_not_started();
            return;
        }

        println!("{}", "[+] Playing a standard round...\n".bold());

        if let Err(e) = self.game_client.load_agent_config() {
            println!(
                "[!] error: failed to load data from agents.config - {}\n",
                e
            );
            return;
        }

        println!(
            "{}{}{}\n",
            "[+] Querying ".bold(),
            self.game_client.get_peers().len(),
            " agents for their values...".bold()
        );

        match self.game_client.play_standard().await {
            Ok(agent_values) => {
                Client::print_network_value(&Client::infer_network_value(&agent_values))
            }
            Err(e) => println!("{}", e),
        };
    }

    /// Executes the `stop` command. The `stop` command stops all agents listed
    /// in the `agents.config`file, removes all agent information from the same file,
    /// and exit from the program.
    pub async fn stop(&mut self) {
        if self.is_ready() {
            // Load agent information from agents.config, as the game may have been extended but not
            // played, causing the client's information on agents to be outdated.
            if let Err(e) = self.game_client.load_agent_config() {
                println!(
                    "[!] error: failed to load data from agents.config - {}\n",
                    e
                );
                return;
            }

            println!("{}", "[+] Stopping all agents...\n".bold());

            // Send a MsgKillAgent to every agent in the game client's list of peers
            for agent in self.game_client.get_peers() {
                match self
                    .game_client
                    .kill_agent(agent.get_id(), agent.get_address(), agent.get_port())
                    .await
                {
                    Ok(_) => (),
                    Err(e) => println!("{}", e),
                }
            }

            if let Err(e) = Self::remove_agent_config() {
                println!("[!] error: unable to remove agents.config file - {}\n", e);
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
        std::process::exit(0);
    }

    /// Executes the `kill` command. The `kill` command receives an agent ID as an argument
    /// and kills the corresponding agent, but does not modify the `agents.config` file.
    pub async fn kill(&mut self, target_id: usize) {
        if !self.is_ready() {
            Game::print_not_started();
            return;
        }

        if let Some(index) = self
            .active_agents
            .iter()
            .position(|agent| agent.get_id() == target_id)
        {
            let address = self.active_agents[index].get_address();
            let port = self.active_agents[index].get_port();

            match self.game_client.kill_agent(target_id, address, port).await {
                Ok(success_msg) => {
                    println!("{}", success_msg);
                    self.active_agents[index].set_killed();
                }
                Err(e) => println!("{}", e),
            }
        } else {
            println!(
                "[!] error: the ID '{}' does not correspond to any active agent\n",
                target_id
            );
            return;
        }
    }
    /// Executes the `extend` command. The `extend` command checks for the existence of
    /// the `agents.config` file, and if present, extends it by launching new agents.
    pub async fn extend(&mut self, num_agents: u16, liar_ratio: f32) {
        if !self.is_ready() || !Self::agent_config_exists() {
            Game::print_not_started();
            return;
        }

        let (num_honest, num_liars) = Self::get_agent_distribution(num_agents, liar_ratio);

        // Backup and revert to current agents if something goes wrong after new agents are added
        let agents_backup = self.active_agents.clone();

        // self.value and self.max_value should not be None since self.is_ready() == true,
        if let (Some(value), Some(max_value)) = (self.value, self.max_value) {
            self.add_honest_agents(value, num_honest);
            self.add_liar_agents(value, max_value, num_liars);
        } else {
            panic!("[!] Unable to extend game; missing game settings.");
        }

        self.start_game_agents().await;

        let agent_config = match self.gen_agent_config() {
            Ok(agent_config) => agent_config,
            Err(e) => {
                // Should not happen unless there is an issue in the code of the application itself
                panic!(
                    "[!] error: unable to extend game; failed to generate agent configuration - {}\n",
                    e
                );
            }
        };

        if let Err(e) = Self::write_agent_config(&agent_config) {
            println!(
                "[!] error: unable to extend game; failed to write agents.config file - {}\n",
                e
            );
            // If unable to write new agent configuration to the agents.config file, new agents
            // will be unreachable. Kill the newly spawned agents.
            for agent in self.active_agents.iter() {
                // Kill any agents that were not present in `Game.active_agents` before the execution
                // of the extend command
                if !agents_backup
                    .iter()
                    .any(|old_agent| old_agent.get_id() == agent.get_id())
                {
                    let _ = self
                        .game_client
                        .kill_agent(agent.get_id(), agent.get_address(), agent.get_port())
                        .await;
                }
            }
            // Reset `active_agents` to its previous state, before extension
            self.active_agents = agents_backup;

            return;
        }
    }

    /// Executes the `playexpert` command. The `playexpert` command plays a round of the
    /// the game in expert mode. Expert mode is similar to the standard mode implemented by
    /// the `play` command, however unlike in standard mode, the client can only directly
    /// query a subset of the currently deployed agents, the size of which is taken as
    /// an argument by `fn play_expert()`.
    pub fn play_expert(&self, num_agents: u16, liar_ratio: f32) {
        if !self.is_ready() {
            Game::print_not_started();
            return;
        }
        todo!()
    }

    /// Attempts to read user input from stdin, trim it, and return it.
    pub fn get_user_input() -> Result<String, io::Error> {
        let mut user_input = String::new();
        print!("{} ", ">>".bold().green());
        io::stdout().flush()?;
        io::stdin().read_line(&mut user_input)?;
        println!();
        Ok(user_input.trim().to_owned())
    }

    /// Returns a bool that represents the state of the game.
    fn is_ready(&self) -> bool {
        self.is_ready
    }
}

// ******************************************************************************************
// ************************************* UNIT TESTS *****************************************
// ******************************************************************************************

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reset_game() {
        let mut game = Game::new();

        game.is_ready = true;
        game.value = Some(5);
        game.max_value = Some(10);
        game.reset_game();

        assert_ne!(game.is_ready, true);
        assert_ne!(game.value, Some(5));
        assert_ne!(game.max_value, Some(10));
    }

    #[test]
    fn test_get_agent_distribution() {
        let mut num_agents = 10;
        let mut liar_ratio = 0.5;
        let (mut num_honest, mut num_liars) = Game::get_agent_distribution(num_agents, liar_ratio);
        assert_eq!((num_honest, num_liars), (5, 5));

        num_agents = 6;
        liar_ratio = 0.6;
        (num_honest, num_liars) = Game::get_agent_distribution(num_agents, liar_ratio);
        assert_eq!((num_honest, num_liars), (3, 3));
    }
}
