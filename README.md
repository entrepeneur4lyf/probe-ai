# Probe

This Rust tool combines ripgrep's searching capabilities with tree-sitter's code parsing to find and extract complete code blocks (functions, classes, structs, etc.) based on search patterns. Probe can be used as both a CLI tool and an MCP server.

## Features

- Uses ripgrep as a library for fast code searches
- Leverages tree-sitter to parse code and extract complete code blocks
- Ensures full functions/structs are extracted, not just matching lines
- Supports multiple programming languages
- Can run as a CLI tool or as an MCP server

## Installation

### Using Pre-built Binaries

You can download pre-built binaries for your platform from the [GitHub Releases](https://github.com/yourusername/probe/releases) page.

1. Download the appropriate binary for your platform:
   - `probe-x86_64-linux.tar.gz` for Linux (x86_64)
   - `probe-x86_64-darwin.tar.gz` for macOS (Intel)
   - `probe-aarch64-darwin.tar.gz` for macOS (Apple Silicon)
   - `probe-x86_64-windows.zip` for Windows

2. Extract the archive:
   ```bash
   # For Linux/macOS
   tar -xzf probe-*-*.tar.gz
   
   # For Windows
   unzip probe-x86_64-windows.zip
   ```

3. Move the binary to a location in your PATH:
   ```bash
   # For Linux/macOS
   sudo mv probe /usr/local/bin/
   
   # For Windows
   # Move probe.exe to a directory in your PATH
   ```

### Building from Source

1. Install Rust and Cargo (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Clone this repository:
   ```bash
   git clone https://github.com/yourusername/probe.git
   cd probe
   ```

3. Build the project:
   ```bash
   cargo build --release
   ```

4. (Optional) Install globally:
   ```bash
   cargo install --path .
   ```

## Releasing New Versions

This project uses GitHub Actions to automatically build and release binaries for multiple platforms.

### Creating a New Release

1. Update the version in `Cargo.toml`
2. Commit your changes
3. Create and push a new tag:
   ```bash
   git tag -a v0.1.0 -m "Release v0.1.0"
   git push origin v0.1.0
   ```
4. GitHub Actions will automatically:
   - Build binaries for Linux, macOS (Intel and Apple Silicon), and Windows
   - Create a GitHub release
   - Upload the binaries as release assets
   - Generate checksums for verification

### Release Artifacts

Each release includes:
- Linux binary (x86_64)
- macOS binaries (x86_64 and aarch64)
- Windows binary (x86_64)
- SHA256 checksums for each binary

## Project Structure

The project is organized into the following directories:

- `src/` - Source code for the application
  - `language/` - Language-specific parsing modules
  - `search/` - Search implementation modules
- `tests/` - Test files and utilities
  - `mocks/` - Mock data files for testing
- `target/` - Build artifacts (generated by Cargo)
- `.github/workflows/` - GitHub Actions workflow configurations for CI/CD

## Usage

### CLI Mode

```bash
probe <SEARCH_PATTERN> [OPTIONS]
```

#### Options

- `<SEARCH_PATTERN>` - Pattern to search for (required, positional argument)
- `--paths` - Directory paths to search in (defaults to current directory)
- `--files-only` - Skip AST parsing and just output unique files
- `--ignore` - Custom patterns to ignore (in addition to .gitignore and common patterns)
- `--include-filenames`, `-n` - Include files whose names match query words
- `--reranker`, `-r` - Reranking method to use for search results (hybrid, hybrid2, bm25, tfidf)
  - `hybrid` (default) - Simple combination of TF-IDF and BM25 scores
  - `hybrid2` - Comprehensive multi-metric ranking that considers term matches, code structure, and more
  - `bm25` - BM25 ranking algorithm (better for natural language queries)
  - `tfidf` - TF-IDF ranking algorithm (better for code-specific terms)
- `--frequency`, `-s` - Use frequency-based search with stemming and stopword removal (enabled by default)
- `--exact` - Use exact matching without stemming or stopword removal (overrides frequency search)
- `--max-results` - Maximum number of results to return
- `--max-bytes` - Maximum total bytes of code content to return
- `--max-tokens` - Maximum total tokens in code content to return (for AI usage)
- `--allow-tests` - Allow test files and test code blocks in search results (disabled by default)
- `--any-term` - Match files that contain any of the search terms (by default, files must contain all terms)
- `--merge-blocks` - Merge adjacent code blocks after ranking (disabled by default)
- `--merge-threshold` - Maximum number of lines between code blocks to consider them adjacent for merging (default: 5)

#### Examples

```bash
# Search for "setTools" in the current directory (uses frequency search by default)
probe setTools

# Search for "impl" in the src directory with exact matching
probe impl --paths ./src --exact

# Search for "search" with a maximum of 5 results
probe search --max-results 5

# Search for "function" and merge adjacent code blocks
probe function --merge-blocks --merge-threshold 3
```

### MCP Server Mode

To run Probe as an MCP server:

```bash
cd mcp && npm run build && node build/index.js
```

This starts the tool as an MCP server that can be used with the Model Context Protocol. The server exposes a `search_code` tool that can be used to search for code patterns and extract complete code blocks.

#### MCP Tool: search_code

This tool should be used every time you need to search the codebase for understanding code structure, finding implementations, or identifying patterns. Queries can be any text (including multi-word phrases like "IP whitelist"), but prefer simple, focused queries for better results. Use maxResults parameter to limit the number of results when needed.

Input schema:
```json
{
  "path": "Directory path to search in",
  "query": ["Query patterns to search for"],
  "filesOnly": false,
  "ignore": ["Patterns to ignore"],
  "includeFilenames": false,
  "reranker": "hybrid",
  "frequencySearch": true,
  "exact": false,
  "maxResults": null,
  "maxBytes": null,
  "maxTokens": null,
  "allowTests": false,
  "mergeBlocks": false,
  "mergeThreshold": 5
}
```

Note: `frequencySearch` is enabled by default. If you want exact matching without stemming or stopword removal, set `exact` to `true` which will override the frequency search behavior.

The `reranker` parameter can be set to one of the following values:
- `hybrid` (default) - Simple combination of TF-IDF and BM25 scores
- `hybrid2` - Comprehensive multi-metric ranking that considers term matches, code structure, and more
- `bm25` - BM25 ranking algorithm (better for natural language queries)
- `tfidf` - TF-IDF ranking algorithm (better for code-specific terms)

Example usage with MCP client:
```rust
use std::sync::Arc;
use mcp_rust_sdk::{Client, transport::stdio::StdioTransport};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a stdio transport to connect to the server
    let (transport, _) = StdioTransport::new();
    
    // Create and connect the client
    let client = Client::new(transport);
    
    // Call the search_code tool
    let response = client.request(
        "call_tool",
        Some(json!({
            "name": "search_code",
            "arguments": {
                "path": "./src",
                "query": ["impl", "fn"],
                "filesOnly": false,
                "exact": true  // Use exact matching instead of frequency-based search
            }
        }))
    ).await?;
    
    println!("Search results: {:?}", response);
    
    Ok(())
}
```

The server implements the ServerHandler trait from the MCP Rust SDK, providing:

1. `initialize` - Handles client connection and capabilities negotiation
2. `handle_method` - Processes method calls like list_tools, call_tool, etc.
3. `shutdown` - Handles graceful server shutdown

The server supports the following MCP methods:
- `list_tools` - Returns information about the available tools
- `call_tool` - Executes the search_code tool with the provided parameters
- `list_resources` - Returns an empty list (no resources implemented)
- `list_resource_templates` - Returns an empty list (no resource templates implemented)

## Supported Languages

Currently, the tool supports:
- Rust (.rs)
- JavaScript (.js, .jsx)
- TypeScript (.ts, .tsx)
- Python (.py)
- Go (.go)
- C (.c, .h)
- C++ (.cpp, .cc, .cxx, .hpp, .hxx)
- Java (.java)
- Ruby (.rb)
- PHP (.php)
- Markdown (.md, .markdown)

## How It Works

1. The tool scans files in the specified directory using ripgrep
2. For each match, it parses the file with tree-sitter to build an AST
3. It then finds the smallest AST node that:
   - Contains the matching line
   - Represents a complete code block (function, class, struct, etc.)
4. Finally, it extracts and displays these code blocks with file information

## Adding Support for New Languages

To add support for a new programming language:

1. Add the tree-sitter grammar as a dependency in `Cargo.toml`:
   ```toml
   [dependencies]
   tree-sitter-go = "0.20"  # Example for Go
   ```

2. Create a new file in the `src/language` directory for the language
3. Implement the `Language` trait for the new language
4. Update the language factory to support the new file extension

## Troubleshooting

- **No matches found**: Verify your search pattern and check if there are matches using the regular ripgrep tool
- **File parsing errors**: Some files may have syntax errors or use language features not supported by the tree-sitter grammar
- **Missing code blocks**: Update the language implementation to support additional node types for your language
