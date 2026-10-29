pub mod app_state;
pub mod keymaps;
pub mod views;
pub mod overlays;
pub mod completion;

pub use app_state::{AppState, NavMode};
pub use keymaps::handle_key_binding;
pub use completion::{CompletionItem, CompletionKind, SymbolIndex, fuzzy_score, PopupWidget};

pub type TuiState = AppState;
