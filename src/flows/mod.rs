mod medium_created;

use crate::{flows::medium_created::setup_medium_created_flow, state::AppState};

pub fn setup_flows(state: AppState) {
    setup_medium_created_flow(state);
}
