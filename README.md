# Endpoint Logger

A privacy-first HTTP request logger built in Rust that monitors and logs all HTTP traffic through a reverse proxy architecture.

## What is Endpoint Logger?

Endpoint Logger is a development tool that acts as a transparent reverse proxy sitting between your clients and your application. It captures and logs all HTTP requests and responses in real-time, providing developers with complete visibility into their API traffic without modifying a single line of their application code.

### Key Principles

- **Privacy-First**: All data stays on your local machine. No cloud services, no external APIs, no data leaving your computer.
- **Universal Compatibility**: Works with any backend technology (Node.js, Python, Java, Go, PHP, etc.) because it operates at the HTTP layer.
- **Zero Configuration**: No accounts, API keys, or complex setup. Just point it at your application and start logging.
- **Transparent Monitoring**: Your application doesn't know it's being monitored. No SDK to integrate, no code changes required.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│              YOUR LOCAL MACHINE                          │
│                                                          │
│  Client (Browser/Postman/curl)                          │
│         ↓                                                │
│  ┌────────────────────────────────────────────────┐    │
│  │    ENDPOINT LOGGER (Rust/Axum)                 │    │
│  │    Port: 3000 (configurable)                   │    │
│  │    - Intercepts all HTTP traffic                │    │
│  │    - Logs requests & responses                  │    │
│  │    - Stores data locally (SQLite)              │    │
│  └────────────────────────────────────────────────┘    │
│         ↓                                                │
│  Your Application (localhost:80, 8080, etc.)            │
│                                                          │
│         ALL DATA STAYS ON THIS MACHINE                  │
└─────────────────────────────────────────────────────────┘
```

## Use Cases

- **API Development**: See exactly what requests your frontend is sending
- **Debugging**: Inspect headers, query parameters, and request/response bodies
- **Testing**: Verify your API behaves correctly under different scenarios
- **Learning**: Understand how HTTP works by seeing real traffic
- **Documentation**: Generate API documentation from actual usage patterns

## Technology Stack

- **Rust**: High-performance, memory-safe systems language
- **Axum**: Modern, ergonomic web framework built on Tokio
- **SQLite**: Embedded database for local storage (coming soon)
- **Tokio**: Async runtime for handling concurrent connections
- **Clap**: Command-line argument parsing with excellent UX

## Current Features

The application supports flexible configuration through multiple sources with the following priority chain:

**Priority**: CLI Arguments > Environment Variables > TOML File > Defaults


## Contributing

This project is currently in early development. Contributions, suggestions, and feedback are welcome!

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author

Ioan A. - Building tools for developers

---

**Note**: This is a development tool intended for local debugging and monitoring. Do not use in production environments or to intercept traffic from applications you don't own.
