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

//! Event system for the grid engine that allows tracking and responding to grid changes.
//!
//! This module provides an event-driven mechanism to observe changes in the grid,
//! such as items being added, removed, or moved. It supports registering multiple
//! listeners that can react to these changes in real-time.
//!
//! # Example
//!
//! ```
//! use grid_engine::grid_engine::GridEngine;
//!
//! let mut grid = GridEngine::new(10, 10);
//!
//! // Add a listener to track changes
//! grid.events.add_changes_listener(Box::new(|event| {
//!     println!("Grid changed: {:?}", event.changes);
//! }));
//!
//! // Make changes to the grid
//! grid.add_item("box1".to_string(), 0, 0, 2, 2).unwrap();
//! // The listener will be notified automatically
//! ```

use crate::grid_engine::Change;
use std::fmt::Debug;

/// Event data structure containing information about grid changes.
///
/// This structure is passed to event listeners whenever changes occur in the grid,
/// providing details about what changes were made.
#[derive(Debug, Clone)]
pub struct ChangesEventValue {
    /// Vector of changes that occurred in the grid
    pub changes: Vec<Change>,
}

/// Type alias for change event listener functions.
///
/// These functions:
/// - Receive a reference to `ChangesEventValue`
pub type ChangesEventFn = Box<dyn Fn(&ChangesEventValue) + Send + 'static + Sync>;

/// Represents a registered event listener function.
///
/// Each listener has a unique ID for management purposes and holds the actual
/// callback function to be executed when changes occur.
pub struct ListenerFunction {
    /// Unique identifier for the listener
    pub id: String,
    /// The callback function to execute when changes occur
    pub function: ChangesEventFn,
}

impl Debug for ListenerFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ListenerFunction")
            .field("id", &self.id)
            .finish()
    }
}

/// Event management system for grid changes.
///
/// `GridEvents` manages a collection of event listeners that are notified
/// whenever changes occur in the grid. It provides methods to register
/// and remove listeners, as well as trigger events when changes happen.
#[derive(Debug, Default)]
pub struct GridEvents {
    /// Collection of registered change event listeners
    changes_listeners: Vec<ListenerFunction>,
}

impl GridEvents {
    /// Registers a new change event listener.
    ///
    /// When changes occur in the grid, the provided function will be called
    /// with details about those changes.
    ///
    /// # Arguments
    ///
    /// * `function` - The callback function to execute when changes occur
    ///
    /// # Returns
    ///
    /// A unique identifier string for the registered listener that can be used
    /// to remove it later.
    ///
    /// # Example
    ///
    /// ```
    /// use grid_engine::grid_engine::GridEngine;
    ///
    /// let mut grid = GridEngine::new(10, 10);
    /// let listener_id = grid.events.add_changes_listener(Box::new(|event| {
    ///     println!("Changes occurred: {:?}", event.changes);
    /// }));
    /// ```
    pub fn add_changes_listener(&mut self, function: ChangesEventFn) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let listener = ListenerFunction {
            id: id.clone(),
            function,
        };

        self.changes_listeners.push(listener);
        id
    }

    /// Removes a previously registered change event listener.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID returned when the listener was registered
    ///
    /// # Example
    ///
    /// ```
    /// use grid_engine::grid_engine::GridEngine;
    ///
    /// let mut grid = GridEngine::new(10, 10);
    /// let listener_id = grid.events.add_changes_listener(Box::new(|_| {}));
    /// grid.events.remove_changes_listener(&listener_id); // Listener removed
    /// ```
    pub fn remove_changes_listener(&mut self, id: &str) {
        self.changes_listeners.retain(|listener| listener.id != id);
    }

    /// Triggers the change event, notifying all registered listeners.
    ///
    /// This is called internally by the grid engine when changes occur.
    /// Each registered listener's callback function is executed with
    /// the provided change event value.
    ///
    /// # Arguments
    ///
    /// * `value` - The event data containing information about the changes
    pub fn trigger_changes_event(&mut self, value: &ChangesEventValue) {
        for listener in &mut self.changes_listeners {
            (listener.function)(value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_add_changes_listener() {
        let mut events = GridEvents::default();
        let listener_id = events.add_changes_listener(Box::new(|_| {}));

        assert_eq!(events.changes_listeners.len(), 1);
        assert!(!listener_id.is_empty());
    }

    #[test]
    fn test_remove_changes_listener() {
        let mut events = GridEvents::default();
        let listener_id = events.add_changes_listener(Box::new(|_| {}));

        events.remove_changes_listener(&listener_id);
        assert_eq!(events.changes_listeners.len(), 0);
    }

    #[test]
    fn test_multiple_listeners() {
        let mut events = GridEvents::default();
        let _id1 = events.add_changes_listener(Box::new(|_| {}));
        let _id2 = events.add_changes_listener(Box::new(|_| {}));

        assert_eq!(events.changes_listeners.len(), 2);
    }

    #[test]
    fn test_trigger_changes_event() {
        let mut events = GridEvents::default();
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();

        events.add_changes_listener(Box::new(move |_| {
            let mut count = counter_clone.lock().unwrap();
            *count += 1;
        }));

        let changes = ChangesEventValue { changes: vec![] };
        events.trigger_changes_event(&changes);

        assert_eq!(*counter.lock().unwrap(), 1);
    }

    #[test]
    fn test_trigger_multiple_listeners() {
        let mut events = GridEvents::default();
        let counter = Arc::new(Mutex::new(0));

        // Add two listeners that increment the same counter
        for _ in 0..2 {
            let counter_clone = counter.clone();
            events.add_changes_listener(Box::new(move |_| {
                let mut count = counter_clone.lock().unwrap();
                *count += 1;
            }));
        }

        let changes = ChangesEventValue { changes: vec![] };
        events.trigger_changes_event(&changes);

        assert_eq!(*counter.lock().unwrap(), 2);
    }

    #[test]
    fn test_listener_receives_changes() {
        let mut events = GridEvents::default();
        let received_changes = Arc::new(Mutex::new(Vec::new()));
        let received_changes_clone = received_changes.clone();

        events.add_changes_listener(Box::new(move |event| {
            let mut changes = received_changes_clone.lock().unwrap();
            changes.extend(event.changes.clone());
        }));

        // Create a mock change
        let node = crate::node::Node::new("test".to_string(), 0, 0, 1, 1);
        let change = Change::Add(crate::grid_engine::AddChangeData { value: node });
        let event = ChangesEventValue {
            changes: vec![change.clone()],
        };

        events.trigger_changes_event(&event);

        let received = received_changes.lock().unwrap();
        let received_change = received.first().unwrap();
        assert_eq!(received_change, &change);
    }
}
