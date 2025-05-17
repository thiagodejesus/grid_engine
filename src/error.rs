use thiserror::Error;

#[derive(Error, Debug)]
pub enum GridEngineError {
    #[error(transparent)]
    InnerGrid(#[from] InnerGridError),

    #[error(transparent)]
    Item(#[from] ItemError),

    // Temporary error for unhandled errors, must be removed and all errors should be handled
    #[error("UnhandledError: {0}")]
    Unhandled(Box<dyn std::error::Error>),
}

#[derive(Error, Debug)]
pub enum InnerGridError {
    #[error("Out of bounds access: x: {x}, y: {y}")]
    OutOfBoundsAccess { x: usize, y: usize },

    #[error("RawGrid item not matching grid items: id: {id}")]
    MismatchedGridItem { id: String },
}

#[derive(Error, Debug)]
pub enum ItemError {
    #[error("Item not found: {id}")]
    ItemNotFound { id: String },

    #[error("Item already exists: {id}")]
    ItemAlreadyExists { id: String },
}
