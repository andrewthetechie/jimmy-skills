pub mod config;
pub mod fanout;
pub mod handler;
pub mod synthesizer;
pub mod types;

use std::sync::Arc;

use config::RouterConfig;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<RouterConfig>,
    pub client: Arc<reqwest::Client>,
}
