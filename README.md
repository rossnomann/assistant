# Assistant

An assistant bot for Telegram

## Installation

Make sure that you have installed PostgreSQL and Redis.

Download binary:

```sh
$ curl -L https://github.com/rossnomann/assistant/releases/download/0.1.0/assistant-0.1.0_x86_64-linux-gnu --output assistant
$ chmod +x assistant
```

Create `config.yaml`:

```yaml
token: 'bottoken'  # Token from BotFather
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

## 0.1.0 (18.02.2022)

- First release.

# LICENSE

The MIT License (MIT)
