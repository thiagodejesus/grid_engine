use std::fmt::Debug;

use crate::grid_engine::Change;
use crate::grid_view::GridView;

#[derive(Debug, Clone)]
pub struct ChangesEventValue {
    pub changes: Vec<Change>,
}

pub type ChangesEventFn = Box<dyn Fn(&GridView, &ChangesEventValue) -> () + Send + 'static + Sync>;

pub struct ListenerFunction {
    pub id: String,
    pub function: ChangesEventFn,
}

impl Debug for ListenerFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ListenerFunction")
            .field("id", &self.id)
            .finish()
    }
}

#[derive(Debug)]
pub struct GridEvents {
    changes_listeners: Vec<ListenerFunction>,
}

impl GridEvents {
    pub fn add_changes_listener(&mut self, function: ChangesEventFn) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let listener = ListenerFunction {
            id: id.clone(),
            function,
        };

        self.changes_listeners.push(listener);
        id
    }

    pub fn remove_changes_listener(&mut self, id: &str) {
        self.changes_listeners.retain(|listener| listener.id != id);
    }

    pub fn trigger_changes_event(&mut self, grid: &GridView, value: &ChangesEventValue) {
        for listener in &mut self.changes_listeners {
            (listener.function)(grid, value);
        }
    }
}

impl Default for GridEvents {
    fn default() -> Self {
        Self {
            changes_listeners: Vec::new(),
        }
    }
}
