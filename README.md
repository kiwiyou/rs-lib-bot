<div align="center">
  <h1><code>rs-lib-bot</code></h1>
  <p>
    <image alt="Build and test" src="https://github.com/kiwiyou/rs-lib-bot/workflows/Build%20and%20test/badge.svg">
    <a href='https://coveralls.io/github/kiwiyou/rs-lib-bot?branch=master'><img src='https://coveralls.io/repos/github/kiwiyou/rs-lib-bot/badge.svg?branch=master' alt='Coverage Status' /></a>
    <image alt="License" src="https://img.shields.io/badge/license-MIT%20or%20Apache--2.0-brightgreen">
    <image alt="Lines of code" src="https://img.shields.io/tokei/lines/github/kiwiyou/rs-lib-bot">
    <image alt="Heroku deploy status" src="https://heroku-badge.herokuapp.com/?app=rs-lib-bot">
  </p>
  <strong>Look up Rust crates on Telegram!</strong>
</div>

## About

`rs-lib-bot` is a telegram bot built to search or share information of a Rust crate easily.

Though on a very early stage of development, you can talk to [`@rslibbot`](https://t.me/rslibbot) if you want to try it.

## Features

- [x] Search crates with inline queries and view its information page
- [ ] Browse docs.rs with inline buttons

## Deploying on your own

You can build `rs-lib-bot` and run as your own bot.

Before starting, you need several things:

- Telegram bot token, which you can get by talking to [@BotFather](https://t.me/BotFather)
- Public server to run https webhook service for the bot.
  - You can use [ngrok](https://ngrok.com/) in case you are just testing/debugging/etc.
- Running instance of [crate-search-cache](https://github.com/kiwiyou/crate-search-cache)
- Client key for your Sentry project. (optional)

Once you are ready, clone the repository.

```bash
git clone https://github.com/kiwiyou/rs-lib-bot.git
cd rs-lib-bot
```

Then, set environment variables as follows:

```bash
export BOT_TOKEN="123456789:xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
export PORT="8080" # port to bind server on
export WEBHOOK_URL="https://your.public.server" # IMPORTANT: without trailing slash
export SEARCH_URL="https://your.cache.server" # IMPORTANT: without trailing slash
export SENTRY_DSN="https://xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx.ingest.sentry.io/1234567" # optional
```

Now you can run the bot with `cargo run`.

```bash
RUST_LOG=info cargo run --release
RUST_LOG=info cargo run --release --no-default-features # If you don't use Sentry
```

## Something got wrong with the bot!

Please contact me on Telegram [@kiwiyou](https://t.me/kiwiyou) or send an email to [kiwiyou.dev@gmail.com](mailto:kiwiyou.dev@gmail.com).
