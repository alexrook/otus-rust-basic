use common::core::{Bank, InMemoryOpsStorage, InMemoryState, Operation, OpsStorage, State};
use common::{core::Account, protocol::Protocol, protocol::IO};
use std::fmt::Debug;
use std::io::{Error, ErrorKind, Result as IOResult, Write};
use std::net::{TcpListener, TcpStream};

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

fn handle_connection<T, S, E>(
    stream: &mut TcpStream,
    bank: &mut Bank<T, S>,
) -> IOResult<()>
where
    T: OpsStorage,
    S: State,
    E: From<String> + Into<String> + Debug,
{
    let message = IO::read::<E>(stream).map_err(|e| Error::new(ErrorKind::BrokenPipe, e.into()))?;

    match message {
        Protocol::Quit => {
            println!("Quit command received");
            close::<E>(stream)
        }
        Protocol::Response(_) => {
            eprintln!("Unexpected client behavior, it should only send Request and Quit protocol commands");
            close::<E>(stream)
        }
        Protocol::Request(op) => {
            println!("Operation[{:?}] request received", op);

            let response = match bank_deal(bank, op) {
                Ok(bank_accs) => {
                    let cloned =
                        Vec::from_iter(bank_accs.into_iter().cloned());
                    Protocol::Response(Ok(cloned))
                }
                Err(bank_err) => {
                    eprintln!("An error[{:?}] occurred while bank dealing", bank_err);
                    Protocol::Response(Err(bank_err))
                }
            };

            IO::write::<E>(stream, &response)
                .map_err(|e| Error::new(ErrorKind::BrokenPipe, e.into()))?;
            handle_connection::<T, S, E>(stream, bank)
        }
    }
}

fn bank_deal<T, S>(bank: &mut Bank<T, S>, op: Operation) -> Result<Vec<&Account>, String>
where
    T: OpsStorage,
    S: State,
{
    fn map_ret(ret: Result<&Account, String>) -> Result<Vec<&Account>, String> {
        ret.map(|account| vec![account])
    }

    match op {
        Operation::Create(acc) => map_ret(bank.create_account(acc)),
        Operation::Deposit(acc, amount) => map_ret(bank.deposit(acc, amount)),
        Operation::Withdraw(acc, amount) => map_ret(bank.withdraw(acc, amount)),
        Operation::GetBalance(acc) => map_ret(bank.get_balance(&acc)),
        Operation::Move { from, to, amount } => bank
            .move_money(from, to, amount)
            .map(Vec::from_iter),
    }
}

fn main() -> IOResult<()> {
    let server = TcpListener::bind("127.0.0.1:8080")?;

    println!("Server is listening on 127.0.0.1:8080");

    let mut bank: Bank<InMemoryOpsStorage, InMemoryState> =
        Bank::new(InMemoryOpsStorage::default(), InMemoryState::default());

    for conn_result in server.incoming() {
        match conn_result {
            Ok(mut stream) => {
                let peer_addr = stream.peer_addr()?;
                println!("Incoming connection from[{:?}]", peer_addr);

                match handle_connection::<InMemoryOpsStorage, InMemoryState, String>(
                    &mut stream,
                    &mut bank,
                ) {
                    Ok(()) => println!("Connection closed"),
                    Err(err) => eprintln!("An error[{:?}] occured while connection handling", err),
                }
            }
            Err(e) => {
                eprintln!("An error[{}] occurred while receiving tcp connection", e);
            }
        }
    }

    Ok(())
}
