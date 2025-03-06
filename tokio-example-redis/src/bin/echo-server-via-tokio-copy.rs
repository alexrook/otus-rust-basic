use tokio::net::TcpListener;
use tokio::{io, stream};

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6142").await?;

    loop {
        let (mut stream, _) = listener.accept().await?;

        tokio::spawn(async move {
            /*
             * TcpStream::split takes a reference to the stream and returns a reader and writer handle.
             * Because a reference is used,
             * both handles must stay on the same task that split() was called from.
             * This specialized split is zero-cost.
             * There is no Arc or Mutex needed.
             * TcpStream also provides into_split which supports handles that can move across tasks at
             * the cost of only an Arc.
             */
            let (mut rd, mut wr) = stream.split();

            if io::copy(&mut rd, &mut wr).await.is_err() {
                eprintln!("failed to copy");
            }
        });
    }
}
