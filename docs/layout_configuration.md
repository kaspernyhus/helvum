# Layout Configuration

This feature allows you to customize node placement in Helvum using a configuration file with column-based layout and pattern matching.

## Configuration File Format

The configuration uses TOML format with three main sections:

### 1. Columns Section
Define columns with their X positions:
```toml
[layout]
columns = [
  { x = 50.0, name = "sources" },
  { x = 300.0, name = "filters" },
  { x = 550.0, name = "apps" },
  { x = 800.0, name = "sinks" }
]
```

### 2. Rules Section
Pattern matching rules (processed in order, first match wins):
```toml
[[layout.rules]]
pattern = "Firefox*"  # Simple glob patterns
column = "apps"
node_type = "filter"  # Optional: restrict to specific node types

[[layout.rules]]
pattern = "*Microphone*"
column = "sources"
node_type = "source"  # Optional: only match source nodes

[[layout.rules]]
pattern = "*sink*"  # Case-insensitive matching
column = "sinks"
```

**Supported Patterns:**
- `*` - matches any number of characters
- `?` - matches exactly one character
- Case-insensitive matching

**Node Type Constraints (optional):**
- `"source"` - matches NodeType::Output nodes only
- `"sink"` - matches NodeType::Input nodes only
- `"filter"` - matches nodes that are neither input nor output

### 3. Defaults Section
Default column positions when no pattern matches:
```toml
[layout.defaults]
sink = 800.0
source = 50.0
other = 300.0
```

## Usage

### Standard Locations
The application will automatically look for configuration files in this order:
1. `./layout.toml` (current working directory where helvum is launched)
2. `~/.config/helvum/layout.toml` (user config directory)


## Example Configuration

See `example_layout.toml` for a complete example with common audio application patterns.

## Validation Requirements

- Layout must contain at least one column
- Layout must contain at least one rule
- All rule column references must exist in the columns list
- Columns must be spaced at least `COLUMN_WIDTH` pixels apart to prevent overlap
- Invalid configurations will log warnings and fall back to original behavior
