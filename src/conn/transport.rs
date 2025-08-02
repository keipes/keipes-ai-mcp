use axum::http::request;
use bytes::{BufMut, Bytes, BytesMut};
use http_body_util::{Empty, Full};
use hyper::Request;
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::{client::legacy::Client, rt::TokioExecutor};
use parking_lot::RwLock;
use parking_lot::{lock_api::Mutex, RawFairMutex};
use reqwest::Error;
use rmcp::service::{RxJsonRpcMessage, TxJsonRpcMessage};
use rmcp::{transport::Transport, RoleClient};
use std::{
    prelude::rust_2024::Future,
    sync::{atomic::AtomicPtr, Arc, OnceLock},
};

struct NexusTransport {
    client: Client<HttpsConnector<HttpConnector>, Full<Bytes>>,
}

impl NexusTransport {
    pub fn new() -> Self {
        let http_connector = HttpConnector::new();
        let https = HttpsConnector::new();
        let client = Client::builder(TokioExecutor::new()).build::<_, Full<Bytes>>(https);
        NexusTransport { client }
    }
}

// static BUFFER_POOL: OnceLock<BytesMut> = OnceLock::new();

struct BufferPool {
    buffers: Vec<BytesMut>,
    max_buffer_len_before_reap: usize,
    initial_capacity: usize,
    max_buffers: usize,
    buffer_lock: RwLock<Vec<Arc<BytesMut>>>,
    locked_buffers: Vec<Mutex<RawFairMutex, BytesMut>>,
}

trait BufferPoolTrait {
    fn get_buffer(&mut self) -> Option<BytesMut>;
    fn return_buffer(&mut self, buffer: BytesMut);
}

impl BufferPoolTrait for BufferPool {
    fn get_buffer(&mut self) -> Option<BytesMut> {
        let candidate = self.get_buffer_inner();

        // vs
        let mut buffers = self.buffer_lock.lock();
        return buffers
            .iter()
            .find(|b| Arc::strong_count(b) < 2)
            .map(|b| {
                let mut buffer = b.as_ref().clone();
                buffer.clear();
                buffer
            })
            .or_else(|| {
                if buffers.len() < self.max_buffers {
                    let new_buffer = BytesMut::with_capacity(self.initial_capacity);
                    buffers.push(Arc::new(new_buffer));
                    Some(BytesMut::with_capacity(self.initial_capacity))
                } else {
                    None
                }
            });
    }

    fn return_buffer(&mut self, mut buffer: BytesMut) {
        // let pointer = AtomicPtr::new(buffer.as_mut_ptr());
        // let shared_bytes = Arc<BytesMut>::from(buffer);
        // Arc::strong_count(&shared_bytes);

        if self.buffers.len() < self.max_buffers {
            self.buffers.push(buffer);
        }
    }
}

impl BufferPool {
    fn new(initial_capacity: usize, max_bytes_before_reap: usize, max_buffers: usize) -> Self {
        let buffers = Vec::with_capacity(initial_capacity);
        BufferPool {
            buffers,
            max_bytes_before_reap,
            initial_capacity,
        }
    }

    fn get_buffer_inner(&mut self) -> Option<BytesMut> {
        self.locked_buffers.iter().find(|b| b.try_lock()).map(|b| {
            let mut buffer = b.lock();
            buffer.clear();
            buffer.clone()
        })
        // let buffers = self.buffer_lock.read();
        // let candidate = buffers.iter().find(|b| Arc::strong_count(b) < 2);
        // match candidate {
        //     Some(buffer) => {}
        //     None => None,
        // }

        // buffers.iter().find(|b| Arc::strong_count(b) < 2).map(|b| {
        //     let mut buffer = b.as_ref().clone();
        //     buffer.clear();
        //     buffer
        // })
    }
}

impl Transport<RoleClient> for NexusTransport {
    type Error = Error;
    // fn send(
    //     &mut self,
    //     item: TxJsonRpcMessage<R>,
    // ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'static;

    fn send(
        &mut self,
        item: TxJsonRpcMessage<RoleClient>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'static {
        let (tx, rx): (Sender<i32>, Receiver<i32>) = mpsc::channel();

        async move {
            // Here you would implement the logic to send the message
            // For example, using a reqwest client to send an HTTP request
            let serialized = serde_json::to_string(&item).ok();
            let writer = std::io::stdout(); // Replace with actual writer
            let didWrite = serde_json::to_writer(writer, &item);

            let builder = Request::builder()
                .method("POST")
                .uri("http://example.com/api")
                .header("Content-Type", "application/json");
            let byteBuffer = BytesMut::new().writer();
            let request = builder.body(serialized.unwrap_or_default()).unwrap();
            self.client
                .request(request)
                .send()
                .await
                .map_err(|e| "error")?;
            Ok(())
        }
    }

    fn receive(&mut self) -> impl Future<Output = Option<RxJsonRpcMessage<RoleClient>>> + Send {
        async move {
            todo!();
            None
        }
    }

    fn close(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send {
        async move {
            todo!();
            Ok(())
        }
    }
}

pub fn get_transport() -> Result<NexusTransport, Error> {
    Ok(NexusTransport::new())
}
