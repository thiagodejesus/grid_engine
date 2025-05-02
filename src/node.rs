use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    error::InnerGridError,
    inner_grid::{InnerGrid, UpdateGridOperation},
    utils::{for_cell, ForCellArgs},
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[wasm_bindgen]
pub struct Node {
    #[wasm_bindgen(skip)]
    pub id: String,
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

#[wasm_bindgen]
impl Node {
    pub fn new(id: String, x: usize, y: usize, w: usize, h: usize) -> Node {
        Node { id, x, y, w, h }
    }

    #[wasm_bindgen(js_name = getId)]
    pub fn get_id(&self) -> String {
        self.id.clone()
    }

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

    pub(crate) fn update_grid(
        &self,
        grid: &mut InnerGrid,
        update_operation: UpdateGridOperation,
    ) -> Result<(), InnerGridError> {
        self.for_cell(&mut |x, y| {
            return grid.update(self, x, y, update_operation);
        })?;

        Ok(())
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

        assert_eq!(node.get_id(), "test_node");
        // Test that get_id returns a clone
        let id1 = node.get_id();
        let id2 = node.get_id();
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
