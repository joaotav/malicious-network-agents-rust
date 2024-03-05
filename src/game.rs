use std::io::{self, Write};
use text_colorizer::Colorize;
use tokio::spawn;
use tokio::sync::oneshot;

use crate::agent::Agent;
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
            game_client: Client::new(),
            active_agents: Vec::new(),
        }
    }

    pub fn print_welcome() {
        println!("\n{}\n", ">>>>> Welcome to Liars Lie! <<<<<".bold().green());
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

    // Convert all instances of `Agent` stored in `Game.active_agents` into
    // instances of `AgentConfig` and convert them to JSON
    fn gen_agent_config(&self) -> Result<String, serde_json::Error> {
        let mut agents_config = Vec::new();
        for agent in &self.active_agents {
            agents_config.push(agent.to_config());
        }
        serde_json::to_string_pretty(&agents_config)
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
                self.game_client.keys.get_public_key().to_owned(),
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
                self.game_client.keys.get_public_key().to_owned(),
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

    /// A setter function for `Game.value`
    // May not be idiomatic Rust, see: https://www.reddit.com/r/rust/comments/d7w6n7
    fn set_value(&mut self, value: u64) {
        self.value = Some(value);
    }

    /// A setter function for `Game.max_value`
    fn set_max_value(&mut self, max_value: u64) {
        self.max_value = Some(max_value);
    }

    /// A setter function for `Game.is_ready`
    fn set_ready(&mut self) {
        self.is_ready = true;
    }

    /// Asynchronously spawns tasks for the game agents in `Game.active_agents`. Waits for
    /// the initialization of all agents before continuing execution.
    async fn start_game_agents(&self) {
        let mut ready_signals = Vec::new();

        for agent in &self.active_agents {
            // Use a oneshot channel to wait for agents to be spawned
            let (signal_transmitter, signal_receiver) = oneshot::channel();
            let agent = agent.clone();
            spawn(async move {
                agent.start_agent(signal_transmitter).await;
            });
            ready_signals.push(signal_receiver);
        }

        // Wait for all tasks to finish their attempt at spawning an agent
        for signal_receiver in ready_signals {
            signal_receiver.await;
        }

        println!("{}\n", "[+] Finished spawning game agents.".bold());
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

        // OBS: An improvement would be to shuffle the values or ids of agents in
        // active_agents to prevent honest agents and liars from being identified
        // by looking at the agents.config file. E.g, given a vector with agent_ids
        // in an increasing order, the first half of agents all have the same value (honest)
        // and the second half all have different values (liars)
        self.add_honest_agents(value, num_honest);
        self.add_liar_agents(value, max_value, num_liars);

        let agent_config = match self.gen_agent_config() {
            Ok(agent_config) => agent_config,
            Err(e) => {
                self.reset_game();
                println!("error: failed to generate agent configuration - {}", e);
                return;
            }
        };

        if let Err(e) = Self::write_agent_config(&agent_config) {
            self.reset_game();
            println!("error: failed to write agents.config file - {}", e);
            return;
        }

        self.start_game_agents().await;
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

        let agent_config = match Client::read_agent_config() {
            Ok(agent_config) => agent_config,
            Err(e) => {
                println!("error: failed to read agents.config file - {}\n", e);
                return;
            }
        };

        if let Err(e) = self.game_client.load_agent_config(&agent_config) {
            println!("error: failed to load data from agents.config - {}\n", e);
            return;
        }

        println!("{}", "[+] Querying agents for their values...\n".bold());

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
    pub fn stop(&self) {
        todo!()
    }

    /// Executes the `extend` command. The `extend` command checks for the existence of
    /// the `agents.config` file, and if present, extends it by launching new honest
    /// agents and liars.
    pub fn extend(&mut self, num_agents: u16, liar_ratio: f32) {
        if !self.is_ready() {
            Game::print_not_started();
            return;
        }
        todo!()
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

    /// Executes the `kill` command. The `kill` command receives an agent ID as an argument
    /// and kills the corresponding agent, but does not modify the `agents.config` file.
    pub fn kill(&mut self, agent_id: u16) {
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
