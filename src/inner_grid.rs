use grid::Grid;
use serde::{Deserialize, Serialize};

use std::ops::{Deref, DerefMut};

use crate::{error::InnerGridError, node::Node};

#[derive(Debug, Clone, Copy)]
pub enum UpdateGridOperation {
    Add,
    Remove,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InnerGrid {
    can_expand_y: bool,
    inner: Grid<Option<String>>,
}

// Allow automatic dereferencing to the inner Grid
impl Deref for InnerGrid {
    type Target = Grid<Option<String>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// Allow mutable dereferencing
impl DerefMut for InnerGrid {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl InnerGrid {
    fn handle_expansion(&mut self, x: usize, y: usize) {
        let rows = self.rows();
        let cols = self.cols();

        let can_expand = self.can_expand_y && x < cols;

        if can_expand && y >= rows {
            self.expand_rows(y - rows + 1);
        }
    }

    pub fn get(&mut self, x: usize, y: usize) -> Option<&Option<String>> {
        if self.inner.get(y, x).is_none() {
            self.handle_expansion(x, y);
        }

        return self.inner.get(y, x);
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut Option<String>> {
        if self.inner.get(y, x).is_none() {
            self.handle_expansion(x, y);
        }

        return self.inner.get_mut(y, x);
    }

    pub fn new(rows: usize, cols: usize) -> Self {
        let inner = Grid::new(rows, cols);
        InnerGrid {
            inner,
            can_expand_y: true,
        }
    }

    pub fn update(
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
                *cell = Some(node.id.clone());
            }
            UpdateGridOperation::Remove => {
                if cell.as_ref() == Some(&node.id) {
                    *cell = None;
                }
            }
        }
        Ok(())
    }

    pub fn expand_rows(&mut self, rows: usize) {
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
