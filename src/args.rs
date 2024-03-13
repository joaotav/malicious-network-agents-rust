use crate::commands::Commands;
use clap::Parser;

#[derive(Parser, Debug, PartialEq)]
#[command(
    about,
    no_binary_name(true),
    override_usage("<COMMAND> [COMMAND-SPECIFIC-ARGS]")
)]
/// This program implements 'Liars Lie', a game in which a client queries a set of agents about an
/// integer value. A subset of the agents tells the truth and will reveal the true value when asked.
/// Liars, on the other hand, will respond with an arbitrary value. The challenge lies in determining
/// the true value by querying the agents for their individual values.
pub struct Args {
    #[command(subcommand)]
    /// An user issued command
    command: Commands,
}

impl Args {
    /// Returns a copy of the instance of `Commands` in `Args.command``
    pub fn get_command(&self) -> Commands {
        self.command.clone()
    }

    /// Receives a string slice `user_input` and attempts to parse it according
    /// to the rules defined by the program's parser.
    pub fn parse_args(user_input: &str) -> Result<Self, String> {
        let user_input: Vec<&str> = user_input.split_whitespace().collect();

        match Args::try_parse_from(user_input).map_err(|e| e.to_string()) {
            Ok(args) => {
                // Validate the values passed to the command's options (e.g, valid range and non-zero)
                match args.command.validate_args_values() {
                    Ok(()) => Ok(args),
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
    }
}

// ******************************************************************************************
// ************************************* UNIT TESTS *****************************************
// ******************************************************************************************

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_start_command() {
        let input =
            "start --value 5 --max-value 8 --num-agents 5 --liar-ratio 0.2 --tamper-chance 0.35";
        assert_eq!(
            Ok(Args {
                command: Commands::Start {
                    value: 5,
                    max_value: 8,
                    num_agents: 5,
                    liar_ratio: 0.2,
                    tamper_chance: 0.35,
                }
            }),
            Args::parse_args(input)
        );

        let incomplete_input = "start --value 5 --max-value 8 --num-agents 5";
        assert!(Args::parse_args(incomplete_input).is_err());
    }

    #[test]
    fn test_parse_play_command() {
        let input = "play";
        assert_eq!(
            Ok(Args {
                command: Commands::Play
            }),
            Args::parse_args(input)
        );

        let wrong_input = "play --id 3";
        assert!(Args::parse_args(wrong_input).is_err());
    }

    #[test]
    fn test_parse_extend_command() {
        let input = "extend --num-agents 5 --liar-ratio 0.6";
        assert_eq!(
            Ok(Args {
                command: Commands::Extend {
                    num_agents: 5,
                    liar_ratio: 0.6
                }
            }),
            Args::parse_args(input)
        );

        let incomplete_input = "extend --liar-ratio 0.5";
        assert!(Args::parse_args(incomplete_input).is_err());
    }

    #[test]
    fn test_parse_playexpert_command() {
        let input = "play-expert --num-agents 2 --liar-ratio 0.5";
        assert_eq!(
            Ok(Args {
                command: Commands::PlayExpert {
                    num_agents: 2,
                    liar_ratio: 0.5
                }
            }),
            Args::parse_args(input)
        );

        let incomplete_input = "extend --num-agents 2";
        assert!(Args::parse_args(incomplete_input).is_err());
    }

    #[test]
    fn test_parse_stop_command() {
        let input = "stop";
        assert_eq!(
            Ok(Args {
                command: Commands::Stop
            }),
            Args::parse_args(input)
        );

        let wrong_input = "stop --value 2";
        assert!(Args::parse_args(wrong_input).is_err());
    }

    #[test]
    fn test_parse_kill_command() {
        let input = "kill --id 5";
        assert_eq!(
            Ok(Args {
                command: Commands::Kill { agent_id: 5 }
            }),
            Args::parse_args(input)
        );

        let incomplete_input = "kill";
        assert!(Args::parse_args(incomplete_input).is_err());
    }
}
