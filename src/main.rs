use crate::args::Args;
use crate::client::Client;
use crate::commands::Commands;
use crate::game::Game;

mod agent;
mod agent_config;
mod args;
mod client;
mod commands;
mod game;

fn main() {
    let mut game = Game::default();
    Game::print_welcome();

    loop {
        let user_input = match Game::get_user_input() {
            Ok(user_input) => user_input,
            Err(e) => {
                println!("error: failed to read user input.\n");
                continue;
            }
        };

        match Args::parse_args(&user_input) {
            Ok(args) => match args.get_command() {
                Commands::Start {
                    value,
                    max_value,
                    num_agents,
                    liar_ratio,
                } => game.start(value, max_value, num_agents, liar_ratio),
                Commands::Play => game.play(),
                Commands::Stop => game.stop(),
                Commands::Extend {
                    num_agents,
                    liar_ratio,
                } => game.extend(num_agents, liar_ratio),
                Commands::PlayExpert {
                    num_agents,
                    liar_ratio,
                } => game.play_expert(num_agents, liar_ratio),
                Commands::Kill { agent_id } => game.kill(agent_id),
            },

            Err(e) => {
                println!("{}", e);
                continue;
            }
        };
    }
}

// ******************************************************************************************
// ************************************* UNIT TESTS *****************************************
// ******************************************************************************************