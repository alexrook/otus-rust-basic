use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufStream},
    net::{TcpListener, TcpStream},
    sync::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        RwLock,
    },
};

struct State {
    clients: Vec<Client>,
}

struct Client {
    addr: SocketAddr,
    sender: UnboundedSender<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:5555").await?;
    let state = Arc::new(RwLock::new(State { clients: vec![] }));

    loop {
        let (stream, addr) = listener.accept().await?;
        let state = Arc::clone(&state);

        let (sender, receiver) = mpsc::unbounded_channel();
        let client = Client { addr, sender };

        state.write().await.clients.push(client);

        tokio::spawn(async move {
            match client_loop(stream, addr, state.clone(), receiver).await {
                Ok(()) => println!("[{addr}] Client disconnected"),
                Err(err) => println!("[{addr}] Client error: {err}"),
            }
            state
                .write()
                .await
                .clients
                .retain(|client| client.addr != addr);
        });
    }
}

async fn client_loop(
    stream: TcpStream,
    addr: SocketAddr,
    state: Arc<RwLock<State>>,
    mut receiver: UnboundedReceiver<String>,
) -> Result<()> {
    let mut stream = BufStream::new(stream);

    loop {
        let mut buf = String::new();
        tokio::select!(
            Ok(n) = stream.read_line(&mut buf) => {
                if n == 0 {
                    break;
                }
                println!("[{addr}] Message: {buf}");

                for client in &state.read().await.clients {
                    if client.addr == addr {
                        continue;
                    }
                    client.sender.send(buf.clone())?;
                }
            }
            Some(msg) = receiver.recv() => {
                stream.write_all(b">> ").await?;
                stream.write_all(msg.as_bytes()).await?;
                stream.flush().await?;
            }
        );
    }

    Ok(())
}