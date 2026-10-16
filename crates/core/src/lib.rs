//! rjot's data: notes and queued prompts kept in one local JSON file, plus
//! a free-form scratchpad. No sync, no account.

pub mod store;
pub mod time;

pub use store::{Appearance, Color, Item, Kind, Settings, Store};
