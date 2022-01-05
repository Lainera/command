#![cfg(feature = "owned")]

extern crate std;

use crate::CommandError;

impl std::error::Error for CommandError {
    fn description(&self) -> &str {
        self.into()
    }
}
