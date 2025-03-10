use common::bank::NonZeroMoney;
use common::protocol::{self, *};
use ftail::Ftail;

use std::{
    io::{self, Write},
    net::TcpStream,
    thread,
};

fn close(stream: &mut TcpStream) -> io::Result<()> {
    stream.shutdown(std::net::Shutdown::Both)?;
    Ok(())
}

fn write_request(stream: &mut TcpStream, request: &ClientRequest) -> io::Result<()> {
    protocol::write(stream, request)?;
    stream.flush()
}

fn read_response(stream: &mut TcpStream) -> io::Result<ServerResponse> {
    protocol::read(stream)
}

fn handle_connection(stream: &mut TcpStream, requests: &Vec<ClientRequest>) -> io::Result<()> {
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
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

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
        handle_connection(&mut stream, commands)
    } else {
        let msg = "Couldn't connect to server";
        log::error!("{}", msg);
        Err(io::Error::new(io::ErrorKind::BrokenPipe, msg))
    }
}
