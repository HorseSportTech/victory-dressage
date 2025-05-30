use futures_util::SinkExt;
use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_store::StoreExt;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::{self, Message};

use crate::STORE_URI;

use super::manager::{TxWS, STORED_MESSAGES};
use super::message_types::AppSocketMessage;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SocketMessage {
    pub id: ulid::Ulid,
    pub ver: u8,
    pub msg: AppSocketMessage,
}
impl SocketMessage {
    pub fn new(msg: AppSocketMessage) -> Self {
        Self {
            id: ulid::Ulid::new(),
            ver: 2,
            msg,
        }
    }
    pub async fn generate(
        handle: tauri::AppHandle,
        msg: AppSocketMessage,
    ) -> Result<(), tokio_tungstenite::tungstenite::Error> {
        let original_message = Self::new(msg);
        let encode_message = rmp_serde::to_vec(&original_message).map_err(|_err| {
            tungstenite::Error::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to serialize data",
            ))
        })?;
        {
            let guarded_tx = handle.state::<Arc<Mutex<TxWS>>>();
            let mut tx = guarded_tx.lock().await;
            tx.send(Message::Binary(encode_message.into())).await?;
        }
        {
            if let Ok(store) = handle.store(STORE_URI) {
                let mut prev_messages = store
                    .get(STORED_MESSAGES)
                    .and_then(|msgs| serde_json::from_value(msgs).unwrap_or_else(|_| Some(vec![])))
                    .unwrap_or_else(|| vec![]);
                prev_messages.push(original_message);
                store.set(
                    STORED_MESSAGES,
                    serde_json::to_value(prev_messages).expect("Should be able to parse"),
                );
            }
        }
        Ok(())
    }
}
// pub trait TransferableSocketMessage: Clone + serde::Serialize + serde::Deserialize<'static> {
// 	fn wrap(self) -> SocketMessage<AppSocketMessage>;
// }

