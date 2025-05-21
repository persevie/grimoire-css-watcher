# Grimoire CSS Watcher

[![Crates.io](https://img.shields.io/crates/v/grimoire-css-watcher.svg)](https://crates.io/crates/grimoire-css-watcher)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance file watcher for [Grimoire CSS](https://github.com/persevie/grimoire-css) projects. It monitors your Grimoire CSS configuration and automatically rebuilds your CSS whenever changes are detected.

## Features

- Efficient, non-blocking file watching.
- Automatic detection of files from `grimoire.config.json`.
- Smart debouncing to prevent excessive rebuilds.
- Clean, colorful logging output.
- Lightweight and focused.

## Installation

### From Crates.io

```bash
cargo install grimoire-css-watcher
```

### From GitHub Releases

Pre-compiled binaries for Linux, macOS (x86_64, arm64), and Windows (x86_64) are available on the [GitHub Releases page](https://github.com/persevie/grimoire-css-watcher/releases). Download the appropriate binary for your system, make it executable, and place it in your PATH.

## Requirements

- Grimoire CSS (`grimoire_css`) must be installed and available in your PATH.
- A valid `grimoire.config.json` file in your project's `grimoire/config` directory (or the root of the project if you modify the watcher's expected path).

## Usage

```bash
# Run the watcher in the current directory
grimoire-css-watcher

# Specify a different project directory
grimoire-css-watcher --path /path/to/your/project

# Customize the debounce duration (default: 300ms)
grimoire-css-watcher --debounce 500

# Enable verbose logging
grimoire-css-watcher --verbose
```

## Command Line Options

- `-p, --path <PATH>`: Path to the project directory (defaults to current directory).
- `-d, --debounce <MILLISECONDS>`: Debounce duration in milliseconds (default: 300).
- `-v, --verbose`: Enable verbose output.
- `-h, --help`: Show help information.
- `--version`: Show version information.

## How It Works

1. Reads `grimoire/config/grimoire.config.json` (by default).
2. Extracts input file patterns.
3. Watches resolved files and the config file itself.
4. On change, runs `grimoire_css build`.
5. Stops with Ctrl+C.

## License

MIT
