pub mod application_page;
mod application_state;
pub mod battery;
mod managed_state;
pub mod users;

pub use application_state::ApplicationState;
pub use managed_state::{ManagedApplicationState, StatefulRequestError};
pub use users::UserType;

const API_KEY: &str = env!("API_KEY");
