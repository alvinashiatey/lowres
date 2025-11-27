# lowres

A minimal, high-performance image resizer and compressor built with Rust and Tauri.

## Features

- **Drag & Drop**: Simply drop your images to process them.
- **Fast**: Powered by Rust for multi-threaded image processing.
- **Minimal UI**: Clean, Swiss-style interface.
- **Privacy Focused**: All processing happens locally on your machine.

## Installation

### macOS (Homebrew)

You can install `lowres` via Homebrew:

```bash
brew install --cask alvinashiatey/tap/lowres
```

### Manual Download

Download the latest version from the [Releases](https://github.com/alvinashiatey/lowres/releases) page.

## Development

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/) & [pnpm](https://pnpm.io/)

### Setup

1. Clone the repository:

   ```bash
   git clone https://github.com/alvinashiatey/lowres.git
   cd lowres
   ```

2. Install dependencies:

   ```bash
   pnpm install
   ```

3. Run in development mode:
   ```bash
   pnpm tauri dev
   ```

## License

MIT
