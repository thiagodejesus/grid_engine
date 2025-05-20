# Grid Engine

A flexible and efficient Rust library for managing 2D grid-based layouts with automatic collision handling and dynamic vertical expansion.

## Features

- 🎯 Automatic collision detection and resolution
- 📏 Dynamic grid expansion on the y-axis
- 🔄 Event system for tracking grid changes
- 🛡️ Strong type safety and error handling
- 📦 No unsafe code

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
grid_engine = "0.1.0"
```

## Usage

Here's a basic example of using grid_engine:

```rust
use grid_engine::grid_engine::GridEngine;

// Create a new 10x12 grid
let mut grid = GridEngine::new(10, 12);

// Add a change listener to track modifications
grid.events.add_changes_listener(Box::new(|event| {
    println!("Grid changed: {:?}", event.changes);
}));

// Add items to the grid (with automatic collision handling)
grid.add_item("box1".to_string(), 0, 0, 2, 2).unwrap();
grid.add_item("box2".to_string(), 0, 0, 2, 2).unwrap(); // Will be repositioned to avoid collision

// Move items
grid.move_item("box1", 2, 2).unwrap();

// Remove items
grid.remove_item("box2").unwrap();
```

Check out the [examples](examples/) directory for more usage examples.

## API Overview

The main components of the library are:

- `GridEngine`: The main engine for managing the grid system
- `Node`: Represents an item in the grid with position and dimensions
- `GridEvents`: Event system for tracking changes
- Error types for robust error handling

For detailed API documentation, run:
```bash
cargo doc --open
```

## Development

### Running Tests

```bash
cargo test
```

### Running Examples

```bash
cargo run --example managing_grid
```

## Planned Features

- [ ] Serde serialization support (optional feature)
- [ ] WebAssembly support (optional feature)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.

## License

This project is licensed under either:

- MIT license

at your option.

## Acknowledgments

- Built with the [grid](https://crates.io/crates/grid) crate for efficient grid operations