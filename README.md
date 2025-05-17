# RustMP

**Team members:**  
Veronika Bei  
Kostiantyn Cherniakov  
(Team FX)  

---

## Launch guide
In order to launch the project succesfully, due to multiple binaries in the project, two commands are needed:

`cargo build --bins`

`cargo run --bin rust_mp`

After these two commands are executed, a launcher window will open, where the user may choose a character at the bottom of the screen, as well as wether they want to host the game or to join one, in which case they need to input a valid IP.

## Disclaimers

### Fair use disclaimer
This project uses code and assets(most notably for the map) based on one of the examples in the macroquad repository (https://github.com/not-fl3/macroquad), licensed under the MIT License.

### Project success disclaimer
Due to the lack of time and experience on the sides of both people working on the project, this project still has some issues, most notably some visual bugs in the game and one issue with syncronization. This project also did not meet the original scope set at the start, for the reasons mentioned above.

Additionally, due to the insistance of the team member responsible for the graphics, the game's visual style changed from top-down to side view.



## Zámer projektu / Project intent

### RustMP

We would like to create a multiplayer top-down-shooter style PvP game, where players can select a playable character and enter a lobby to fight other participating players. The main goal of the game is to provide a dynamic and responsive gameplay experience with reasonably low connection issues on a local network.

We aim to address the challenge of network synchronization in fast-paced PvP games by implementing efficient data transmission algorithms and optimizing packet usage. One of the key objectives is to provide frequent object state updates while minimizing network load.

During development, we plan to explore and implement modern approaches to multiplayer gaming, including:

- Optimization of data transmission over UDP
- Motion smoothing algorithms
- RPC calls for network interaction
- Balancing of gameplay systems and interactions

This project will give us valuable experience in network programming, asynchronous systems, game development and general knowledge of low-level programming in larger projects.

---

## Requirements

### Core Features

- **Multiplayer PvP:** Players can enter lobbies and fight in real-time.
- **Top-down perspective:** The game uses a 2D top-down view, which shifts focus from visual to networking and development challenges.
- **Character selection:** Players select different characters before battles, each with their own unique abilities.
- **Client-Server Networking:** UDP-based communication for fast data transfer.
- **Entity Synchronization:** Position, rotation, and movement updates optimized with linear interpolation and similar methods.
- **RPC System:** Remote commands for executing actions across different devices.
- **Packet Optimization:** Minimize redundant messages to avoid overloading networks and players' computers.

---

### Success Criteria

- Stable online matches with minimal desynchronization.
- Efficient network packet handling to reduce latency.
- A simple yet enjoyable gameplay experience.

---

## Dependencies

We will use Rust and relevant crates for game networking and development. Some key dependencies:

- **Game Engine:** `macroquad` or `bevy` (specific crate TBD)
- **Serialization:** `serde`, `bincode`
- **Networking**: Standard Rust libraries, with potential for external crates later if necessary

---

## Architecture Overview

### Network Architecture

- **Client-Server Model:** The server manages game logic and state synchronization across a number of clients.
- **UDP Communication:** Faster transmission of movement and action updates.
- **HashMap-based Message Storage:** Key-value storage for different message types.
- **RPC Calls:** Allows efficient execution of remote commands.
- **Interpolation:** Smooth movement transitions to minimize lag effects and decrease network load.

---

## What We Hope to Learn

During the development of the game, we plan to deepen our knowledge and skills in the following areas:

- **Network Programming:** Working with UDP, implementing client-server architecture, and managing data synchronization between clients to ensure fast and accurate information transfer of various types and purposes.
- **Game Mechanics Optimization:** Applying interpolation and motion prediction algorithms to ensure a smooth gameplay experience, minimize delays, and improve the responsiveness of player controls.
- **General Game Design:** Creating an enjoyable experience for the player and balancing of the combat system to provide a variety of gameplay strategies and a fair game environment.
- **Working with Rust:** Using and testing various crates for working with networking, graphics, and game logic, developing skills in Rust programming, particularly in multithreading and asynchronous programming.
