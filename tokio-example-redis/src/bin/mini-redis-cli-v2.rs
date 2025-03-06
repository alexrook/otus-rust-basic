use std::{fmt::Debug, time::Duration};

use bytes::Bytes;
use mini_redis::client;
use tokio::{
    io,
    sync::{
        mpsc::{self, Receiver},
        oneshot,
    },
    task::JoinHandle,
    time::sleep,
};

extern crate tokio; // 0.2.13

type Sender<T> = oneshot::Sender<mini_redis::Result<T>>;
/// Multiple different commands are multiplexed over a single channel.
#[derive(Debug)]
enum Command {
    Get {
        key: String,
        responder: Sender<Option<Bytes>>,
    },
    Set {
        key: String,
        val: Bytes,
        responder: Sender<()>,
    },
}

/// Provided by the requester and used by the manager task to send
/// the command response back to the requester.
async fn client_manager(mut rx: Receiver<Command>) -> io::Result<JoinHandle<()>> {
    let mut client = client::connect("127.0.0.1:6379")
        .await
        .map_err(|err| io::Error::new(std::io::ErrorKind::ConnectionAborted, err))?;
    let handle = tokio::spawn(async move {
        // Establish a connection to the server

        // Start receiving messages
        while let Some(cmd) = rx.recv().await {
            use Command::*;

            match cmd {
                Get { key, responder } => {
                    let r = client.get(&key).await;
                    responder.send(r).expect("something wrong with responder");
                }
                Set {
                    key,
                    val,
                    responder,
                } => {
                    let r = client.set(&key, val).await;
                    responder.send(r).expect("something wrong with responder");
                }
            }
        }
    });

    Ok(handle)
}

#[tokio::main]
async fn main() -> io::Result<()> {
    fn out<T: Debug>(ret: Result<T, Box<dyn std::error::Error + Send + Sync>>) {
        match ret {
            Ok(p) => println!("Received result[{p:?}]"),
            Err(e) => eprintln!("An error[{e}] occurred"),
        }
    }

    let (tx, rx) = mpsc::channel(32);
    // The `move` keyword is used to **move** ownership of `rx` into the task.

    client_manager(rx).await?;

    let tx2 = tx.clone();

    // Spawn two tasks, one gets a key, the other sets a key
    let t1 = tokio::spawn(async move {
        let (sender, receiver) = oneshot::channel();
        let cmd = Command::Get {
            key: "foo".to_string(),
            responder: sender,
        };
        sleep(Duration::from_secs(1)).await;
        tx.send(cmd).await.unwrap();
        receiver.await.unwrap()
    });

    let t2 = tokio::spawn(async move {
        let (sender, receiver) = oneshot::channel();
        let cmd = Command::Set {
            key: "foo".to_owned(),
            val: Bytes::from_owner("bar"),
            responder: sender,
        };

        tx2.send(cmd).await.unwrap();
        receiver.await.unwrap()
    });

    let r2 = t2.await?;
    let r1 = t1.await?;
    out(r2);
    out(r1);

    Ok(())
}
