use futures_util::{SinkExt, StreamExt, stream};
use std::marker::PhantomData;
use std::pin::Pin;

use tokio::sync::mpsc::*;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

type AsyncFnMut<'a> = Box<
    dyn FnMut(
            Message,
        ) -> Pin<
            Box<
                dyn Future<
                        Output = std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>,
                    > + Send,
            >,
        > + Send
        + Sync,
>;
type TransformFn<'a, T = Message> = Box<dyn Fn(T) -> Message + Send + Sync + 'a>;
struct SocketManager<'a, T = Message> {
    sender: UnboundedSender<Message>,
    handler: TransformFn<'a, T>,
}

impl<T> SocketManager<'_, T> {
    pub fn send(&mut self, msg: T) -> Result<()> {
        let transformed_message = (self.handler)(msg);
        Ok(self.sender.send(transformed_message)?)
    }
}

struct Set;
struct NotSet;
/// This is the builder for the Socket Manager
pub struct SocketManagerBuilder<'a, 'b, ReceiveHandlerSet, T = Message> {
    url: &'a str,
    send_handler: Option<TransformFn<'b, T>>,
    receive_handler: Option<AsyncFnMut<'b>>,
    _receive_handler: PhantomData<ReceiveHandlerSet>,
}

impl<'a, 'b, T> SocketManagerBuilder<'a, 'b, NotSet, T> {
    pub fn new(url: &'a str) -> Self {
        Self {
            url,
            send_handler: None,
            receive_handler: None,
            _receive_handler: PhantomData,
        }
    }
}

impl<'a, 'b, ReceiveHandlerSet, T: 'b> SocketManagerBuilder<'a, 'b, ReceiveHandlerSet, T> {
    pub fn send_handler<F>(self, handler: F) -> Self
    where
        F: Fn(T) -> Message + Send + Sync + 'static,
    {
        SocketManagerBuilder {
            url: self.url,
            receive_handler: self.receive_handler,
            send_handler: Some(Box::new(handler)),
            _receive_handler: PhantomData,
        }
    }
}

impl<'a, 'b, T: 'b> SocketManagerBuilder<'a, 'b, NotSet, T>
where
    'b: 'static,
{
    /// The receive handler takes in the function which will be run when the
    /// socket receives a message. It returns tokio_tungstenite::Message::{Binary, Text} which
    /// can be used as necessary by the client.
    /// If this handler returns an Err, the Socket will close, so non-fatal errors should be managed
    /// within the handler.
    pub fn receive_handler<F, Fut, E>(self, mut handler: F) -> SocketManagerBuilder<'a, 'b, Set, T>
    where
        F: FnMut(Message) -> Fut + Sync + Send + 'static,
        Fut: Future<Output = std::result::Result<(), E>> + Send + 'static,
        E: std::error::Error + Send + Sync + 'static,
    {
        let new_handler: Option<AsyncFnMut> = Some(Box::new(move |msg| {
            let fut = handler(msg);
            Box::pin(async move {
                fut.await
                    .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })
            })
        }));
        SocketManagerBuilder {
            url: self.url,
            receive_handler: new_handler,
            send_handler: self.send_handler,
            _receive_handler: PhantomData,
        }
    }
}

impl<'a: 'b, 'b> SocketManagerBuilder<'a, 'b, Set, Message> {
    pub async fn run(self) -> Result<SocketManager<'b, Message>> {
        let (tx, rx) = unbounded_channel::<Message>();
        let receive_handler = self.receive_handler.expect("Receive handler must exist");
        let send_handler = self
            .send_handler
            .unwrap_or_else(|| Box::new(|msg| -> Message { msg }));

        tokio::spawn(manage_socket(receive_handler, rx, self.url.to_string()));
        Ok(SocketManager {
            sender: tx,
            handler: send_handler,
        })
    }
}
async fn manage_socket<'a>(
    mut receive_handler: AsyncFnMut<'a>,
    mut rx: UnboundedReceiver<Message>,
    url: String,
) -> Result<()> {
    let request = url.into_client_request()?;
    let (socket, _err) = connect_async(request).await?;
    let (mut soc_tx, mut soc_rx) = socket.split();
    let mut sender_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let mut batch = vec![Ok(msg)];

            while let Ok(msg) = rx.try_recv() {
                batch.push(Ok(msg));
            }

            let mut stream = stream::iter(batch);
            if let Err(err) = soc_tx.send_all(&mut stream).await {
                #[cfg(debug_assertions)]
                println!("SOCKET SEND ERROR. {err:?}");
                break;
            };
            if let Err(err) = soc_tx.flush().await {
                #[cfg(debug_assertions)]
                println!("SOCKET FLUSH ERROR. {err:?}");
            }

            if rx.sender_strong_count() == 0 {
                rx.close();
                break;
            }
        }
    });
    let mut receive_task = tokio::spawn(async move {
        loop {
            let duration = tokio::time::Duration::from_secs(30);
            match tokio::time::timeout(duration, soc_rx.next()).await {
                Ok(Some(Ok(msg @ Message::Binary(_) | msg @ Message::Text(_)))) => {
                    match receive_handler(msg).await {
                        Ok(_) => continue,
                        Err(_) => break,
                    }
                }
                Ok(Some(Ok(Message::Close(_)))) => break,
                Ok(Some(Ok(_))) => continue,
                Ok(Some(Err(err))) => {
                    #[cfg(debug_assertions)]
                    println!("MESSAGE ERROR. {err:?}");
                    continue;
                }
                Ok(None) | Err(_) => {
                    #[cfg(debug_assertions)]
                    println!(
                        "SOCKET CLOSED. Socket waited too long for a response, or was closed externally."
                    );
                    break;
                }
            }
        }
    });

    tokio::select!(
        _ = &mut sender_task => {
            receive_task.abort()
        },
        _ = &mut receive_task => {
            sender_task.abort()
        }
    );
    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum SocketError {
    #[error("Could not send via the MPSC Channel")]
    MPSCSend,
    #[error(transparent)]
    Tungstenite(#[from] tokio_tungstenite::tungstenite::error::Error),
    #[error("Socket timed out after {0} secs")]
    TimedOut(f64),
    #[error("Socket was closed at the server. This could be a server timeout, or some other fault")]
    Closed,
}
impl<T> From<tokio::sync::mpsc::error::SendError<T>> for SocketError {
    fn from(_: tokio::sync::mpsc::error::SendError<T>) -> SocketError {
        SocketError::MPSCSend
    }
}
pub type Result<T> = std::result::Result<T, SocketError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn setup_and_send_message() {
        let mut ws = SocketManagerBuilder::new("wss://victory-hst.au/ws/dressage/app")
            .send_handler(Box::new(|m| m))
            .receive_handler(|msg| async move {
                match msg {
                    Message::Binary(_) => Ok(()),
                    _ => Err(SocketError::MPSCSend),
                }
            })
            .run()
            .await
            .unwrap();
        _ = ws.send(Message::Text("Hello".into()));
    }
}
