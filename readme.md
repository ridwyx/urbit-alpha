# Urbit Alpha Chatbot

Get crypto charts in your Urbit channels instantly. **Current state: POC**.

## Development: How to set up

This bot is designed to run on a moon. You may want to run it persistently on your server of choice. Additionally, you will need to have Rust installed.

1. Copy `.env-example` into `.env` and add your own AWS credentials. You'll need an S3 bucket with public read permissions and a IAM user with permissions to write to that bucket. S3 is used to store images of charts.
2. On your first `cargo run` the app will create a demo `ship_config.yaml` file. Fill that file with info about the ship you want to use for the bot (can be a moon) .
3. On your second `cargo run` the app should connect to the moon and start listening for messages.
4. To accept payments make sure that bitcoin-wallet is configured to a working provider node. 

I needed to invite the moon to a channel so I can issue the commands.

## The commands

Similar to [Alpha Bot on Discord](https://www.alphabotsystem.com/), you can write `c <symbol> <timeframe>` and get a screenshot of a TradingView chart.

![Screenshot](https://ridwyx-storage.s3.eu-west-2.amazonaws.com/screenshot.png)

## TODO

Current phase

- [x] Non crypto exchanges
- [x] Open image without prompting download
- [x] Spawn chatbots in any chat
- [x] Help menu
- [x] Helpful errors
- [x] Add welcome message (this means listening for invites to groups & joining channels)
- [x] Clean up the code

Next phase

- [ ] Add tests (?)
- [ ] Accept payments
- [ ] Build out premium features (?)
- [ ] Optimize bot reply time – currently takes up to 10 sec

Conceptual

- [ ] Figure out cloud hosting
- [ ] Figure out S3 costs (maybe automatic cleanup of images older than 1 month?)
- [ ] Figure out a proper roadmap

Bugs

- [ ] c btcusd 1mo: doens't show enough history (binance)
