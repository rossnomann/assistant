# Assistant

An assistant bot for Telegram

## Installation

Make sure that you have installed PostgreSQL and Redis.

Download binary:

```sh
$ curl -L https://github.com/rossnomann/assistant/releases/download/0.3.0/assistant-0.3.0_x86_64-linux-gnu --output assistant
$ chmod +x assistant
```

Create `config.yaml`:

```yaml
token: 'bot-token'  # Token from BotFather
database_url: postgresql://user:password@localhost:5432/database  # PostgreSQL connection
session_url: redis://127.0.0.1:6379  # Redis connection
users:  # ID of users who has access to this bot
  - 100000000
  - 200000000
  - 300000000
```

If you want to change log level, use [`RUST_LOG`](https://docs.rs/env_logger/0.9.0/env_logger/) environment variable.

Run migrations:

```sh
$ ./assistant config.yaml migrate
```

Start bot:

```sh
$ ./assistant config.yaml start
````

# Changelog

## 0.3.0 (01.01.2024)

- Updated carapax to 0.14.
- Updated redis to 0.24.
- Updated tokio to 1.35.

## 0.2.0 (05.12.2023)

- Updated carapax to 0.13.
- Updated clap to 4.4.
- dotenv replaced by dotenvy.
- Updated redis to 0.23.
- Updated tokio to 1.34.

## 0.1.0 (18.02.2022)

- First release.

# LICENSE

The MIT License (MIT)
