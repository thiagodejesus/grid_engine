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

//! Internal grid implementation for managing the spatial layout of nodes.
//!
//! This module provides the core grid functionality, including:
//! - Grid cell management
//! - Dynamic row expansion
//! - Node placement and removal operations
//!
//! The grid automatically expands vertically when needed, allowing for
//! flexible layout management while maintaining horizontal constraints.

use crate::{error::InnerGridError, node::Node};
use grid::Grid;
use std::ops::{Deref, DerefMut};

/// Operation to perform when updating the grid.
#[derive(Debug, Clone, Copy)]
pub enum UpdateGridOperation {
    /// Add a node to the grid cells
    Add,
    /// Remove a node from the grid cells
    Remove,
}

/// Internal grid structure that manages the spatial layout of nodes.
///
/// The grid maintains a 2D layout of cells, where each cell can either be
/// empty (None) or contain a node ID (Some(String)). The grid can dynamically
/// expand vertically to accommodate new nodes.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct InnerGrid {
    /// Whether the grid can expand vertically (add rows)
    can_expand_y: bool,
    /// The underlying grid structure
    inner: Grid<Option<String>>,
}

/// Allows using InnerGrid with methods from the underlying Grid type.
///
/// This implementation enables transparent access to Grid methods without
/// explicitly accessing the inner field.
impl Deref for InnerGrid {
    type Target = Grid<Option<String>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// Allows mutable access to the underlying Grid methods.
///
/// This implementation enables modifying the grid using Grid methods
/// while maintaining InnerGrid's invariants.
impl DerefMut for InnerGrid {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl InnerGrid {
    /// Creates a new grid with the specified dimensions.
    ///
    /// The grid is initially empty (all cells are None) and can expand
    /// vertically by default.
    ///
    /// # Arguments
    ///
    /// * `rows` - Initial number of rows
    /// * `cols` - Number of columns (fixed)
    pub fn new(rows: usize, cols: usize) -> Self {
        let inner = Grid::new(rows, cols);
        InnerGrid {
            inner,
            can_expand_y: true,
        }
    }

    /// Handles automatic grid expansion when accessing cells.
    ///
    /// If the requested y-coordinate is beyond the current grid bounds
    /// and expansion is allowed, the grid will automatically add rows
    /// to accommodate the access.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate to check
    /// * `y` - Y coordinate to check
    fn handle_expansion(&mut self, x: usize, y: usize) {
        let rows = self.rows();
        let cols = self.cols();

        let can_expand = self.can_expand_y && x < cols;

        if can_expand && y >= rows {
            self.expand_rows(y - rows + 1);
        }
    }

    /// Gets a reference to the cell at the specified coordinates.
    ///
    /// If the coordinates are beyond the current grid bounds and expansion
    /// is allowed, the grid will automatically expand to accommodate the access.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate of the cell
    /// * `y` - Y coordinate of the cell
    ///
    /// # Returns
    ///
    /// * `Some(&Option<String>)` - Reference to the cell if coordinates are valid
    /// * `None` - If coordinates are invalid or beyond expansion limits
    pub fn get(&mut self, x: usize, y: usize) -> Option<&Option<String>> {
        if self.inner.get(y, x).is_none() {
            self.handle_expansion(x, y);
        }

        return self.inner.get(y, x);
    }

    pub(crate) fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut Option<String>> {
        if self.inner.get(y, x).is_none() {
            self.handle_expansion(x, y);
        }

        return self.inner.get_mut(y, x);
    }

    /// Updates a cell in the grid based on the specified operation.
    ///
    /// Adds or removes a node's ID from the specified cell. When removing,
    /// it only clears the cell if it contains the specified node's ID.
    ///
    /// # Arguments
    ///
    /// * `node` - The node being added or removed
    /// * `x` - X coordinate of the cell to update
    /// * `y` - Y coordinate of the cell to update
    /// * `operation` - Whether to add or remove the node
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the update was successful
    /// * `Err(InnerGridError)` - If the coordinates are invalid
    pub(crate) fn update(
        &mut self,
        node: &Node,
        x: usize,
        y: usize,
        operation: UpdateGridOperation,
    ) -> Result<(), InnerGridError> {
        let cell = self
            .get_mut(x, y)
            .ok_or(InnerGridError::OutOfBoundsAccess { x, y })?;

        match operation {
            UpdateGridOperation::Add => {
                *cell = Some(node.id().to_string());
            }
            UpdateGridOperation::Remove => {
                if cell.as_ref() == Some(&node.id().to_string()) {
                    *cell = None;
                }
            }
        }
        Ok(())
    }

    /// Expands the grid by adding the specified number of rows.
    ///
    /// New rows are initialized with empty cells (None). This is used
    /// internally when automatic expansion is triggered.
    ///
    /// # Arguments
    ///
    /// * `rows` - Number of rows to add
    pub(crate) fn expand_rows(&mut self, rows: usize) {
        let cols = self.cols();

        for _ in 0..rows {
            let row = vec![None; cols];
            self.push_row(row);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::error::InnerGridError;
    use crate::inner_grid::{InnerGrid, UpdateGridOperation};
    use crate::node::Node;

    #[test]
    fn test_update_grid_add_node() {
        let mut grid = InnerGrid::new(3, 3);

        let node = Node {
            id: String::from("test_node"),
            w: 1,
            h: 1,
            x: 1,
            y: 1,
        };

        grid.update(&node, 1, 1, UpdateGridOperation::Add).unwrap();
        assert_eq!(grid.get(1, 1), Some(&Some("test_node".to_string())));
    }

    #[test]
    fn test_update_grid_remove_node() {
        let mut grid = InnerGrid::new(3, 3);

        let node = Node {
            id: String::from("test_node"),
            w: 1,
            h: 1,
            x: 1,
            y: 1,
        };

        // First add the node
        grid.get_mut(1, 1)
            .map(|cell| *cell = Some("test_node".to_string()));

        // Then remove it
        grid.update(&node, 1, 1, UpdateGridOperation::Remove)
            .unwrap();
        assert_eq!(grid.get(1, 1), Some(&None));
    }

    #[test]
    fn test_update_grid_remove_different_node() {
        let mut grid = InnerGrid::new(3, 3);

        let node = Node {
            id: String::from("test_node"),
            w: 1,
            h: 1,
            x: 1,
            y: 1,
        };

        // Add a different node's ID
        grid.get_mut(1, 1)
            .map(|cell| *cell = Some("different_node".to_string()));

        // Try to remove our node
        grid.update(&node, 1, 1, UpdateGridOperation::Remove)
            .unwrap();

        // The different node should still be there
        assert_eq!(grid.get(1, 1), Some(&Some("different_node".to_string())));
    }

    #[test]
    fn test_update_grid_out_of_bounds() {
        let mut grid = InnerGrid::new(3, 3);
        let node = Node {
            id: String::from("test_node"),
            w: 1,
            h: 1,
            x: 0,
            y: 0,
        };

        let result = grid.update(&node, 3, 3, UpdateGridOperation::Add);
        assert!(matches!(
            result,
            Err(InnerGridError::OutOfBoundsAccess { x: 3, y: 3 })
        ));
    }

    #[test]
    fn test_grid_expands_when_can_expand_y_is_true() {
        let mut grid = InnerGrid::new(3, 3);
        let node = Node {
            id: String::from("test_node"),
            w: 1,
            h: 1,
            x: 1,
            y: 4,
        };

        // Try to add node at y=4 (beyond current grid size) with can_expand_y=true
        let result = grid.update(&node, 1, 4, UpdateGridOperation::Add);
        assert!(result.is_ok());

        // Verify grid has expanded and node was added
        assert_eq!(grid.rows(), 5); // Grid should have expanded to 5 rows
        assert_eq!(grid.get(1, 4), Some(&Some("test_node".to_string())));
    }

    #[test]
    fn test_grid_does_not_expand_when_can_expand_y_is_false() {
        let mut grid = InnerGrid::new(3, 3);
        grid.can_expand_y = false; // Set can_expand_y to false
        let node = Node {
            id: String::from("test_node"),
            w: 1,
            h: 1,
            x: 1,
            y: 4,
        };

        // Try to add node at y=4 (beyond current grid size) with can_expand_y=false
        let result = grid.update(&node, 1, 4, UpdateGridOperation::Add);

        // Verify operation failed with OutOfBoundsAccess
        assert!(matches!(
            result,
            Err(InnerGridError::OutOfBoundsAccess { x: 1, y: 4 })
        ));

        // Verify grid size hasn't changed
        assert_eq!(grid.rows(), 3);
    }
}
