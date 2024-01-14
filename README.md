# ProjectLily-rs

This is an experimental project for creating an artificial intelligence agent capable of developing an evolving, long-term personality, with access to reading and writing long term memories, and internal thoughts and monologue frequently.

The goal of this project is to prioritize realism and consistency of the bot, rather than compliance. The bot should prefer to make it's own decisions that align more with it's personality and known memories, more so than obeying commands. In addition, this project seeks to make the agent work using only quantized, local language models as much as possible. Thought larger, commercial language models might be supported as well.

This project takes heavy inspirations from [MemGPT](https://github.com/cpacker/MemGPT) and [AutoGen](https://github.com/microsoft/autogen). Though, ProjectLily does not incorporate these due to a having a different goal set in mind.

## Getting Started

### Building

This project requires the torch library to be installed. You can review possible installation alternatives, [here](https://github.com/LaurentMazare/tch-rs).

***Steps:***

1. Download LibTorch `2.0.0-cu118`
    - <https://download.pytorch.org/libtorch/cu118/libtorch-cxx11-abi-shared-with-deps-2.0.0%2Bcu118.zip>

2. Clone the repository
    - `git clone https://github.com/TheDudeFromCI/ProjectLily-rs`

3. Enter the new project folder
    - `cd ProjectLily-rs`

4. Export binary paths within your shell that point to LibTorch
    - `export LIBTORCH="/home/user/libtorch/2.0.0-cu118/"`
    - `export LD_LIBRARY_PATH="$LIBTORCH/lib"`

5. Run Cargo with a pointer to your agent file.
    - `cargo run -- --agent /path/to/agent.json`
