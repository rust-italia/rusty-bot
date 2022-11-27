# Rusty bot

This is a very simple Telegram Bot made in Rust for Rust Telegram groups.

## Running

There are some environment variables that are used by the executable:

* `TELOXIDE_TOKEN` (required): this is the token of your bot obtain from
  [BotFather](https://telegram.me/BotFather).
* `HOST` (required): the name of the host that is going to host the bot.
* `PORT`: the port of the HTTP server. By default it's 8080 if the `tls` feature
  is disabled, 443 when enabled (see below for information about `tls`).
* `RUST_LOG`: the log level for `tracing` crate.
* `USE_POLLING`: force polling instead of websockets to interact with Telegram
  APIs.


### Using TLS

You can use TLS instead of unencrypted HTTP enabling the `tls` feature. If you
do so, the default port is changed to 443 and you need to specify two more
environment variables:

* `SSL_CERT`: the path to the full chain certificate file in PEM format.
* `SSL_KEY`: the path to the private key file in PEM format.


### Deploy to fly.io

This repo contains the bare minimum to deploy the service to
[fly.io](https://fly.io). You need to [perform a basic setup]
(https://fly.io/docs/languages-and-frameworks/dockerfile/) in order to select
the app you are going to deploy. Moreover, you also need to config the `HOST`
and `TELOXIDE_TOKEN` secrets before running `fly deploy`.
