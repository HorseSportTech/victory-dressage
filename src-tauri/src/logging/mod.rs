use std::{fs::OpenOptions, io, io::Write};

use serde_json::Value;
use tauri::Manager;
use tauri_plugin_store::StoreExt;

use crate::state::ManagedApplicationState;

pub struct Logger(tauri::AppHandle);

const LOG: &str = "LOGGING_TRAIL";
impl Logger {
    pub fn new(app_handle: &tauri::AppHandle) -> Self {
        Self(app_handle.clone())
    }
    pub fn log(&self, logging_data: Box<dyn Loggable>) -> Result<(), LoggingError> {
        let date = chrono::Utc::now();
        let app_state = self.0.state::<ManagedApplicationState>();
        let user = app_state
            .read(|aps| aps.user.to_log())
            .map_err(|_| LoggingError::CouldNotAccessState)?;

        let log_data = logging_data.to_log();

        let log_line = format!("@{date} {user} - {log_data}");
        self.log_to_file(&log_line)?;
        let store = self.0.store(env!("STORE_URI"))?;
        let mut entries = match store.get(LOG) {
            Some(Value::Array(arr)) => arr,
            _ => vec![],
        };
        entries.push(Value::String(log_line));
        store.set(LOG, Value::Array(entries));

        Ok(())
    }
    fn log_to_file(&self, output: &str) -> Result<(), LoggingError> {
        let document_dir = self.0.path().document_dir();
        let Ok(mut document_path) = document_dir else {
            return Err(
                io::Error::new(io::ErrorKind::NotFound, "No document directory found").into(),
            );
        };

        document_path.push("Log.txt");

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(document_path)?;

        writeln!(file, "{output}")?;
        Ok(())
    }
    pub fn get_logs(&self) -> String {
        let entries: Vec<String> = self
            .0
            .store(env!("STORE_URI"))
            .ok()
            .and_then(|store| store.get(LOG))
            .and_then(|arr| serde_json::from_value(arr).ok())
            .unwrap_or_default();
        entries.join("\n")
    }
}

#[derive(thiserror::Error, Debug)]
pub enum LoggingError {
    #[error("Could not store in datastore")]
    CouldNotStore,
    #[allow(unused)]
    #[error("Could not access state")]
    CouldNotAccessState,
    #[error(transparent)]
    Io(#[from] io::Error),
}
impl From<tauri_plugin_store::Error> for LoggingError {
    fn from(_: tauri_plugin_store::Error) -> Self {
        Self::CouldNotStore
    }
}
pub trait Loggable {
    fn to_log(&self) -> String;
}
