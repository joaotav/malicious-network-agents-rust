# Liars Lie

![Rust 1.70](https://img.shields.io/badge/rustc-1.70%2B-blue.svg)

Liars lie is a game where a client queries a set of agents about an integer value. A configurable subset of agents tells the truth and will reveal the true integer value when asked. On the other hand, a subset of the participants are liars and behave dishonestly. Each liar responds to queries with a random value, but always the same value. The game's challenge is determining the true integer value, called the **_network value_**, by connecting to the game's agents over the network and querying them for their individual values.

## Prerequisites

This project requires the Rust programming language to compile. If you do not have Rust installed on your machine, you can download it and find installation instructions at the official Rust website:

[Rust Programming Language - Installation](https://www.rust-lang.org/tools/install)

## Building the project

This project compiles into an executable **_liarslie_** which can be used to play the game. Build the project using `cargo build` and run the executable with `./target/debug/liarslie`.

## The Game

The game is composed by two different modes of play, the _**standard mode**_ and the _**expert mode**_. 

### Standard Mode

In _**standard mode**_, the game launches a number of independent agents as network nodes, where a configurable number of agents are honest and others are dishonest/liars, and stores their information in the `agents.config` file. The game's client will then read the configuration file and attempt to connect to every agent to query it for its value. Once the client collects all valid replies, it determines and displays the **_network value_**.

- Each agent is aware only of its own identity and value. Agents may learn values or identities of other agents solely by adhering to the game's protocol.
- The `agents.config` file contains agent identities and public keys but not their values, ensuring network values cannot be deduced from this file.
  
### Expert Mode

In _**expert mode**_, the game's objective is the same as in _**standard mode**_ but additional constraints are introduced. The game's client can only communicate directly with a subset of the agents whose information is listed in the `agents.config` file, while the remaining agents are unreachable. In this mode, the client must leverage the connection with the available agents to retrieve the values from every agent in the network. Dishonest agents may also attempt to modify other agents' messages, requiring the client to authenticate received messages to ensure they are valid before determining the **_network value_**. 

Finally, agents might also be killed and become unreachable, leading to network failures. The client must also be able to overcome this in order to retrieve all agents' values.

## Commands

#### Starting the game

``` start --value v --max-value max --num-agents number --liar-ratio ratio ```

This command launches a network of agents with a specified number of honest and liar agents based on the liar-ratio. Honest agents always respond with the integer value v, while liar agents respond with x such that x != v and 1 <= x <= max.

#### Playing a standard round

```play```

The client reads the `agents.config` file, connects to the agents, plays a round, and prints the **_network value_**.

#### Killing an agent

```kill --id agentid```

This command removes the specified agent from the network, but keeps its information in the `agents.config` file.

#### Stopping the game

```stop```

This stops all agents listed in `agents.config`, removes their information from the file, and exits the executable.

#### Extending the game

```extend --num-agents number --liar-ratio ratio```

This command checks for the existence of `agents.config` and extends the network by launching the specified agents, appending their information into `agents.config`.

#### Playing an expert round

```play-expert --num-agents number --liar-ratio ratio```

The client receives the addresses of a randomly selected subset of agents, with a distribution of honest/liar agents according to the specified parameters. The client connects to the agents, queries their values and the values of other unreachable agents and prints the **_network value_**.
