use std::sync::Mutex;
use crate::db::{EventStore, ProjectionStore};

pub struct AppState {
    pub events: Mutex<EventStore>,
    pub projections: Mutex<ProjectionStore>,
}
