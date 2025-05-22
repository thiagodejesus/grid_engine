// Copyright (c) 2025 Thiago Ramos
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

//! Grid Engine is a flexible and efficient library for managing 2D grid-based layouts.
//!
//! This crate provides functionality for:
//! - Managing items in a 2D grid with automatic collision handling
//! - Dynamic vertical expansion of the grid
//! - Event system for tracking grid changes
//! - Strong type safety and error handling
//!
//! # Example
//!
//! ```rust
//! use grid_engine::grid_engine::GridEngine;
//! # use std::error::Error;
//!
//! # fn main() -> Result<(), Box<dyn Error>> {
//! // Create a new 10x12 grid
//! let mut grid = GridEngine::new(10, 12);
//!
//! // Add items (automatic collision handling)
//! grid.add_item("box1".to_string(), 0, 0, 2, 2)?;
//!
//! // Move items
//! grid.move_item("box1", 2, 2)?;
//!
//! # Ok(())
//! # }
//! ```
//!
//! See the `examples` directory for more usage examples.

mod error;
pub mod grid_engine;
mod grid_events;
mod inner_grid;
pub mod node;
mod utils;
