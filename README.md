# Shiori

**A release tracker for anime and other content, built with Rust.**

[![Language](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Shiori is a work-in-progress release tracking application that retrieves release information from external providers, stores changes, and delivers notifications through Telegram.

The project is being developed incrementally as a practical way to learn Rust, from API integration and asynchronous programming to persistence and application architecture.

## About the Project

Keeping up with releases means checking multiple sources and noticing when information changes. Shiori aims to bring that information together and notify users when tracked content is new or updated.

The application currently focuses on anime data from AniList. Its provider-oriented design is intended to make it possible to integrate additional sources as the project evolves.

Shiori is also a hands-on Rust learning project. Features are introduced progressively, with an emphasis on understanding the design and implementation rather than building everything at once.

## Features

The following features are implemented in the current version:

- **AniList integration** — retrieves anime release information through the AniList GraphQL API.
- **Release persistence** — stores release information in PostgreSQL.
- **Change detection and history** — compares incoming data with stored records and records detected changes.
- **Season association** — associates releases with season records in the database.
- **Notification queue** — stores notification events for processing.
- **Telegram notifications** — sends notifications for supported release events.
- **Periodic processing** — runs AniList synchronization every six hours and processes pending notifications every 30 seconds. Both loops begin immediately at startup.
- **Modular application structure** — separates clients, providers, repositories, services, configuration, and application orchestration.

## Planned Features and Roadmap

Shiori is actively evolving. The checklist below distinguishes future work from the current implementation.

### Phase 1 — Code Quality and Testing

- [ ] Add documentation to source files, modules, structs, enums, and functions where appropriate.
- [ ] Improve internal documentation and code organization.
- [ ] Add unit and integration tests.
- [ ] Improve error handling and application reliability.
- [ ] Introduce automated code quality checks.

### Phase 2 — CI/CD and Deployment

- [ ] Set up GitHub Actions workflows.
- [ ] Automate formatting, linting, and testing.
- [ ] Explore building and publishing release binaries.
- [ ] Host the application so it can run continuously, 24/7.
- [ ] Evaluate a deployment strategy for a Rust application, considering reliability, cost, and maintenance.

### Phase 3 — External Data Integration

- [ ] Evaluate and integrate TMDB (The Movie Database), if appropriate for the project's scope.
- [ ] Explore additional release data providers.
- [ ] Improve metadata retrieval and synchronization.

### Phase 4 — Seasonal Organization

- [ ] Introduce seasonal organization for releases.
- [ ] Support Winter, Spring, Summer, and Autumn seasons.
- [ ] Allow users to navigate between seasons and view their associated releases.
- [ ] Track the current season and its relevant releases.
- [ ] Organize Telegram notifications and chat topics according to the selected season.

### Phase 5 — Telegram Improvements

- [ ] Improve notification formatting and readability.
- [ ] Organize notifications by season.
- [ ] Explore dedicated Telegram topics for different content categories.
- [ ] Support the existing topic plan: All, AnimeList, and Others.
- [ ] Consider additional topic organization as the project evolves.

### Phase 6 — Release Tracking and Status Management

- [ ] Track the status of each entry.
- [ ] Add filters for Watching, Completed, Pending, On Hold, Hiatus, Dropped, and Upcoming.
- [ ] Allow tracked content to be filtered and organized by status.
- [ ] Explore release progress tracking, including identifying new episodes or updates.

## Technology Stack

### Currently used

- **Rust** — application language.
- **Tokio** — asynchronous runtime.
- **Reqwest** — HTTP client.
- **Serde and Serde JSON** — serialization and deserialization.
- **SQLx** — PostgreSQL access and database migrations.
- **PostgreSQL** — persistent storage.
- **Chrono** — date and time handling.
- **dotenvy** — loading environment variables from a `.env` file.
- **async-trait** — asynchronous provider abstraction.

### Under consideration

- **TMDB** — a potential additional data provider.
- **GitHub Actions** — planned automation for quality checks and CI/CD.
- **Deployment platform** — to be evaluated based on reliability, cost, and maintenance requirements.

## Getting Started

### Prerequisites

- A Rust toolchain compatible with the project's edition.
- PostgreSQL.
- An AniList API connection.
- A Telegram bot and destination chat.
- The Telegram topic ID used for AniList notifications.

### Installation

Clone the repository and enter the project directory:

```bash
git clone <repository-url>
cd shiori
```

### Database setup

Create a PostgreSQL database and apply the SQL migration files in the `migrations/` directory before starting the application.

The application connects to PostgreSQL using `DATABASE_URL`.

### Configuration

Shiori loads configuration from environment variables and can read them from a `.env` file in the project directory.

Configure the following values:

| Variable | Purpose |
| --- | --- |
| `DATABASE_URL` | PostgreSQL connection string |
| `TELEGRAM_BOT_TOKEN` | Telegram bot token |
| `TELEGRAM_CHAT_ID` | Telegram destination chat |
| `TELEGRAM_ANIMELIST_THREAD_ID` | Telegram topic ID for AniList notifications |

Although the topic ID is optional in the configuration structure, the current bootstrap process requires it to start the application.

Keep secrets in your local environment or `.env` file. Do not commit credentials or tokens to the repository.

### Run

Start the application with Cargo:

```bash
cargo run
```

Shiori begins its synchronization and notification-processing loops immediately after startup.

## Project Structure

The application is organized by responsibility:

```text
src/
├── main.rs
├── bootstrap.rs
├── application/
│   ├── mod.rs
│   └── runner.rs
├── config/
│   └── mod.rs
├── clients/
├── database/
├── models/
├── provider/
├── repositories/
└── services/
migrations/
```

| Component | Responsibility |
| --- | --- |
| `main.rs` | Application entry point and Tokio runtime |
| `bootstrap.rs` | Loads configuration and constructs application dependencies |
| `application/` | Coordinates application execution and periodic work |
| `config/` | Loads and validates environment-based configuration |
| `clients/` | External API and HTTP clients |
| `database/` | Database connection setup |
| `models/` | Application and data-transfer models |
| `provider/` | Abstraction for retrieving release data from providers |
| `repositories/` | Database persistence and retrieval |
| `services/` | Synchronization, notification, and application-level business logic |
| `migrations/` | SQL schema migrations |

## Learning Objectives

Shiori provides a practical environment for exploring Rust concepts, including:

- Ownership, borrowing, and shared access to dependencies.
- Structs, enums, traits, and trait objects.
- Error handling with `Result` and error propagation.
- Asynchronous programming with Tokio and `async`/`await`.
- Concurrent asynchronous workflows.
- HTTP API integration and JSON serialization.
- Database access, transactions, and persistence with SQLx.
- Modular architecture and separation of responsibilities.
- Testing and automated quality checks as they are introduced.

## Contributing

Shiori is currently a personal learning project, but suggestions and contributions are welcome.

Before opening a pull request:

1. Check the existing implementation and roadmap.
2. Keep changes focused and explain their purpose.
3. Avoid presenting planned functionality as implemented.
4. Include tests when practical.

For larger changes, opening an issue or discussion first is encouraged so the scope and approach can be aligned.

## License

Shiori is distributed under the [MIT License](LICENSE).

