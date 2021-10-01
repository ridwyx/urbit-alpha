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

[ ] Non crypto exchanges
[ ] Non 
[ ] Improve the TradingView screenshot format (dimensions, zoom) to better fit the chat UI
[x] open image without prompting download
[ ] Clean up the code
[ ] Add tests (?)
[ ] Spawn chatbots in any chat. What is the setup process?
[ ] Help menu
[ ] accept payments
[ ] helpful errors

conceptual
[ ] 3 tyes of customers
[ ] Figure out cloud hosting
[ ] Figure out S3 costs (maybe automatic cleanup of images older than 1 month?)
[ ] Figure out a proper roadmap


Bug
[ ] c btcusd 1mo: doens't show enough history (binance)


@mikeosborne could you add the following to the milestones section?

=================

Milestone 1: Bot prototype + Moon deployment
Details: 
- Create chatbot that responds to incoming messages in the format of "c {ticker} {time interval}" and return a chart image with relevant financial data.
- The bot should fetch data for any asset supported by TradingView API, such as stocks, crypto, forex, or metals.
- Deploy this prototype bot to a Moon.
Estimated Completion: Done

Milestone 2: Ensure customers can add Cypher bot to their chats with ease.
Details: 
- Modify the chatbot framework we're using so that our Moon reacts when it is added to a new chat. When added, the bot displays a welcome message + menu.
- Add "c help" command for a full list of the bot's features
- Ensure improper inputs like "c nonexistentTicker" returns a helpful error message.
Estimated completion: 1 week

Milestone 3: Add tests
Details: Add unit tests for command parsing, AWS, and image rendering logic.
Estimated completion: 1 week

==============

The following might be out of this contract's scope.

Milestone 4: Add paid tier to pay for S3 bucket
Details: 
- Allow bot to accept payment to the Moon it is hosted on to pay for upgraded features.
- Create chat interface so users of our bot can upgrade or downgrade seamlessly.
Estimated completion: 2 weeks