use core::core::{Bank, InMemoryOpsStorage, InMemoryState, Operation, OpsStorage, State};
use core::{core::Account, protocol::Protocol, protocol::IO};
use std::fmt::Debug;
use std::io::{Error, ErrorKind, Result as IOResult, Write};
use std::net::{TcpListener, TcpStream};

fn handle_connection<'a, 'b, T, S, E>(
    stream: &'b mut TcpStream,
    bank: &'a mut Bank<T, S>,
) -> IOResult<()>
where
    T: OpsStorage,
    S: State,
    E: From<String> + Into<String> + Debug,
{
    fn close<'a, 'b>(stream: &'b mut TcpStream) -> IOResult<()> {
        let _ = stream.flush()?;
        let _ = stream.shutdown(std::net::Shutdown::Both)?;
        Ok(())
    }

    let message = IO::read::<E>(stream).map_err(|e| Error::new(ErrorKind::BrokenPipe, e.into()))?;

    match message {
        Protocol::Quit => {
            println!("Quit command received");
            return close(stream);
        }
        Protocol::Response(_) => {
            eprintln!("Unexpected client behavior, it should only send Request and Quit protocol commands");
            return close(stream);
        }
        Protocol::Request(op) => {
            let bank_ret = bank_deal(bank, op)?;
            let cloned = Vec::from_iter(bank_ret.into_iter().map(|account| account.clone()));
            let response = Protocol::Response(Ok(cloned));
            IO::write::<E>(stream, &response)
                .map_err(|e| Error::new(ErrorKind::BrokenPipe, e.into()))?;
            handle_connection::<T, S, E>(stream, bank)
        }
    }
}

fn bank_deal<'a, T, S>(bank: &'a mut Bank<T, S>, op: Operation) -> IOResult<Vec<&'a Account>>
where
    T: OpsStorage,
    S: State,
{
    fn map_ret<'a>(ret: Result<&'a Account, String>) -> IOResult<Vec<&'a Account>> {
        ret.map(|account| vec![account])
            .map_err(|bank_err| std::io::Error::new(ErrorKind::Other, bank_err))
    }

    match op {
        Operation::Create(acc) => map_ret(bank.create_account(acc)),
        Operation::Deposit(acc, amount) => map_ret(bank.deposit(acc, amount)),
        Operation::Withdraw(acc, amount) => map_ret(bank.withdraw(acc, amount)),
        Operation::GetBalance(acc) => map_ret(bank.get_balance(&acc)),
        Operation::Move { from, to, amount } => bank
            .move_money(from, to, amount)
            .map(|iter| Vec::from_iter(iter))
            .map_err(|bank_err| std::io::Error::new(ErrorKind::Other, bank_err)),
    }
}

fn main() -> IOResult<()> {
    let server = TcpListener::bind("localhost:8080")?;

    println!("Server is listening on localhost:8080");

    let mut bank: Bank<InMemoryOpsStorage, InMemoryState> =
        Bank::new(InMemoryOpsStorage::default(), InMemoryState::default());

    for conn_result in server.incoming() {
        match conn_result {
            Ok(mut stream) => {
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
