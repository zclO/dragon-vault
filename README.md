# Dragon Vault

[English](README.md) | [中文](README-ZH.md)

A secure vault for managing LLM API keys, on both desktop and mobile. Built with [Tauri](https://tauri.app/), [React](https://react.dev/), and [Rust](https://www.rust-lang.org/).

## Features

- 🔐 **AES-256-GCM Encryption** — All API keys are encrypted at rest, zero plaintext storage
- 🔑 **Master Password Protection** — Argon2id key derivation, verified via constant-time authenticated decryption
- 🔔 **Biometric Unlock** — Fingerprint unlock on Android with the derived key sealed in the hardware Keystore
- 📱 **Desktop & Mobile** — Windows, macOS, Linux, and Android (bottom-tab mobile UI)
- 🏢 **Multi-Provider Support** — OpenAI, Anthropic, Google, DeepSeek and more, plus custom OpenAI-compatible providers
- 📊 **Quota & Usage Queries** — Pull balance and token usage from supported vendors, with snapshot trends
- 🔌 **Connectivity Test** — Verify a key against its provider with one click
- 🏷️ **Tags & Grouping** — Organize keys by project, team, or purpose
- 📋 **One-Click Copy** — Copy to clipboard with auto-clear timer
- ⏱️ **Auto-Lock** — Vault relocks after an idle timeout, requiring the master password again
- 💾 **Encrypted Backup** — Export and import encrypted vault backups

## Screenshots

> _Coming soon_

## Installation

### From Source

#### Prerequisites

- [Node.js](https://nodejs.org/) >= 20
- [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/tools/install)
- Tauri system dependencies (see [Tauri Prerequisites](https://tauri.app/start/prerequisites/))
- For Android builds: [JDK](https://adoptium.net/) 17+, Android SDK & NDK (see [Tauri Android Guide](https://tauri.app/mobile/android/))

#### Desktop

```bash
# Clone the repository
git clone https://github.com/zclO/dragon-vault.git
cd dragon-vault

# Install frontend dependencies
pnpm install

# Start development mode
pnpm tauri dev

# Build for production
pnpm tauri build
```

#### Android

```bash
# Debug on a device/emulator (hot reload)
adb reverse tcp:1420 tcp:1420   # required for a physical device
pnpm android:dev

# Build a signed release APK
# Output: src-tauri/gen/android/app/build/outputs/apk/universal/release/
pnpm android:build
```

## Documentation

- [Architecture](docs/architecture.md)
- [Rust Guidelines](docs/rust-guidelines.md)
- [TypeScript Guidelines](docs/typescript-guidelines.md)

## Tech Stack

| Layer | Technology |
|-------|-----------|
| App Framework | [Tauri 2](https://tauri.app/) (desktop + Android) |
| Frontend | React 19 + TypeScript 6 |
| Backend | Rust (edition 2021) |
| Build Tool | Vite 8 |
| UI | Tailwind CSS 4 + Radix UI + lucide-react |
| Encryption | AES-256-GCM + Argon2id |
| Biometrics | tauri-plugin-biometry (Android Keystore) |

## Project Structure

```
dragon-vault/
├── src/                  # Frontend (React + TypeScript)
│   ├── components/       # UI primitives + layout components
│   ├── pages/            # Dashboard, Keys, Providers, Usage, Settings
│   ├── hooks/            # Custom React hooks (vault state)
│   ├── services/         # Tauri IPC wrappers + types
│   └── lib/              # Shared utilities
├── src-tauri/            # Backend (Rust)
│   └── src/
│       ├── commands/     # Tauri command handlers
│       ├── models/       # Data models
│       ├── services/     # Business logic (crypto, keys, providers, usage, vault)
│       └── storage/      # Persistence layer (secure + local storage)
└── docs/                 # Documentation
```

## Contributing

Contributions are welcome! Please read the [architecture docs](docs/architecture.md) and coding guidelines before submitting a PR.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

Distributed under the [AGPL-3.0](LICENSE) license.


<p align="center">
  Made with ❤️ for the AI developer community
</p>
