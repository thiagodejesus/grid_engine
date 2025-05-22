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

//! Node representation and management for grid items.
//!
//! This module provides the [`Node`] type which represents an item in the grid.
//! Each node has a position (x, y), dimensions (width, height), and a unique identifier.
//! Nodes are managed by the grid engine and can be added, moved, or removed from the grid.

use crate::{
    error::InnerGridError,
    inner_grid::{InnerGrid, UpdateGridOperation},
    utils::{ForCellArgs, for_cell},
};

/// Represents an item in the grid with position and dimensions.
///
/// A node occupies a rectangular area in the grid defined by:
/// - Its top-left corner position (x, y)
/// - Its dimensions (width, height)
/// - A unique identifier
///
/// The node's area can be iterated over using the `for_cell` method,
/// which visits each cell in the node's occupied space.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Node {
    /// Unique identifier for the node
    pub(crate) id: String,
    /// X coordinate of the top-left corner
    pub(crate) x: usize,
    /// Y coordinate of the top-left corner
    pub(crate) y: usize,
    /// Width of the node in grid cells
    pub(crate) w: usize,
    /// Height of the node in grid cells
    pub(crate) h: usize,
}

impl Node {
    /// Creates a new Node with the specified parameters.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the node
    /// * `x` - X coordinate of the top-left corner
    /// * `y` - Y coordinate of the top-left corner
    /// * `w` - Width in grid cells
    /// * `h` - Height in grid cells
    pub(crate) fn new(id: impl Into<String>, x: usize, y: usize, w: usize, h: usize) -> Node {
        Node {
            id: id.into(),
            x,
            y,
            w,
            h,
        }
    }

    /// Iterates over all cells occupied by this node.
    ///
    /// This method provides a way to perform operations on each cell
    /// that the node occupies in the grid. The callback is called with
    /// the coordinates of each cell.
    ///
    /// # Arguments
    ///
    /// * `callback` - Function to execute for each cell
    ///
    /// # Returns
    ///
    /// * `Ok(())` if all cells were processed successfully
    /// * `Err(InnerGridError)` if the callback returns an error
    pub(crate) fn for_cell(
        &self,
        callback: &mut impl FnMut(usize, usize) -> Result<(), InnerGridError>,
    ) -> Result<(), InnerGridError> {
        for_cell(
            ForCellArgs {
                x: self.x,
                y: self.y,
                w: self.w,
                h: self.h,
            },
            callback,
        )
    }

    /// Updates the grid state for this node.
    ///
    /// Used internally to modify the grid when a node is added, moved,
    /// or removed. The operation type determines how the grid is updated.
    ///
    /// # Arguments
    ///
    /// * `grid` - The grid to update
    /// * `update_operation` - The type of update to perform (Add/Remove)
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the update was successful
    /// * `Err(InnerGridError)` if the update fails (e.g., out of bounds)
    pub(crate) fn update_grid(
        &self,
        grid: &mut InnerGrid,
        update_operation: UpdateGridOperation,
    ) -> Result<(), InnerGridError> {
        self.for_cell(&mut |x, y| grid.update(self, x, y, update_operation))?;

        Ok(())
    }

    /// Returns the unique identifier of the node.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the x coordinate of the node.
    pub fn x(&self) -> &usize {
        &self.x
    }

    /// Returns the y coordinate of the node.
    pub fn y(&self) -> &usize {
        &self.y
    }

    /// Returns the width of the node.
    pub fn w(&self) -> &usize {
        &self.w
    }

    /// Returns the height of the node.
    pub fn h(&self) -> &usize {
        &self.h
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = Node::new("test_node".to_string(), 1, 2, 3, 4);

        assert_eq!(node.id, "test_node");
        assert_eq!(node.x, 1);
        assert_eq!(node.y, 2);
        assert_eq!(node.w, 3);
        assert_eq!(node.h, 4);
    }

    #[test]
    fn test_get_id() {
        let node = Node::new("test_node".to_string(), 0, 0, 1, 1);

        assert_eq!(node.id.clone(), "test_node");
        // Test that get_id returns a clone
        let id1 = node.id.clone();
        let id2 = node.id.clone();
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_for_cell() {
        let node = Node::new("test_node".to_string(), 1, 2, 2, 2);

        let mut visited = vec![];
        node.for_cell(&mut |x, y| {
            visited.push((x, y));
            Ok(())
        })
        .unwrap();

        assert_eq!(
            visited,
            vec![(1, 2), (1, 3), (2, 2), (2, 3)],
            "Should visit all cells in the node's area"
        );
    }

    #[test]
    fn test_for_cell_error_propagation() {
        let node = Node::new("test_node".to_string(), 0, 0, 2, 2);

        let result = node.for_cell(&mut |x, _y| {
            if x > 0 {
                Err(crate::error::InnerGridError::OutOfBoundsAccess { x: 0, y: 0 })
            } else {
                Ok(())
            }
        });

        assert!(result.is_err());
    }

    #[test]
    fn test_for_cell_zero_dimensions() {
        let node = Node::new("test_node".to_string(), 0, 0, 0, 0);

        let mut visited = vec![];
        node.for_cell(&mut |x, y| {
            visited.push((x, y));
            Ok(())
        })
        .unwrap();

        assert!(
            visited.is_empty(),
            "Should not visit any cells for zero dimensions"
        );
    }
}
