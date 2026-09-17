# Dragon Vault

[English](README.md) | [中文](README-ZH.md)

A secure desktop application for managing LLM API keys. Built with [Tauri](https://tauri.app/), [React](https://react.dev/), and [Rust](https://www.rust-lang.org/).

## Features

- 🔐 **AES-256-GCM Encryption** — All API keys are encrypted at rest, zero plaintext storage
- 🔑 **Master Password Protection** — Argon2id key derivation from your master password
- 🏢 **Multi-Provider Support** — Manage keys for OpenAI, Anthropic, Google, and more
- 🏷️ **Tags & Grouping** — Organize keys by project, team, or purpose
- 📋 **One-Click Copy** — Copy to clipboard with auto-clear timer
- 💾 **Encrypted Backup** — Export and import encrypted vault backups
- 🖥️ **Cross-Platform** — Runs on Windows, macOS, and Linux

## Screenshots

> _Coming soon_

## Installation

### From Source

#### Prerequisites

- [Node.js](https://nodejs.org/) >= 20
- [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/tools/install)
- Tauri system dependencies (see [Tauri Prerequisites](https://tauri.app/start/prerequisites/))

#### Build

```bash
# Clone the repository
git clone https://github.com/your-org/dragon-vault.git
cd dragon-vault

# Install frontend dependencies
pnpm install

# Start development mode
pnpm tauri dev

# Build for production
pnpm tauri build
```

## Documentation

- [Architecture](docs/architecture.md)
- [Rust Guidelines](docs/rust-guidelines.md)
- [TypeScript Guidelines](docs/typescript-guidelines.md)

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop Framework | [Tauri 2](https://tauri.app/) |
| Frontend | React 19 + TypeScript 6 |
| Backend | Rust (edition 2021) |
| Build Tool | Vite 8 |
| Encryption | AES-256-GCM + Argon2id |

## Project Structure

```
dragon-vault/
├── src/                  # Frontend (React + TypeScript)
│   ├── components/       # UI components
│   ├── pages/            # Page components
│   ├── hooks/            # Custom React hooks
│   ├── services/         # Tauri IPC wrapper
│   └── stores/           # State management
├── src-tauri/            # Backend (Rust)
│   └── src/
│       ├── commands/     # Tauri command handlers
│       ├── models/       # Data models
│       ├── services/     # Business logic (crypto, key management)
│       └── storage/      # Persistence layer
└── docs/                 # Documentation
```

## Contributing

Contributions are welcome! Please read the [architecture docs](docs/architecture.md) and coding guidelines before submitting a PR.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request


<p align="center">
  Made with ❤️ for the AI developer community
</p>
