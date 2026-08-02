# discord-pushover-notifier

A Discord bot that forwards messages to [Pushover](https://pushover.net/) as push notifications. Supports slash commands with configurable priority levels and a quick `!` prefix for emergency alerts.

## Features

- **Slash commands** — `/notify` (or `/n`) to send a notification with optional priority, retry, and expiry settings
- **Quick alerts** — prefix any message with `!` to instantly fire an emergency-priority Pushover notification and ping `@everyone`
- **Bot mentions** — mention the bot with a message to send an emergency-priority Pushover notification
- **Priority levels** — Lowest, Low, Normal, High, and Emergency (with configurable retry/expire for Emergency)
- **Role gating** — only members with a configured Discord role can trigger notifications
- **Group link** — `/group` command shares your Pushover group invite link
- **Automatic retries** — failed Pushover API calls are retried with Fibonacci backoff
- **Logging** — daily rotating log files via ftail (10 MB max, 1-day retention)
- **Docker support** — multi-stage Dockerfile with process-based healthcheck

## Setup

### Prerequisites

- Rust 1.97.1+ (2024 edition) or Docker
- A [Pushover](https://pushover.net/) account with an application token
- A [Discord bot](https://discord.com/developers/applications) token with the Message Content privileged intent enabled

### Environment variables

Copy `.env.example` to `.env` and fill in the values:

| Variable | Description |
|---|---|
| `DISCORD_TOKEN` | Your Discord bot token |
| `PUSHOVER_TOKEN` | Pushover application API token |
| `PUSHOVER_KEY` | Pushover user/group key |
| `GROUP_LINK` | Pushover group invite URL (shown by `/group`) |
| `NOTIFIER_ROLE_ID` | Discord role ID required to use the bot |

### Run locally

```bash
cp .env.example .env
# fill in .env
cargo run --release
```

### Run with Docker

```bash
docker build -t discord-pushover-notifier .
docker run -d --env-file .env --name pushover-bot discord-pushover-notifier
```

## Usage

| Command | Description |
|---|---|
| `/notify <message> [priority] [retry] [expire]` | Send a Pushover notification |
| `/n <message> [priority] [retry] [expire]` | Alias for `/notify` |
| `/group` | Show the Pushover group invite link |
| `!<message>` | Quick emergency notification (30s retry, 15m expire) + `@everyone` ping |
| `@bot <message>` | Emergency notification (30s retry, 15m expire) |

Priority defaults to **Emergency** if omitted. For Emergency priority, `retry` (minimum 30s) and `expire` (maximum 10800s / 3 hours) can be customized; they default to 30s and 15 minutes respectively.

## License

This project is licensed under the GNU General Public License v3.0. See [LICENSE](LICENSE) for details.
