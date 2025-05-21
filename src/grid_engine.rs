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

//! Grid Engine manages a 2D grid system with support for adding, removing, and moving items.
//!
//! # Key Features
//!
//! - Automatic collision detection and handling
//! - Event system for tracking changes
//! - Expanding the grid on the y axis dynamically
//!
//! # Example
//!
//! ```
//! use grid_engine::grid_engine::GridEngine;
//! # use std::error::Error;
//!
//! # fn main() -> Result<(), Box<dyn Error>> {
//! let mut grid = GridEngine::new(10, 12);
//!
//! // Add items to the grid
//! grid.add_item("item1".to_string(), 2, 2, 2, 4)?;
//!
//! // Move items (handles collisions automatically)
//! grid.move_item("item1", 4, 4)?;
//!
//! // Remove items
//! grid.remove_item("item1")?;
//! #
//! # Ok(())
//! # }
//! ```

use crate::error::{GridEngineError, InnerGridError, ItemError};
use crate::grid_events::{ChangesEventValue, GridEvents};
use crate::inner_grid::{InnerGrid, UpdateGridOperation};
use crate::node::Node;
use crate::utils::{ForCellArgs, for_cell};
use std::{collections::BTreeMap, fmt::Debug};

/// Represents data for an item addition change
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct AddChangeData {
    /// The node being added to the grid
    pub value: Node,
}

/// Represents data for an item removal change
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct RemoveChangeData {
    /// The node being removed from the grid
    pub value: Node,
}

/// Represents data for an item movement change
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct MoveChangeData {
    /// The original state of the node
    pub old_value: Node,
    /// The new state of the node after movement
    pub new_value: Node,
}

/// Represents different types of changes that can occur in the grid
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Change {
    /// Adding a new item to the grid
    Add(AddChangeData),
    /// Removing an existing item from the grid
    Remove(RemoveChangeData),
    /// Moving an item to a new position
    Move(MoveChangeData),
}

/// The main engine for managing a 2D grid system.
///
/// `GridEngine` provides functionality for:
/// - Adding items to specific grid positions
/// - Moving items while handling collisions
/// - Removing items from the grid
/// - Tracking changes through an event system
/// - Expand the grid dynamically on the y axis
///
/// When items collide during placement or movement, the engine automatically
/// repositions affected items to prevent overlapping, the default is to move the collided items down, increasing their y axis.
#[derive(Debug)]
pub struct GridEngine {
    /// The underlying grid structure
    grid: InnerGrid,
    /// Map of item IDs to their Node representations
    items: BTreeMap<String, Node>,
    /// Changes waiting to be applied
    pending_changes: Vec<Change>,
    /// Event system for tracking grid changes
    pub events: GridEvents,
}

impl GridEngine {
    /// Creates a new GridEngine with specified dimensions.
    ///
    /// # Arguments
    ///
    /// * `rows` - Initial number of rows in the grid
    /// * `cols` - Initial number of columns in the grid
    ///
    /// # Example
    ///
    /// ```
    /// use grid_engine::grid_engine::GridEngine;
    ///
    /// let grid = GridEngine::new(10, 10); // Creates a 10x10 grid
    /// ```
    pub fn new(rows: usize, cols: usize) -> GridEngine {
        GridEngine {
            grid: InnerGrid::new(rows, cols),
            items: BTreeMap::new(),
            pending_changes: Vec::new(),
            events: GridEvents::default(),
        }
    }

    /// Creates a new node with the specified parameters.
    fn new_node(&mut self, id: String, x: usize, y: usize, w: usize, h: usize) -> Node {
        let node = Node::new(id, x, y, w, h);
        node
    }

    /// Creates a change operation to add a new node to the grid.
    fn create_add_change(&mut self, node: Node) {
        self.pending_changes
            .push(Change::Add(AddChangeData { value: node }));
    }

    /// Get the nodes sorted by id
    ///
    /// # Example
    ///
    /// ```
    /// use grid_engine::grid_engine::GridEngine;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut grid = GridEngine::new(10, 10);
    /// grid.add_item("b".to_string(), 0, 0, 2, 2).unwrap();
    /// grid.add_item("a".to_string(), 0, 2, 2, 2).unwrap();
    ///
    /// let nodes = grid.get_nodes();
    /// assert_eq!(nodes.len(), 2);
    /// assert_eq!(nodes[0].id, "a");
    /// assert_eq!(nodes[1].id, "b");
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_nodes(&self) -> Vec<Node> {
        let mut cloned: Vec<Node> = self.items.values().cloned().collect();
        // Would be better to sort by some created_at
        cloned.sort_by_key(|n| n.id.clone());
        cloned
    }

    /// Gets a reference to the underlying grid structure.
    ///
    /// This provides access to the raw grid data for inspection purposes.
    /// Note that modifications should be made through GridEngine's public methods
    /// rather than directly manipulating the inner grid.
    ///
    /// # Returns
    ///
    /// A reference to the InnerGrid instance
    ///
    /// # Example
    ///
    /// ```
    /// use grid_engine::grid_engine::GridEngine;
    /// use std::error::Error;
    ///
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let grid = GridEngine::new(10, 10);
    /// let inner_grid = grid.get_inner_grid();
    /// assert_eq!(inner_grid.rows(), 10);
    /// assert_eq!(inner_grid.cols(), 10);
    /// # Ok(())
    /// # }
    pub fn get_inner_grid(&self) -> &InnerGrid {
        &self.grid
    }

    /// Adds an item to the grid at the specified position.
    ///
    /// If the new item would collide with existing items, those items are
    /// automatically repositioned to avoid overlap.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the item
    /// * `x` - X coordinate (column) for item placement
    /// * `y` - Y coordinate (row) for item placement
    /// * `w` - Width of the item in grid cells
    /// * `h` - Height of the item in grid cells
    ///
    /// # Returns
    ///
    /// * `Ok(&Node)` - Reference to the newly added node
    /// * `Err(GridEngineError)` - If item already exists or placement fails
    ///
    /// # Example
    ///
    /// ```
    /// use grid_engine::grid_engine::GridEngine;
    /// use std::error::Error;
    ///
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let mut grid = GridEngine::new(10, 10);
    /// grid.add_item("box1".to_string(), 0, 0, 2, 2)?; // 2x2 item at top-left
    ///
    /// // Check if the item was added correctly
    /// let item = grid.get_nodes();
    /// assert_eq!(item.len(), 1);
    /// assert_eq!(item[0].id, "box1");
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_item(
        &mut self,
        id: String,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
    ) -> Result<&Node, GridEngineError> {
        if self.items.get(&id).is_some() {
            return Err(GridEngineError::ItemError(ItemError::ItemAlreadyExists {
                id: id.clone(),
            }));
        };

        let node = self.new_node(id, x, y, w, h);
        let node_id = node.id.to_string();

        self.handle_collision(&node, x, y, &mut self.grid.clone())?;

        self.create_add_change(node);

        self.apply_changes(&self.pending_changes.clone())?;
        self.pending_changes.clear();

        let node = self
            .items
            .get(&node_id)
            .ok_or(InnerGridError::MismatchedGridItem { id: node_id })?;
        Ok(&node)
    }

    fn create_remove_change(&mut self, node: &Node) {
        self.pending_changes.push(Change::Remove(RemoveChangeData {
            value: node.clone(),
        }));
    }

    /// Removes an item from the grid by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the item to remove
    ///
    /// # Returns
    ///
    /// * `Ok(Node)` - The removed node
    /// * `Err(GridEngineError)` - If item doesn't exist
    ///
    /// # Example
    ///
    /// ```
    /// use grid_engine::grid_engine::GridEngine;
    /// use std::error::Error;
    ///
    /// # fn main() -> Result<(), Box<dyn Error>> {
    ///
    /// let mut grid = GridEngine::new(10, 10);
    /// grid.add_item("box1".to_string(), 0, 0, 2, 2)?;
    /// grid.remove_item("box1")?; // Removes the item
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn remove_item(&mut self, id: &str) -> Result<Node, GridEngineError> {
        let node = match self.items.get(id) {
            Some(node) => node,
            None => Err(GridEngineError::ItemError(ItemError::ItemNotFound {
                id: id.to_string(),
            }))?,
        }
        .clone();

        self.create_remove_change(&node);

        self.apply_changes(&self.pending_changes.clone())?;
        self.pending_changes.clear();
        Ok(node)
    }

    /// Checks if a node would collide with any existing items at the specified position.
    ///
    /// This is used internally to detect potential collisions before making grid changes.
    /// It considers the node's dimensions and any existing items in the target area.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<&Node>)` - List of nodes that would collide with the given node
    /// * `Err(InnerGridError)` - If position check fails (e.g., out of bounds)
    fn will_collides_with(
        &self,
        node: &Node,
        x: usize,
        y: usize,
        grid: &mut InnerGrid,
    ) -> Result<Vec<&Node>, InnerGridError> {
        let mut collides_with: Vec<&Node> = Vec::new();

        for_cell(
            ForCellArgs {
                x,
                y,
                w: node.w,
                h: node.h,
            },
            &mut |x, y| {
                let cell = grid
                    .get(x, y)
                    .ok_or(InnerGridError::OutOfBoundsAccess { x, y })?;

                match cell {
                    Some(cell_ref) => {
                        if cell_ref != &node.id {
                            let node = self.items.get(cell_ref).ok_or(
                                InnerGridError::MismatchedGridItem {
                                    id: cell_ref.to_string(),
                                },
                            )?;

                            if !collides_with.contains(&node) {
                                collides_with.push(&node);
                            }
                        }
                    }
                    None => {
                        // Nothing to collide with
                    }
                }
                Ok(())
            },
        )?;

        Ok(collides_with)
    }

    /// Handles collision resolution when adding or moving items.
    ///
    /// When a collision is detected, this method:
    /// 1. Identifies all affected items
    /// 2. Calculates new positions for colliding items
    /// 3. Creates appropriate move changes to relocate affected items
    ///
    /// The default collision resolution strategy moves affected items downward,
    /// which may trigger dynamic grid expansion in the y-axis.
    fn handle_collision(
        &mut self,
        node: &Node,
        x: usize,
        y: usize,
        grid: &mut InnerGrid,
    ) -> Result<(), InnerGridError> {
        let collides_with = self
            .will_collides_with(node, x, y, grid)?
            .iter()
            .map(|n| (*n).clone())
            .collect::<Vec<Node>>();

        for collided in collides_with {
            let mut new_grid = grid.clone();

            node.update_grid(&mut new_grid, UpdateGridOperation::Remove)?;
            let new_x = collided.x;
            let new_y = y + node.h;
            self.create_move_change(collided, new_x, new_y, &mut new_grid)?;
        }

        Ok(())
    }

    /// Creates a change operation to move a node to a new position.
    ///
    /// This method:
    /// 1. Handles any collisions at the new position
    /// 2. Checks if the node was already scheduled to move
    /// 3. Creates a Move change operation if needed
    ///
    /// # Arguments
    ///
    /// * `node` - The node to move
    /// * `new_x` - Target x coordinate
    /// * `new_y` - Target y coordinate
    /// * `grid` - The grid to check for collisions
    fn create_move_change(
        &mut self,
        node: Node,
        new_x: usize,
        new_y: usize,
        grid: &mut InnerGrid,
    ) -> Result<(), InnerGridError> {
        let old_node = node.clone();
        self.handle_collision(&node, new_x, new_y, grid)?;

        let already_moved = self.pending_changes.iter().any(|change| match change {
            Change::Move(data) => data.new_value.id == node.id,
            _ => false,
        });

        if already_moved {
            return Ok(());
        }

        self.pending_changes.push(Change::Move(MoveChangeData {
            old_value: old_node,
            new_value: Node::new(node.id.to_string(), new_x, new_y, node.w, node.h),
        }));

        Ok(())
    }

    /// Moves an existing item to a new position in the grid.
    ///
    /// If the move would cause collisions, affected items are automatically
    /// repositioned to prevent overlap.
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the item to move
    /// * `new_x` - New X coordinate
    /// * `new_y` - New Y coordinate
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If move successful
    /// * `Err(GridEngineError)` - If item doesn't exist or move invalid
    ///
    /// # Example
    ///
    /// ```
    /// use grid_engine::grid_engine::GridEngine;
    /// # use std::error::Error;
    ///
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// 
    /// let mut grid = GridEngine::new(10, 10);
    /// grid.add_item("box1".to_string(), 0, 0, 2, 2)?;
    /// grid.move_item("box1", 2, 2)?; // Moves box to position 2,2
    /// 
    /// // Check if the item was moved correctly
    /// let item = grid.get_nodes();
    /// assert_eq!(item.len(), 1);
    /// assert_eq!(item[0].x, 2);
    /// assert_eq!(item[0].y, 2);
    /// 
    /// # Ok(())
    /// # }
    /// 
    /// ```
    pub fn move_item(
        &mut self,
        id: &str,
        new_x: usize,
        new_y: usize,
    ) -> Result<(), GridEngineError> {
        let node = match self.items.get(id) {
            Some(node) => node,
            None => Err(GridEngineError::ItemError(ItemError::ItemNotFound {
                id: id.to_string(),
            }))?,
        };

        self.create_move_change(node.clone(), new_x, new_y, &mut self.grid.clone())?;

        self.apply_changes(&self.pending_changes.clone())?;
        self.pending_changes.clear();

        Ok(())
    }

    /// Applies a batch of changes to the grid.
    ///
    /// This method handles the actual application of all pending changes to both
    /// the grid structure and the item tracking system. Changes are applied in order,
    /// and all operations are executed atomically - if any change fails, none of
    /// the changes will be applied.
    ///
    /// After successful application, triggers change events to notify any registered listeners.
    ///
    /// # Arguments
    ///
    /// * `changes` - Vector of changes to apply (Add, Remove, or Move operations)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If all changes were applied successfully
    /// * `Err(GridEngineError)` - If any change application fails
    /// ```
    fn apply_changes(&mut self, changes: &Vec<Change>) -> Result<(), GridEngineError> {
        for change in changes.iter() {
            match &change {
                Change::Add(data) => {
                    let node = &data.value;

                    node.update_grid(&mut self.grid, UpdateGridOperation::Add)?;

                    self.items.insert(node.id.to_string(), node.clone());
                }
                Change::Remove(data) => {
                    let node = &data.value;

                    node.update_grid(&mut self.grid, UpdateGridOperation::Remove)?;

                    self.items.remove(&node.id);
                }
                Change::Move(data) => {
                    let node = &data.new_value;
                    let old_node = &data.old_value;

                    old_node.update_grid(&mut self.grid, UpdateGridOperation::Remove)?;

                    self.items.insert(node.id.to_string(), node.clone());

                    node.update_grid(&mut self.grid, UpdateGridOperation::Add)?;
                }
            }
        }

        self.events.trigger_changes_event(&ChangesEventValue {
            changes: changes.iter().map(|change| change.clone()).collect(),
        });
        Ok(())
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_for_cell() {
        let mut results = Vec::new();
        let mut callback = |x: usize, y: usize| {
            results.push((x, y));
            Ok(())
        };

        for_cell(
            ForCellArgs {
                x: 1,
                y: 2,
                w: 2,
                h: 2,
            },
            &mut callback,
        )
        .unwrap();

        assert_eq!(results, vec![(1, 2), (1, 3), (2, 2), (2, 3)]);
    }

    #[test]
    fn test_add_item() {
        let mut engine = GridEngine::new(10, 10);
        let item_0_id = engine
            .add_item("0".to_string(), 0, 0, 2, 2)
            .unwrap()
            .id
            .clone();

        assert!(engine.items.len() == 1);
        for_cell(
            ForCellArgs {
                x: 0,
                y: 0,
                w: 2,
                h: 2,
            },
            &mut |x, y| {
                assert_eq!(engine.grid.get(x, y).unwrap().as_ref().unwrap(), &item_0_id);
                Ok(())
            },
        )
        .unwrap();
    }

    #[test]
    fn test_add_item_handle_duplicated_id() {
        let mut engine = GridEngine::new(10, 10);
        engine.add_item("0".to_string(), 0, 0, 2, 2).unwrap();

        assert!(engine.add_item("0".to_string(), 0, 0, 2, 2).is_err())
    }

    #[test]
    fn test_add_item_handle_collision() {
        let mut engine = GridEngine::new(10, 10);
        let item_0_id = engine
            .add_item("0".to_string(), 0, 0, 2, 2)
            .unwrap()
            .id
            .clone();
        let item_1_id = engine
            .add_item("1".to_string(), 0, 0, 2, 2)
            .unwrap()
            .id
            .clone();

        // Item 0 should stay in position 0, 0
        let item_0 = engine.items.get(&item_0_id).unwrap();
        assert_eq!(item_0.x, 0);
        assert_eq!(item_0.y, 2);
        item_0
            .for_cell(&mut |x, y| {
                assert_eq!(engine.grid.get(x, y).unwrap().as_ref().unwrap(), &item_0_id);
                Ok(())
            })
            .unwrap();

        // Item 1 should go to position 0, 2
        let item_1 = engine.items.get(&item_1_id).unwrap();
        assert_eq!(item_1.x, 0);
        assert_eq!(item_1.y, 0);
        item_1
            .for_cell(&mut |x, y| {
                assert_eq!(engine.grid.get(x, y).unwrap().as_ref().unwrap(), &item_1_id);
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn test_remove_item() {
        let mut engine = GridEngine::new(10, 10);
        let item_0_id = engine
            .add_item("0".to_string(), 0, 0, 2, 3)
            .unwrap()
            .id
            .clone();
        engine.remove_item(&item_0_id).unwrap();
        for_cell(
            ForCellArgs {
                x: 0,
                y: 0,
                w: 2,
                h: 3,
            },
            &mut |x, y| {
                let value = engine.grid.get(x, y).unwrap();
                assert_eq!(value, &None);
                Ok(())
            },
        )
        .unwrap();
    }

    #[test]
    fn test_move_item() {
        let mut engine = GridEngine::new(10, 10);
        let item_0_id = engine
            .add_item("0".to_string(), 0, 0, 2, 2)
            .unwrap()
            .id
            .clone();
        engine.move_item(&item_0_id, 1, 1).unwrap();

        // Asserts that its present on the new position
        for_cell(
            ForCellArgs {
                x: 1,
                y: 1,
                w: 2,
                h: 2,
            },
            &mut |x, y| {
                let item_on_expected_position = engine.grid.get(x, y).unwrap().as_ref().unwrap();
                assert_eq!(item_on_expected_position, &item_0_id);
                Ok(())
            },
        )
        .unwrap();

        // Asserts that its not present on the old position
        for_cell(
            ForCellArgs {
                x: 0,
                y: 0,
                w: 1,
                h: 1,
            },
            &mut |x, y| {
                assert_eq!(engine.grid.get(x, y).unwrap(), &None);
                Ok(())
            },
        )
        .unwrap();
    }

    #[test]
    fn test_move_item_handle_collision() {
        let mut engine = GridEngine::new(10, 10);
        let item_0_id = engine
            .add_item("0".to_string(), 0, 0, 2, 2)
            .unwrap()
            .id
            .clone();
        let item_1_id = engine
            .add_item("1".to_string(), 0, 2, 2, 2)
            .unwrap()
            .id
            .clone();
        engine.move_item("0", 0, 1).unwrap();

        // Item 0 should go to position 0, 1
        let item_0 = engine.items.get(&item_0_id).unwrap();
        assert_eq!(item_0.x, 0);
        assert_eq!(item_0.y, 1);
        item_0
            .for_cell(&mut |x, y| {
                assert_eq!(engine.grid.get(x, y).unwrap().as_ref().unwrap(), &item_0_id);
                Ok(())
            })
            .unwrap();

        // Item 1 should go to position 0, 3
        let item_1 = engine.items.get(&item_1_id).unwrap();
        assert_eq!(item_1.x, 0);
        assert_eq!(item_1.y, 3);
        item_1
            .for_cell(&mut |x, y| {
                assert_eq!(engine.grid.get(x, y).unwrap().as_ref().unwrap(), &item_1_id);
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn test_will_collides_with() {
        let mut engine = GridEngine::new(10, 10);
        let item_0_id = engine
            .add_item("0".to_string(), 0, 0, 1, 2)
            .unwrap()
            .id
            .clone();

        // Asserts that does not collide with self
        assert!(
            engine
                .will_collides_with(
                    &engine.items.get(&item_0_id).unwrap(),
                    0,
                    0,
                    &mut engine.grid.clone()
                )
                .unwrap()
                .len()
                == 0
        );

        // Asserts that does not collide with empty position
        assert!(
            engine
                .will_collides_with(
                    &engine.items.get(&item_0_id).unwrap(),
                    2,
                    2,
                    &mut engine.grid.clone()
                )
                .unwrap()
                .len()
                == 0
        );

        // Asserts that collide with occupied position
        engine.add_item("1".to_string(), 1, 2, 1, 2).unwrap();

        // Full collision
        assert!(
            engine
                .will_collides_with(
                    &engine.items.get(&item_0_id).unwrap(),
                    1,
                    2,
                    &mut engine.grid.clone()
                )
                .unwrap()
                .len()
                == 1
        );

        // Partial collision
        assert!(
            engine
                .will_collides_with(
                    &engine.items.get(&item_0_id).unwrap(),
                    1,
                    1,
                    &mut engine.grid.clone()
                )
                .unwrap()
                .len()
                == 1
        );
    }

    #[test]
    fn test_get_nodes() {
        let mut engine = GridEngine::new(10, 10);
        let item_0_id = engine
            .add_item("0".to_string(), 0, 0, 2, 2)
            .unwrap()
            .id
            .clone();
        let item_1_id = engine
            .add_item("1".to_string(), 0, 2, 2, 2)
            .unwrap()
            .id
            .clone();

        let nodes = engine.get_nodes();
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].id, item_0_id);
        assert_eq!(nodes[1].id, item_1_id);
    }

    #[test]
    fn test_move_result_will_not_collides_with_moving_item() {
        let mut engine = GridEngine::new(10, 10);
        engine.add_item("0".to_string(), 0, 0, 2, 3).unwrap();
        engine.add_item("1".to_string(), 0, 6, 2, 2).unwrap();
        engine.move_item("1", 0, 2).unwrap();

        for_cell(
            ForCellArgs {
                x: 0,
                y: 7,
                w: 2,
                h: 2,
            },
            &mut |x, y| {
                let value = engine.grid.get(x, y).unwrap();
                println!("value: {:?}", value);
                assert_ne!(value, &Some("1".to_string()));
                Ok(())
            },
        )
        .unwrap();
    }

    #[test]
    fn test_node_movements_that_collides_twice_works() {
        let mut engine = GridEngine::new(14, 10);
        engine.add_item("0".to_string(), 1, 1, 2, 3).unwrap();
        engine.add_item("1".to_string(), 2, 4, 2, 4).unwrap();
        engine.add_item("2".to_string(), 0, 6, 2, 4).unwrap();
        engine.move_item("2", 1, 2).unwrap();

        println!("Items: {:#?}", engine.items);

        engine.items.iter().for_each(|(_, node)| {
            node.for_cell(&mut |x, y| {
                let value = engine.grid.get(x, y).unwrap();
                println!("Validating x: {}, y: {}", x, y);
                assert_eq!(&Some(node.clone().id), value);
                Ok(())
            })
            .unwrap();
        });
    }
}
