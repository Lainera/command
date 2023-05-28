#![cfg(all(feature = "owned", feature = "stderr"))]

use crate::CommandError;

impl std::error::Error for CommandError {
    fn description(&self) -> &str {
        self.into()
    }
}
