# HermesApp

Native iOS app for Hermes Desktop — connects to the existing Hermes API Server (port 8642, OpenAI-compatible).

## Overview

Personal iOS app for managing Hermes Desktop without Telegram or Multica. Connects directly to existing services:
- **API Server** (port 8642, OpenAI-compatible)
- **Dashboard** (port 9119)

## Features (MVP)

- Chat with Bot (Bot Mode)
- Session history
- Essential commands (`/new`, `/clear`, `/status`)
- Basic notifications

## Architecture

```
HermesApp/
├── Package.swift              # Swift Package
├── HermesApp/                 # Main app target
│   ├── HermesApp.swift        # Entry point, TabView
│   ├── Models/
│   │   └── Models.swift       # Data models (ChatMessage, Session, API responses)
│   ├── Services/
│   │   ├── HermesClient.swift # HTTP client for API Server
│   │   └── SessionManager.swift # Session management + local storage
│   ├── ViewModels/
│   │   └── ChatViewModel.swift # Chat UI state management
│   ├── Views/
│   │   ├── ChatView.swift     # Main chat interface
│   │   ├── ChatListView.swift # Session list
│   │   └── SettingsView.swift # App settings
│   ├── Constants/
│   │   └── AppConstants.swift # App-wide constants
│   ├── Extensions/
│   │   └── DateExtensions.swift # Date formatting
│   └── Assets/
│       └── AssetCatalogPlaceholder.swift
└── HermesAppTests/
    └── HermesClientTests.swift # Unit tests
```

## API Server Integration

The app connects to the Hermes API Server at `localhost:8642`:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Health check |
| `/v1/models` | GET | List available models |
| `/v1/chat/completions` | POST | Send chat messages (streaming supported) |
| `/api/sessions/list` | GET | List all sessions |
| `/api/sessions/new` | POST | Create new session |
| `/api/sessions/history` | GET | Get session history |
| `/api/sessions/clear` | POST | Clear session messages |
| `/api/sessions/delete` | DELETE | Delete session |
| `/api/commands` | GET | List available slash commands |
| `/api/status` | GET | Get server status |
| `/api/notifications/register` | POST | Register for push notifications |

Authentication: Bearer token via `API_SERVER_KEY` environment variable.

## Building

Requires:
- Swift 5.9+ (Swift 6.3.3 available)
- iOS 17+
- Xcode (for full development) or Swift CLI for basic builds

```bash
# Build with Swift CLI
cd HermesApp
swift build

# Run tests
swift test
```

## Configuration

Set the API key via environment variable:

```bash
export HERMES_API_KEY="your-api-key-here"
```

Or configure in Settings view within the app.

## Development Notes

- No Xcode available in this environment — building with Swift CLI only
- API Server requires authentication; the key must be configured
- Streaming chat uses SSE (Server-Sent Events)
- Local session storage via UserDefaults for offline support
- Swipe actions on session list (delete, clear)

## Next Steps

- [ ] Add Xcode project for full IDE support
- [ ] Implement push notifications (UserNotifications framework)
- [ ] Add deep linking support
- [ ] Implement haptic feedback
- [ ] Add widget extension for home screen
- [ ] Add App Intents for Siri integration
- [ ] Implement biometric authentication (Face ID / Touch ID)
- [ ] Add dark mode support (via SwiftUI automatic adaptation)

## License

Internal use only.
