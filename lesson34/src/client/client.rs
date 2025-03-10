use common::core::{NonZeroMoney, Operation};
use common::protocol::*;
use std::fmt::Debug;
use std::{
    io::{Error, ErrorKind, Result as IOResult, Write},
    net::TcpStream,
};

fn close<E>(stream: &mut TcpStream) -> IOResult<()>
where
    E: From<String> + Into<String> + Debug,
{
    IO::write::<E>(stream, &Protocol::Quit)
        .map_err(|e| Error::new(ErrorKind::BrokenPipe, e.into()))?;
    stream.flush()?;
    stream.shutdown(std::net::Shutdown::Both)?;
    Ok(())
}

fn handle_connection<E>(stream: &mut TcpStream, requests: Vec<Protocol>) -> IOResult<()>
where
    E: From<String> + Into<String> + Debug,
{
    for req in requests {
        println!("Sending request[{:?}] to server", req);
        IO::write::<E>(stream, &req).map_err(|e| Error::new(ErrorKind::BrokenPipe, e.into()))?;

        let response =
            IO::read::<E>(stream).map_err(|e| Error::new(ErrorKind::BrokenPipe, e.into()))?;

        match response {
            Protocol::Quit => {
                println!("Quit command received");
                return close::<E>(stream);
            }
            Protocol::Response(ret) => match ret {
                Ok(accs) => {
                    println!("The server returned a successful response.[{:?}]", accs);
                }
                Err(err) => {
                    eprintln!("An error[{:?}] occurred while server negotiation", err);
                }
            },
            Protocol::Request(_) => {
                eprintln!("Unexpected server behavior, it should only send Response and Quit protocol commands");
                return close::<E>(stream);
            }
        }
    }

    Ok(())
}

fn main() -> IOResult<()> {
    if let Ok(mut stream) = TcpStream::connect("127.0.0.1:8080") {
        println!("Connected to the server");
        let commands = vec![
            Protocol::Request(Operation::Create("acc1".to_string())),
            Protocol::Request(Operation::GetBalance("acc1".to_string())),
            Protocol::Request(Operation::Deposit(
                "acc1".to_string(),
                NonZeroMoney::new(42).unwrap(),
            )),
            Protocol::Request(Operation::Withdraw(
                "acc1".to_string(),
                NonZeroMoney::new(12).unwrap(),
            )),
            Protocol::Request(Operation::GetBalance("acc1".to_string())),
            Protocol::Request(Operation::Create("acc2".to_string())),
            Protocol::Request(Operation::Move {
                from: "acc1".to_string(),
                to: "acc2".to_string(),
                amount: NonZeroMoney::new(12).unwrap(),
            }),
            Protocol::Request(Operation::GetBalance("acc1".to_string())),
            Protocol::Request(Operation::GetBalance("acc2".to_string())),
            Protocol::Quit,
        ];

        handle_connection::<String>(&mut stream, commands)
    } else {
        let msg = "Couldn't connect to server";
        eprintln!("{}", msg);
        Err(Error::new(ErrorKind::BrokenPipe, msg))
    }
}
