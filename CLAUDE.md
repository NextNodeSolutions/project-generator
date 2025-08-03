# CLAUDE.md - Project Generator

This file provides guidance to Claude Code when working specifically with the Project Generator CLI tool.

## MCP Context7 Configuration

**IMPORTANT**: Always use MCP context7 server for documentation lookup in PRIORITY for the following technologies:

### Priority Documentation Sources (MCP Context7)
- **Rust**: Use MCP context7 docs for Rust language features, ownership, lifetimes, error handling
- **Cargo**: Use MCP context7 docs for Cargo commands, Cargo.toml configuration, dependencies
- **Clap**: Use MCP context7 docs for CLI argument parsing, derive macros, subcommands
- **Serde**: Use MCP context7 docs for serialization/deserialization, JSON/YAML handling
- **Tokio**: Use MCP context7 docs for async runtime, async/await patterns
- **Reqwest**: Use MCP context7 docs for HTTP client, JSON requests, error handling
- **Git2**: Use MCP context7 docs for Git operations, repository management
- **Regex**: Use MCP context7 docs for pattern matching, string processing

When a user asks about any of these technologies, **ALWAYS query MCP context7 FIRST** before using general knowledge.

## Project Overview

**Project Generator** is a CLI tool built in Rust for generating projects from templates with GitHub integration support.

### Technology Stack
- **Language**: Rust (2021 edition)
- **CLI Framework**: Clap v4 with derive macros
- **Serialization**: Serde with JSON/YAML support
- **HTTP Client**: Reqwest for GitHub API integration
- **Async Runtime**: Tokio for async operations
- **Git Operations**: Git2 for repository management
- **User Interaction**: Inquire for interactive prompts

### Development Commands

```bash
# Build
cargo build --release

# Run (Interactive Mode)
cargo run                                    # Interactive prompt mode
cargo run -- --config config.yaml           # With config file
cargo run -- --remote --config config.yaml  # GitHub mode (requires token)
cargo run -- --debug                        # Debug mode

# Testing
cargo test                                   # Run all tests
cargo test --lib                            # Run library tests only

# Code Quality
cargo fmt                                    # Format code
cargo clippy                                # Run linter
cargo check                                 # Type checking
```

### Architecture

**Module Structure:**
- `src/cli/`: CLI interface and command handling
- `src/config/`: Configuration file parsing (YAML/JSON)
- `src/generate/`: Core project generation logic
- `src/github/`: GitHub API integration and repository operations
- `src/template/`: Template processing and variable replacement
- `src/utils/`: Utility functions (context, validation, strings, errors)

**Key Files:**
- `src/main.rs`: Entry point and CLI setup
- `src/args.rs`: Command-line argument definitions
- `src/lib.rs`: Library interface
- `Cargo.toml`: Dependencies and project configuration

### Functionality

**Core Features:**
1. **Local Mode**: Generate projects from local templates
2. **Remote Mode**: Generate projects from GitHub repositories
3. **Template Processing**: Variable replacement in files and directories
4. **Configuration**: YAML/JSON config file support
5. **Interactive Mode**: User prompts for project configuration

**GitHub Integration:**
- Requires `GITHUB_TOKEN` environment variable
- Supports private and public repositories
- Handles repository cloning and template extraction

### Environment Variables
- `GITHUB_TOKEN`: GitHub personal access token for remote mode
- `DEBUG`: Enable debug output (any value enables)

### Configuration Files
- **Format**: YAML or JSON
- **Location**: Specified via `--config` flag
- **Structure**: Template variables and generation settings

### Dependencies
- **clap**: CLI argument parsing with derive macros
- **inquire**: Interactive user prompts
- **serde**: Serialization framework (JSON/YAML)
- **serde_json**: JSON support for Serde
- **serde_yaml**: YAML support for Serde
- **regex**: Pattern matching for template processing
- **indexmap**: Ordered maps for configuration
- **git2**: Git repository operations
- **reqwest**: HTTP client for GitHub API
- **tokio**: Async runtime for concurrent operations

### Error Handling
- Custom error types in `src/utils/error.rs`
- Comprehensive error propagation using `?` operator
- User-friendly error messages for CLI

### Testing Strategy
- Unit tests for core logic
- Integration tests for template generation
- Test configurations in `test-config.yaml`

### Code Style
- Standard Rust formatting with `cargo fmt`
- Clippy linting for additional checks
- Clear module organization with public APIs