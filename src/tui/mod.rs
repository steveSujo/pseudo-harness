pub mod app_state;
pub mod keymaps;
pub mod views;
pub mod overlays;

pub use app_state::{AppState, NavMode};
pub use keymaps::handle_key_binding;

pub type TuiState = AppState;
