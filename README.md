# 🦀 rustedbot

![Written in Rust](https://img.shields.io/badge/-Rust-F74C00?style=square&logo=rust&logoColor=white)
[![Documentation](https://img.shields.io/badge/documentation-yes-brightgreen.svg)](https://github.com/mxthevs/rustedbot#readme)
![License: GPL 3.0](https://img.shields.io/badge/License-GPLv3.0-blue.svg)

## What this is?

rustedbot is a Twitch.tv chatbot 🤗. It can be used to moderate your chat, interact with your viewers and much more!

## Running the bot 

1. Clone the repository
2. Rename the `bot.conf.example` file to `bot.conf` and fill in the values
3. Rename the `.env.example` file to `.env` and fill in the values
3. Build the bot with `make build`
4. Run the bot with `./target/release/rustedbot ./bot.conf`

```bash
git clone https://github.com/mxthevs/rustedbot.git
cd rustedbot
cargo build --release
cp bot.conf.example bot.conf # Fill in the values
cp .env.example .env # Fill in the values
make build
make run
```

## Contributing

If you have any problem, feel free to file an issue and if you are willing to contribute to the project, open a pull request as well!

_Released under GPL-3.0 License_
