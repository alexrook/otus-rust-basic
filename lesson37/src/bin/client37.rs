use common::bank::NonZeroMoney;
use common::protocol::*;
use ftail::Ftail;
use std::fmt::Debug;
use std::io::Read;
use std::{
    io,
    io::{Error, ErrorKind, Write},
    net::TcpStream,
    thread,
};

fn close(stream: &mut TcpStream) -> io::Result<()> {
    stream.shutdown(std::net::Shutdown::Both)?;
    Ok(())
}

fn write_request(stream: &mut TcpStream, req: &ClientRequest) -> io::Result<()> {
    let encoded: Vec<u8> = req
        .serialize()
        .map_err(|e| Error::new(ErrorKind::BrokenPipe, e))?;
    let len = (encoded.len() as u32).to_be_bytes();
    stream.write_all(&len)?;
    stream.write_all(&encoded)?;
    Ok(())
}

fn read_response(stream: &mut TcpStream) -> io::Result<ServerResponse> {
    let mut size_buf = [0u8; 4]; // Буфер для длины
    stream.read_exact(&mut size_buf)?;
    let size = u32::from_be_bytes(size_buf) as usize; // Получаем размер пакета

    let mut data_buf = vec![0; size];
    stream.read_exact(&mut data_buf)?;

    ServerResponse::deserialize(&data_buf).map_err(|e| Error::new(ErrorKind::BrokenPipe, e))
}

fn handle_connection<E>(stream: &mut TcpStream, requests: &Vec<ClientRequest>) -> io::Result<()>
where
    E: From<String> + Into<String> + Debug,
{
    let client_port = stream.local_addr().unwrap().port();
    log::info!("handling connection from[{}] to the server", client_port);
    for req in requests {
        log::info!("Sending from[{}] request[{:?}] to server", client_port, req);

        write_request(stream, req)?;

        let response: ServerResponse = read_response(stream)?;

        match response {

            ServerResponse::AccountState(account_ref) => log::info!("Client[{client_port}]:The server returned an account[{account_ref}]"),

            ServerResponse::FundsMovement { from, to, amount } => log::info!(
                "Client[{client_port}]:The server moved money[{amount}] successfully from[{from}] to[{to}]"
            ),

            ServerResponse::Error { message }=>
                log::error!(
                    "Client[{client_port}]:An error[{message}] occurred while server negotiation with client"
                ),
            ServerResponse::Bye=>{
                    log::debug!("Client[{client_port}]:The server said goodbye");
                    close(stream)?;
                }
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    Ftail::new()
        .console(log::LevelFilter::max())
        .init()
        .map_err(|e| Error::new(ErrorKind::Other, e))?;

    let commands = vec![
        ClientRequest::Create("acc1".to_string()),
        ClientRequest::GetBalance("acc1".to_string()),
        ClientRequest::Deposit("acc1".to_string(), NonZeroMoney::new(42).unwrap()),
        ClientRequest::Withdraw("acc1".to_string(), NonZeroMoney::new(12).unwrap()),
        ClientRequest::GetBalance("acc1".to_string()),
        ClientRequest::Create("acc2".to_string()),
        ClientRequest::Move {
            from: "acc1".to_string(),
            to: "acc2".to_string(),
            amount: NonZeroMoney::new(12).unwrap(),
        },
        ClientRequest::GetBalance("acc1".to_string()),
        ClientRequest::GetBalance("acc2".to_string()),
        ClientRequest::Quit,
    ];

    thread::scope(|s| {
        fn run(commands: &Vec<ClientRequest>) {
            match run_client(commands) {
                Ok(_) => log::debug!("Run client finished"),
                Err(e) => log::error!("Run client error[{}]", e),
            };
        }

        s.spawn(|| {
            run(&commands);
        });
        s.spawn(|| {
            run(&commands);
        });
        s.spawn(|| {
            run(&commands);
        });
    });

    Ok(())
}

fn run_client(commands: &Vec<ClientRequest>) -> io::Result<()> {
    if let Ok(mut stream) = TcpStream::connect("127.0.0.1:8080") {
        handle_connection::<String>(&mut stream, commands)
    } else {
        let msg = "Couldn't connect to server";
        log::error!("{}", msg);
        Err(Error::new(ErrorKind::BrokenPipe, msg))
    }
}
