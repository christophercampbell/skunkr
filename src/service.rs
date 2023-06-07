
use std::pin::Pin;
use tonic::{Request, Response, Status};
use tokio::sync::{mpsc, oneshot};
use tonic::codegen::futures_core::Stream;
use tokio_stream::wrappers::UnboundedReceiverStream;

use super::api::{GetRequest,GetResponse,SetRequest,SetResponse,ScanRequest,ScanResponse};
use super::api::data_server::Data;

use super::store::*;

pub struct Service {
    pub store: Box<dyn KeyValueStore + Send + Sync>
}

impl Service {
    pub fn new(path: &str) -> Self {
        Service {
            store: Box::new(MdbxStore::new(path))
        }
    }
}

#[tonic::async_trait]
impl Data for Service {
    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        let r = request.into_inner();
        match &self.store.get(r.key) {
            Some(value) => Ok(Response::new(GetResponse {
                value: value.to_vec()
            })),
            None => Ok(Response::new(GetResponse{ // TODO: error responses
                value: Vec::new()
            })),
        }
    }

    async fn set(&self, request: Request<SetRequest>) -> Result<Response<SetResponse>, Status> {
        let r = request.into_inner();
        let inserted = &self.store.set(r.key, r.value);
        Ok(Response::new(SetResponse {
            success: *inserted
        }))
    }

    type ScanStream = Pin<Box<dyn Stream<Item = Result<ScanResponse, Status>> + Send>>;

    async fn scan(&self, request: Request<ScanRequest>) -> Result<Response<Self::ScanStream>, Status> {

        let req = request.into_inner();

        let from = match req.from.len() {
            0 => None,
            _ => Some(req.from)
        };

        // command channel
        let (start, accept) = oneshot::channel();
        start.send(from).expect("failed to send filter command");

        // source data channel
        let (source, mut buffer) = mpsc::channel(10);

        // start the scanner
        let _ = &self.store.scan(accept, source);

        // outbound channel
        let (outbound, consumer) = mpsc::unbounded_channel();

        // receive, transform, and relay the rows
        tokio::spawn(async move {
            loop {
                // buffer blocks when consumer falls behind, giving
                // back pressure to data source, avoiding too much memory pressure
                let _ = match buffer.recv().await {
                    Some(kv) => outbound.send(Ok(ScanResponse { key: kv.0, value: kv.1 })),
                    None => break
                };
            }
        });

        let stream = UnboundedReceiverStream::new(consumer);
        Ok(Response::new(Box::pin(stream) as Self::ScanStream))
    }
}
