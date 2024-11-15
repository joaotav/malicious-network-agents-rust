use liarslie::args::Args;
use liarslie::commands::Commands;
use liarslie::game::Game;

#[tokio::main]
async fn main() {
    let mut game = Game::new();
    Game::print_welcome();

    loop {
        let user_input = match Game::get_user_input() {
            Ok(user_input) => user_input,
            Err(e) => {
                println!("error: failed to read user input - {}\n", e);
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
                    tamper_chance,
                } => {
                    game.start(value, max_value, num_agents, liar_ratio, tamper_chance)
                        .await
                }
                Commands::Play => game.play().await,
                Commands::Stop => game.stop().await,
                Commands::Extend {
                    num_agents,
                    liar_ratio,
                } => game.extend(num_agents, liar_ratio).await,
                Commands::PlayExpert {
                    num_agents,
                    liar_ratio,
                } => game.play_expert(num_agents, liar_ratio).await,
                Commands::Kill { agent_id } => game.kill(agent_id).await,
            },

            Err(e) => {
                println!("{}", e);
                continue;
            }
        };
    }
}
