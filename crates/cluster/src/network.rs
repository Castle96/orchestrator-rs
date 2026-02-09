use crate::error::ClusterError;
use anyhow::Result;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::info;

pub struct ClusterNetwork {
    local_address: SocketAddr,
}

impl ClusterNetwork {
    pub fn new(local_address: SocketAddr) -> Self {
        Self { local_address }
    }

    pub async fn connect_to_node(&self, address: SocketAddr) -> Result<TcpStream, ClusterError> {
        info!("Connecting to cluster node at {}", address);

        TcpStream::connect(address)
            .await
            .map_err(|e| ClusterError::Network(format!("Failed to connect: {}", e)))
    }

    pub async fn send_message(
        &self,
        stream: &mut TcpStream,
        message: &[u8],
    ) -> Result<(), ClusterError> {
        let len = message.len() as u32;
        stream
            .write_u32(len)
            .await
            .map_err(|e| ClusterError::Network(format!("Failed to write length: {}", e)))?;
        stream
            .write_all(message)
            .await
            .map_err(|e| ClusterError::Network(format!("Failed to write message: {}", e)))?;
        Ok(())
    }

    pub async fn receive_message(&self, stream: &mut TcpStream) -> Result<Vec<u8>, ClusterError> {
        let len = stream
            .read_u32()
            .await
            .map_err(|e| ClusterError::Network(format!("Failed to read length: {}", e)))?
            as usize;

        let mut buffer = vec![0u8; len];
        stream
            .read_exact(&mut buffer)
            .await
            .map_err(|e| ClusterError::Network(format!("Failed to read message: {}", e)))?;

        Ok(buffer)
    }

    pub fn local_address(&self) -> SocketAddr {
        self.local_address
    }
}
