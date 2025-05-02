use crate::error::{GridEngineError, InnerGridError, ItemError};
use crate::grid_events::{ChangesEventValue, GridEvents};
use crate::grid_view::GridView;
use crate::inner_grid::{InnerGrid, UpdateGridOperation};
use crate::node::Node;
use crate::utils::{for_cell, ForCellArgs};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt::Debug};
use wasm_bindgen::prelude::*;

// TODO, remove unnecessary clones
// TODO, Handle all `expect` and `unwrap` properly
// TODO, set wasm as a optional feature

#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct AddChangeData {
    #[wasm_bindgen(skip)]
    pub value: Node,
}

#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct RemoveChangeData {
    #[wasm_bindgen(skip)]
    pub value: Node,
}

#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct MoveChangeData {
    #[wasm_bindgen(skip)]
    pub old_value: Node,
    #[wasm_bindgen(skip)]
    pub new_value: Node,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(tag = "type", content = "value")]
pub enum Change {
    Add(AddChangeData),
    Remove(RemoveChangeData),
    Move(MoveChangeData),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GridEngine {
    pub(crate) grid: InnerGrid,
    // TODO: Understand deeply BTreeMap and if it is the best option
    pub(crate) items: BTreeMap<String, Node>,
    #[serde(skip)]
    pending_changes: Vec<Change>,
    #[serde(skip)]
    pub events: GridEvents,
}

impl GridEngine {
    pub fn new(rows: usize, cols: usize) -> GridEngine {
        GridEngine {
            grid: InnerGrid::new(rows, cols),
            items: BTreeMap::new(),
            pending_changes: Vec::new(),
            events: GridEvents::default(),
        }
    }

    fn from_str(serialized: &str) -> Result<GridEngine, GridEngineError> {
        let grid_view: GridView = match serde_json::from_str(serialized) {
            Ok(grid_view) => grid_view,
            Err(err) => {
                println!("Error deserializing GridView {:?}", err);
                return Err(GridEngineError::UnhandledError(Box::new(err)));
            }
        };

        return Ok(GridEngine::from(&grid_view));
    }

    fn new_node(&mut self, id: String, x: usize, y: usize, w: usize, h: usize) -> Node {
        let node = Node::new(id, x, y, w, h);
        node
    }

    fn create_add_change(&mut self, node: Node) {
        self.pending_changes
            .push(Change::Add(AddChangeData { value: node }));
    }

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

    pub fn apply_changes(&mut self, changes: &Vec<Change>) -> Result<(), GridEngineError> {
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
        let grid_view = GridView::new(self);

        self.events.trigger_changes_event(
            &grid_view,
            &ChangesEventValue {
                changes: changes.iter().map(|change| change.clone()).collect(),
            },
        );
        Ok(())
    }

    pub fn get_grid_view(&self) -> GridView {
        GridView::new(self)
    }
}

impl TryFrom<&Vec<u8>> for GridEngine {
    type Error = GridEngineError;

    fn try_from(bytes: &Vec<u8>) -> Result<Self, Self::Error> {
        let serialized = match String::from_utf8(bytes.clone()) {
            Ok(serialized) => serialized,
            Err(err) => return Err(GridEngineError::UnhandledError(Box::new(err))),
        };

        let grid = match GridEngine::from_str(&serialized) {
            Ok(grid) => grid,
            Err(err) => return Err(err),
        };

        Ok(grid)
    }
}

impl Into<Vec<u8>> for &GridEngine {
    fn into(self) -> Vec<u8> {
        let serialized = serde_json::to_string(&self).expect("Failed to serialize GridEngine");
        serialized.into_bytes()
    }
}

impl From<&GridView> for GridEngine {
    fn from(grid_view: &GridView) -> Self {
        GridEngine {
            grid: grid_view.grid.clone(),
            items: grid_view.items.clone(),
            pending_changes: Vec::new(),
            events: GridEvents::default(),
        }
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

        let nodes = engine.get_grid_view().get_nodes();
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].id, item_0_id);
        assert_eq!(nodes[1].id, item_1_id);
    }

    #[test]
    fn test_serialize_and_deserialize() {
        let mut engine = GridEngine::new(10, 10);
        engine.add_item("0".to_string(), 0, 0, 2, 2).unwrap();
        engine.add_item("1".to_string(), 0, 2, 2, 2).unwrap();

        let serialized = engine.get_grid_view().serialized_as_str();
        let deserialized_engine = GridEngine::from_str(&serialized).unwrap();

        assert_eq!(
            engine.get_grid_view().get_nodes(),
            deserialized_engine.get_grid_view().get_nodes()
        );
        assert_eq!(
            engine.get_grid_view().get_grid_formatted(2),
            deserialized_engine.get_grid_view().get_grid_formatted(2)
        );
    }

    #[test]
    fn test_move_result_will_not_collides_with_moving_item() {
        let mut engine = GridEngine::new(10, 10);
        engine.add_item("0".to_string(), 0, 0, 2, 3).unwrap();
        engine.add_item("1".to_string(), 0, 6, 2, 2).unwrap();
        engine.get_grid_view().print_grid();
        engine.move_item("1", 0, 2).unwrap();

        engine.get_grid_view().print_grid();
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

        engine.get_grid_view().print_grid();
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
