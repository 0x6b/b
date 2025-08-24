# Beacon

A command-line task planning tool that helps you organize and track work through structured plans and actionable steps.

## Installation

```bash
$ git clone https://github.com/0x6b/b
$ cd b
$ cargo install --path crates/beacon-cli
```

## Quick Start with Claude Code

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
$ b plan show <ID>
# or use short aliases:
$ b p s <ID>
```

See `b help` for more commands and available aliases.

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
      "command": "/path/to/.cargo/bin/b",
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
