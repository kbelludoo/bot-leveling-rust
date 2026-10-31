use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_tungstenite::{
    tungstenite::{client::IntoClientRequest, http::HeaderValue, Message},
    MaybeTlsStream, WebSocketStream,
};
use tracing::{error, warn};

pub type WsWriter = futures_util::stream::SplitSink<
    WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
    Message,
>;

pub struct LevelingSocketClient {
    pub url: String,
    pub writer: Arc<Mutex<Option<WsWriter>>>,
}

impl LevelingSocketClient {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            writer: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn connect<F>(&self, mut on_binary_message: F) -> Result<(), String>
    where
        F: FnMut(Vec<u8>) + Send + 'static,
    {
        let mut req = self
            .url
            .as_str()
            .into_client_request()
            .map_err(|e| format!("Invalid URL: {e}"))?;

        let headers = req.headers_mut();
        headers.insert(
            "User-Agent",
            HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
            ),
        );
        headers.insert(
            "Origin",
            HeaderValue::from_static("https://levelingidle2d.com.br"),
        );

        let (ws_stream, _) = tokio_tungstenite::connect_async(req)
            .await
            .map_err(|e| format!("WebSocket dial failed: {e}"))?;

        let (writer, mut reader) = ws_stream.split();
        *self.writer.lock().await = Some(writer);
        let writer_cleanup = Arc::clone(&self.writer);

        tokio::spawn(async move {
            while let Some(msg_result) = reader.next().await {
                match msg_result {
                    Ok(Message::Binary(bin)) => {
                        on_binary_message(bin);
                    }
                    Ok(Message::Close(_)) => {
                        warn!("[Leveling WS] Fechado pelo servidor");
                        break;
                    }
                    Err(e) => {
                        error!("[Leveling WS] Erro de leitura: {e}");
                        break;
                    }
                    _ => {}
                }
            }
            *writer_cleanup.lock().await = None;
        });

        Ok(())
    }

    pub async fn is_connected(&self) -> bool {
        self.writer.lock().await.is_some()
    }

    pub async fn send_binary(&self, data: Vec<u8>) -> Result<(), String> {
        let mut writer_lock = self.writer.lock().await;
        if let Some(w) = writer_lock.as_mut() {
            w.send(Message::Binary(data))
                .await
                .map_err(|e| format!("Falha de envio WS binário: {e}"))
        } else {
            Err("WebSocket desconectado".to_string())
        }
    }
}
