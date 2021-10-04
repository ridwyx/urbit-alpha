# Urbit Alpha Chatbot

Get crypto charts in your Urbit channels instantly. **Current state: POC**.

## Development: How to set up

1. Copy `.env-example` into `.env` and add your own AWS credentials. You'll need an S3 bucket with public read permissions and a IAM user with permissions to write to that bucket.
2. On your first `cargo run` the app will create a demo `ship_config.yaml` file. Fill that file with info about the ship you want to use for the bot (can be a moon) .
3. On your second `cargo run` the app should connect to the moon and start listening for messages.

I needed to invite the moon to a channel so I can issue the commands.

## The commands

Similar to Alpha bot on Discord, you can write `c <symbol> <timeframe>` and get a screenshot of a TradingView chart.

![Screenshot](https://ridwyx-storage.s3.eu-west-2.amazonaws.com/screenshot.png)

## TODO

[x] Non crypto exchanges
[ ] Improve the TradingView screenshot format (dimensions, zoom) to better fit the chat UI
[x] open image without prompting download
[ ] Clean up the code
[ ] Add tests (?)
[ ] Spawn chatbots in any chat. What is the setup process?
[ ] Help menu
[ ] accept payments
[ ] helpful errors

conceptual
[ ] Figure out cloud hosting
[ ] Figure out S3 costs (maybe automatic cleanup of images older than 1 month?)
[ ] Figure out a proper roadmap


Bug
[ ] c btcusd 1mo: doens't show enough history (binance)

