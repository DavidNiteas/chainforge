//! Noise XX 模式加密传输层。

use std::io;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::message::Message;

/// Noise XX 模式参数。
const NOISE_PARAMS: &str = "Noise_XX_25519_ChaChaPoly_BLAKE2s";

/// 加密后的网络流。
pub struct NoiseStream {
    stream: TcpStream,
    session: snow::TransportState,
}

impl NoiseStream {
    /// 发送定长帧（4 字节 length prefix + payload）。
    pub async fn send(&mut self, payload: &[u8]) -> io::Result<()> {
        let len = payload.len() as u32;
        self.stream.write_all(&len.to_be_bytes()).await?;

        let mut ciphertext = vec![0u8; payload.len() + 16]; // tag overhead
        self.session
            .write_message(payload, &mut ciphertext)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        self.stream.write_all(&ciphertext).await?;
        self.stream.flush().await?;
        Ok(())
    }

    /// 接收定长帧。
    pub async fn recv(&mut self) -> io::Result<Vec<u8>> {
        let mut len_buf = [0u8; 4];
        self.stream.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;

        let mut ciphertext = vec![0u8; len + 16];
        self.stream.read_exact(&mut ciphertext).await?;

        let mut plaintext = vec![0u8; len];
        self.session
            .read_message(&ciphertext, &mut plaintext)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(plaintext)
    }

    /// 发送结构化消息。
    pub async fn send_message(&mut self, msg: &Message) -> io::Result<()> {
        let encoded = msg.encode();
        self.send(&encoded).await
    }

    /// 接收结构化消息。
    pub async fn recv_message(&mut self) -> io::Result<Message> {
        let bytes = self.recv().await?;
        Message::decode(&bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

/// Noise 传输工厂。
pub struct NoiseTransport {
    static_key: [u8; 32],
}

impl NoiseTransport {
    pub fn new(static_key: [u8; 32]) -> Self {
        NoiseTransport { static_key }
    }

    /// 作为发起者（dial）连接到远程节点，完成 XX 握手。
    pub async fn dial(&self, addr: std::net::SocketAddr) -> io::Result<NoiseStream> {
        let mut stream = TcpStream::connect(addr).await?;
        let mut buf = vec![0u8; 65535];

        let mut handshake = snow::Builder::new(NOISE_PARAMS.parse().unwrap())
            .local_private_key(&self.static_key)
            .build_initiator()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

        // -> e
        let len = handshake.write_message(&[], &mut buf).unwrap();
        send_frame(&mut stream, &buf[..len]).await?;

        // <- e, ee, s, es
        let reply = recv_frame(&mut stream).await?;
        handshake.read_message(&reply, &mut buf).unwrap();

        // -> s, se
        let len = handshake.write_message(&[], &mut buf).unwrap();
        send_frame(&mut stream, &buf[..len]).await?;

        let session = handshake
            .into_transport_mode()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(NoiseStream { stream, session })
    }

    /// 作为响应者（listen）接受连接，完成 XX 握手。
    pub async fn accept(&self, listener: &TcpListener) -> io::Result<NoiseStream> {
        let (mut stream, _) = listener.accept().await?;
        let mut buf = vec![0u8; 65535];

        let mut handshake = snow::Builder::new(NOISE_PARAMS.parse().unwrap())
            .local_private_key(&self.static_key)
            .build_responder()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

        // <- e
        let msg = recv_frame(&mut stream).await?;
        handshake.read_message(&msg, &mut buf).unwrap();

        // -> e, ee, s, es
        let len = handshake.write_message(&[], &mut buf).unwrap();
        send_frame(&mut stream, &buf[..len]).await?;

        // <- s, se
        let msg = recv_frame(&mut stream).await?;
        handshake.read_message(&msg, &mut buf).unwrap();

        let session = handshake
            .into_transport_mode()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(NoiseStream { stream, session })
    }
}

/// 发送定长帧（用于握手阶段明文传输）。
async fn send_frame(stream: &mut TcpStream, data: &[u8]) -> io::Result<()> {
    let len = data.len() as u32;
    stream.write_all(&len.to_be_bytes()).await?;
    stream.write_all(data).await?;
    stream.flush().await?;
    Ok(())
}

/// 接收定长帧（用于握手阶段明文传输）。
async fn recv_frame(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn test_noise_handshake() {
        let key_a = [1u8; 32];
        let key_b = [2u8; 32];

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let accept_task = tokio::spawn(async move {
            let transport = NoiseTransport::new(key_b);
            transport.accept(&listener).await.unwrap()
        });

        let transport = NoiseTransport::new(key_a);
        let mut client = transport.dial(addr).await.unwrap();
        let mut server = accept_task.await.unwrap();

        // 测试加密通信
        client.send(b"hello from client").await.unwrap();
        let received = server.recv().await.unwrap();
        assert_eq!(&received, b"hello from client");

        server.send(b"hello from server").await.unwrap();
        let received = client.recv().await.unwrap();
        assert_eq!(&received, b"hello from server");
    }

    #[tokio::test]
    async fn test_message_roundtrip() {
        use crate::message::Message;

        let key_a = [3u8; 32];
        let key_b = [4u8; 32];

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let accept_task = tokio::spawn(async move {
            let transport = NoiseTransport::new(key_b);
            transport.accept(&listener).await.unwrap()
        });

        let transport = NoiseTransport::new(key_a);
        let mut client = transport.dial(addr).await.unwrap();
        let mut server = accept_task.await.unwrap();

        let msg = Message::BlockRequest { from: 1, to: 100 };
        client.send_message(&msg).await.unwrap();
        let received = server.recv_message().await.unwrap();
        assert_eq!(received, msg);

        let response = Message::BlockResponse(vec![vec![1, 2, 3], vec![4, 5, 6]]);
        server.send_message(&response).await.unwrap();
        let received = client.recv_message().await.unwrap();
        assert_eq!(received, response);
    }
}
