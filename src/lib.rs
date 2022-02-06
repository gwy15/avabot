#[macro_use]
extern crate log;

mod config;
pub mod plugins;
pub mod prelude {
    pub use anyhow::*;
    pub use miraie::prelude::*;
    pub use serde::{Deserialize, Serialize};
    pub use std::time::Duration;
    pub use tokio::time::sleep;
}
pub use config::Config;
