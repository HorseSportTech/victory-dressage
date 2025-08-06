pub mod application_page;
mod application_state;
pub mod battery;
mod managed_state;
pub mod store;
pub mod users;

pub use application_state::ApplicationState;
pub use managed_state::{ManagedApplicationState, StatefulRequestError};

const API_KEY: &str = env!("API_KEY");
