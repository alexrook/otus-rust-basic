use common::core::{NonZeroMoney, Operation};
use common::protocol::*;
use std::fmt::Debug;
use std::{
    io,
    io::{Error, ErrorKind, Write},
    net::TcpStream,
    thread,
};

fn close<E>(stream: &mut TcpStream) -> io::Result<()>
where
    E: From<String> + Into<String> + Debug,
{
    write_proto::<E>(stream, &Protocol::Quit)
        .map_err(|e| Error::new(ErrorKind::BrokenPipe, e.into()))?;
    stream.flush()?;
    stream.shutdown(std::net::Shutdown::Both)?;
    Ok(())
}

fn handle_connection<E>(stream: &mut TcpStream, requests: &Vec<Protocol>) -> io::Result<()>
where
    E: From<String> + Into<String> + Debug,
{
    let client_port = stream.local_addr().unwrap().port();
    println!("handling connection from[{}] to the server", client_port);
    for req in requests {
        println!("Sending from[{}] request[{:?}] to server", client_port, req);
        write_proto::<E>(stream, req).map_err(|e| Error::new(ErrorKind::BrokenPipe, e.into()))?;

        let response =
            read_proto::<E>(stream).map_err(|e| Error::new(ErrorKind::BrokenPipe, e.into()))?;

        match response {
            Protocol::Quit => {
                println!("Quit command received for[{}]", client_port);
                return close::<E>(stream);
            }
            Protocol::Response(ret) => match ret {
                Ok(accs) => {
                    println!(
                        "The server returned a successful response.[{:?}] to[{}]",
                        accs, client_port
                    );
                }
                Err(err) => {
                    eprintln!(
                        "An error[{:?}] occurred while server negotiation with[{}]",
                        err, client_port
                    );
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

fn main() {
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

    thread::scope(|s| {
        fn run(commands: &Vec<Protocol>) {
            match run_client(commands) {
                Ok(_) => println!("Run client finished"),
                Err(e) => eprintln!("Run client error[{}]", e),
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
}

fn run_client(commands: &Vec<Protocol>) -> io::Result<()> {
    if let Ok(mut stream) = TcpStream::connect("127.0.0.1:8080") {
        handle_connection::<String>(&mut stream, commands)
    } else {
        let msg = "Couldn't connect to server";
        eprintln!("{}", msg);
        Err(Error::new(ErrorKind::BrokenPipe, msg))
    }
}
