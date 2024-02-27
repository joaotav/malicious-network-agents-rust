use clap::Subcommand;

/// Represents an user issued command along with its associated arguments.
/// All arguments must be passed using the long format, e.g, --value.
///
/// # Example
/// ```
/// let command = Commands::Start { value: 5, max_value: 8, num_agents: 5, liar_ratio: 0.2};
/// ```
#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum Commands {
    /// Launches agents and generates the agents.config file (requires additional arguments)
    Start {
        /// A positive value to be reported by honest participants when queried
        #[arg(long)]
        value: u64,

        /// The maximum, positive value that can be reported by liars
        #[arg(long)]
        max_value: u64,

        /// The number of agents that will participate in the game
        #[arg(long)]
        num_agents: u16,

        /// The ratio of liars (0.0 to 1.0) among the specified number of agents
        #[arg(long)]
        liar_ratio: f32,
    },
    /// Plays a round of the game on standard mode
    Play,
    /// Extends the set of available agents (requires additional arguments)
    Extend {
        /// The number of new agents to be spawned
        #[arg(long)]
        num_agents: u16,

        /// The ratio of liars (0.0 to 1.0) among the newly spawned agents
        #[arg(long)]
        liar_ratio: f32,
    },
    /// Plays a round of the game on expert mode (requires additional arguments)
    PlayExpert {
        /// The maximum number of agents that the client is allowed to directly query
        #[arg(long)]
        num_agents: u16,

        /// The ratio (0.0 to 1.0) of dishonest agents among the specified number of agents
        #[arg(long)]
        liar_ratio: f32,
    },
    /// Stops the game and quits the program
    Stop,
    /// Kills an specified agent (requires additional arguments)
    Kill {
        /// The ID of the agent to be killed
        #[arg(long = "id")]
        agent_id: u16,
    },
}

impl Commands {
    /// Receives a variant of `Commands` and checks whether it contains the `liar_ratio`
    /// field or not. If it does, returns the value contained in `liar_ratio`.
    fn has_liar_ratio(&self) -> Option<f32> {
        match self {
            Commands::Start { liar_ratio, .. }
            | Commands::Extend { liar_ratio, .. }
            | Commands::PlayExpert { liar_ratio, .. } => Some(*liar_ratio),
            Commands::Play { .. } | Commands::Stop { .. } | Commands::Kill { .. } => None,
        }
    }

    /// Receives a variant of `Commands` and checks whether it contains the `value` and
    /// `max_value` fields or not. If it does, returns `(value, max_value)`.
    fn has_agent_values(&self) -> Option<(u64, u64)> {
        match self {
            Commands::Start {
                value, max_value, ..
            } => Some((*value, *max_value)),
            Commands::Play { .. }
            | Commands::Extend { .. }
            | Commands::PlayExpert { .. }
            | Commands::Stop { .. }
            | Commands::Kill { .. } => None,
        }
    }

    /// Receives a variant of `Commands` and checks whether it contains the `num_agents`
    /// field or not. If it does, returns the value contained in `num_agents`.
    fn has_num_agents(&self) -> Option<u16> {
        match self {
            Commands::Start { num_agents, .. }
            | Commands::Extend { num_agents, .. }
            | Commands::PlayExpert { num_agents, .. } => Some(*num_agents),
            Commands::Play { .. } | Commands::Stop { .. } | Commands::Kill { .. } => None,
        }
    }

    fn has_agent_id(&self) -> Option<u16> {
        // REVIEW: This function may be unecessary
        todo!();
    }

    /// Receives a variant of `Commands``, check for, and test all possible arguments to ensure
    /// that they satisfy the program's constraints.
    pub fn validate_args_values(&self) -> Result<(), String> {
        self.validate_liar_ratio()?;
        self.validate_agent_values()?;
        self.validate_num_agents()?;
        Ok(())
    }

    /// Receives a variant of Commands and, if it contains the `liar_ratio` field,
    /// checks if the value of liar_ratio is within the range [0.0, 1.0].
    fn validate_liar_ratio(&self) -> Result<(), String> {
        match self.has_liar_ratio() {
            Some(liar_ratio) if (0.0..=1.0).contains(&liar_ratio) => Ok(()),
            Some(_) => Err(
                "error: --liar-ratio must be within the range of 0.0 to 1.0 (inclusive)\n"
                    .to_string(),
            ),
            None => Ok(()),
        }
    }

    /// Receives a variant of `Commands` and, if it contains the `value` and `max_value` fields,
    /// checks if their values satisfy the program's constraints:
    ///
    /// * `value` must be greater than 0 as (1 <= liar_value <= max_value).
    ///   If value == 0, honest agents can be easily distinguished from liars.
    ///
    /// * `value` cannot be larger than `max_value`.
    ///
    /// * `max_value` cannot be equal to 1, since this would cause both `value`
    ///   (which cannot be 0) and `liar_value` (which cannot be 0 or equal to `value`) to
    ///    be equal to 1.
    fn validate_agent_values(&self) -> Result<(), String> {
        let (value, max_value) = match self.has_agent_values() {
            Some((value, max_value)) => (value, max_value),
            None => return Ok(()),
        };

        if value == 0 {
            return Err("error: --value must be greater than 0\n".to_owned());
        }

        if value > max_value {
            return Err("error: --value cannot be greater than --max-value\n".to_owned());
        }

        if max_value == 1 {
            return Err("error: --max-value must be greater than 1\n".to_owned());
        }

        Ok(())
    }

    /// Receives a variant of `Commands` and, if it contains the `num_agents` field,
    /// checks if `num_agents`` > 0
    fn validate_num_agents(&self) -> Result<(), String> {
        match self.has_num_agents() {
            Some(num_agents) if (num_agents > 0) => Ok(()),
            Some(_) => Err("error: --num-agents must be greater than 0\n".to_owned()),
            None => Ok(()),
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
    fn rejects_invalid_liar_ratio() {
        let case1 = Commands::Start {
            value: 5,
            max_value: 8,
            num_agents: 5,
            liar_ratio: 2.0,
        };
        assert!(case1.validate_liar_ratio().is_err());

        let case2 = Commands::Start {
            value: 5,
            max_value: 8,
            num_agents: 5,
            liar_ratio: -0.1,
        };
        assert!(case2.validate_liar_ratio().is_err());
    }

    #[test]
    fn rejects_invalid_num_agents() {
        let command = Commands::Start {
            value: 5,
            max_value: 8,
            num_agents: 0,
            liar_ratio: 0.5,
        };
        assert!(command.validate_num_agents().is_err());
    }

    #[test]
    fn rejects_invalid_value_and_max_value() {
        // Should throw an error because value = 0
        let case1 = Commands::Start {
            value: 0,
            max_value: 8,
            num_agents: 5,
            liar_ratio: 0.5,
        };
        assert!(case1.validate_agent_values().is_err());

        // Should throw an error because value > max_value
        let case2 = Commands::Start {
            value: 3,
            max_value: 2,
            num_agents: 5,
            liar_ratio: 0.5,
        };
        assert!(case2.validate_agent_values().is_err());

        // Should throw an error because max_value = 1
        let case3 = Commands::Start {
            value: 1,
            max_value: 1,
            num_agents: 5,
            liar_ratio: 0.5,
        };
        assert!(case3.validate_agent_values().is_err());
    }
}
