use macro_state::{proc_read_state, proc_write_state};
use serde::{de::DeserializeOwned, Serialize};
use syn::{Error, Ident};

use crate::errors::format_err;

pub trait State: Serialize + DeserializeOwned {
    const TYPE_NAME: &'static str;

    fn save_state(&self, name: &Ident) -> Result<(), Error> {
        proc_write_state(
            &format!("{}.{name}", Self::TYPE_NAME),
            &serde_json::to_string(self)
                .map_err(|e| format_err!(@name, "Coastal serialisation failed: {e}"))?,
        )
        .map_err(|e| format_err!(@name, "Coastal failed to save state: {e}"))
    }

    fn load_state(name: &Ident) -> Result<Self, Error> {
        let state = proc_read_state(&format!("{}.{name}", Self::TYPE_NAME))
            .map_err(|e| format_err!(@name, "Coastal could not find the function '{name}': {e}"))?;
        let this = serde_json::from_str(&state).map_err(|e| {
            format_err!(
                @name, "Coastal failed to deserialise the state of function '{name}': {e}"
            )
        })?;
        Ok(this)
    }
}
