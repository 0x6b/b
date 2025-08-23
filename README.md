# Beacon

A command-line task planning tool with a clean Display-based architecture for consistent formatting across CLI and MCP interfaces.

## Architecture

Beacon follows a layered architecture that clearly separates business logic from interface concerns:

```text
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Interfaces    │    │   Core Logic    │    │   Data Layer    │
│  (CLI + MCP)    │───▶│   (Handlers)    │───▶│   (Database)    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
     Thin Wrappers        Business Logic           SQLite Storage
```

### Core Separation of Concerns

- **beacon-core**: Contains all business logic, data models, and operations
  - **Handlers**: High-level business workflows that coordinate operations
  - **Models**: Domain objects with Display implementations for formatting
  - **Operations**: Reusable business logic components
  - **Planner**: Database abstraction and low-level data operations
  - **Display**: Formatting functions and result wrappers for consistent output

- **beacon-cli**: Thin wrapper providing command-line interface
  - Argument parsing with clap
  - Terminal rendering with rich markdown output
  - Direct calls to core handlers

- **MCP Server**: Thin wrapper providing AI model integration
  - JSON-RPC protocol handling
  - Parameter validation and conversion
  - Direct calls to core handlers

This architecture ensures:
- **Consistency**: Both interfaces use identical business logic
- **Testability**: Core logic can be tested independently
- **Maintainability**: Single source of truth for all operations
- **Flexibility**: Easy to add new interfaces without duplicating logic

## Installation

```bash
$ git clone https://github.com/0x6b/beacon-rs
$ cd beacon-rs
$ cargo install --path crates/beacon-cli
```

## Quick Start with Cloud Code

Use an embedded prompt template to create a plan and execute it:

```console
$ claude
# create a new plan
/beacon:plan "Add new feature"
# then do it
/beacon:do
```

Occasionally, you may want to see the status of the current plan while Claude is working on it:

```console
$ beacon plan show <ID>
```

See `beacon help` for more commands.

## Configuration

### Database

By default, Beacon stores data in `$XDG_DATA_HOME/beacon/beacon.db`, usually `~/.local/share/beacon/beacon.db`. You can override this behavior with the `--database-file <path>` option.

### MCP

Beacon includes a Model Context Protocol (MCP) server that provides AI assistants with structured access to the task planning system.

```json
{
  "mcpServers": {
    "beacon": {
      "type": "stdio",
      "command": "/path/to/.cargo/bin/beacon",
      "args": [
        "serve"
      ],
      "env": {}
    }
  }
}
```

## License

MIT. See [LICENSE](LICENSE) for details.
