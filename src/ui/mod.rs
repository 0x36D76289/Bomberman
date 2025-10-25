/// Buttons are the interactable building blocks of UI pages
pub mod button;
/// Canvases are the visible elements of a UI
pub mod canvas;
/// The shared data between UI Pages for consistency
pub mod consts;
/// The list of UI Pages
pub mod pages;
/// The shared logic of UI pages
pub mod ui_state;
/// Shortcuts to tools used to build ui pages
pub mod utils;

pub use ui_state::UiState;
