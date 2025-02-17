use common::core::{Bank, InMemoryOpsStorage, InMemoryState, Operation, OpsStorage, State};
use common::protocol::{read_proto, write_proto};
use common::{core::Account, protocol::Protocol};
use std::fmt::Debug;
use std::io::{self, Error, ErrorKind, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

fn close<'a, E>(stream: &'a mut TcpStream) -> io::Result<()>
where
    E: From<String> + Into<String> + Debug,
{
    write_proto::<E>(stream, &Protocol::Quit)
        .map_err(|e| Error::new(ErrorKind::BrokenPipe, e.into()))?;
    stream.flush()?;
    stream.shutdown(std::net::Shutdown::Both)?;
    Ok(())
}

fn handle_connection<'a, 'b, T, S, E>(
    stream: &'b mut TcpStream,
    bank: Arc<Mutex<Bank<T, S>>>,
) -> io::Result<()>
where
    T: OpsStorage,
    S: State,
    E: From<String> + Into<String> + Debug,
{
    loop {
        let message =
            read_proto::<E>(stream).map_err(|e| Error::new(ErrorKind::BrokenPipe, e.into()))?;

        match message {
            Protocol::Quit => {
                println!("Quit command received");
                break close::<E>(stream);
            }
            Protocol::Response(_) => {
                eprintln!("Unexpected client behavior, it should only send Request and Quit protocol commands");
                break close::<E>(stream);
            }
            Protocol::Request(op) => {
                println!("Operation[{:?}] request received", op);
                let mut guard = bank
                    .lock()
                    .expect("It's called PoisonError, so I prefer to panic here");

                let response = match bank_deal(&mut *guard, op) {
                    Ok(bank_accs) => {
                        let cloned =
                            Vec::from_iter(bank_accs.into_iter().map(|account| account.clone()));
                        Protocol::Response(Ok(cloned))
                    }
                    Err(bank_err) => {
                        eprintln!("An error[{:?}] occurred while bank dealing", bank_err);
                        Protocol::Response(Err(bank_err))
                    }
                };

                drop(guard);

                write_proto::<E>(stream, &response)
                    .map_err(|e| Error::new(ErrorKind::BrokenPipe, e.into()))?;
            }
        }
    }
}

fn bank_deal<'a, T, S>(bank: &'a mut Bank<T, S>, op: Operation) -> Result<Vec<&'a Account>, String>
where
    T: OpsStorage,
    S: State,
{
    fn map_ret<'a>(ret: Result<&'a Account, String>) -> Result<Vec<&'a Account>, String> {
        ret.map(|account| vec![account])
    }

    match op {
        Operation::Create(acc) => map_ret(bank.create_account(acc)),
        Operation::Deposit(acc, amount) => map_ret(bank.deposit(acc, amount)),
        Operation::Withdraw(acc, amount) => map_ret(bank.withdraw(acc, amount)),
        Operation::GetBalance(acc) => map_ret(bank.get_balance(&acc)),
        Operation::Move { from, to, amount } => bank
            .move_money(from, to, amount)
            .map(|iter| Vec::from_iter(iter)),
    }
}

fn main() -> io::Result<()> {
    let server = TcpListener::bind("127.0.0.1:8080")?;

    println!("Server is listening on 127.0.0.1:8080");

    let bank: Arc<Mutex<Bank<InMemoryOpsStorage, InMemoryState>>> = Arc::new(Mutex::new(
        Bank::new(InMemoryOpsStorage::default(), InMemoryState::default()),
    ));

    for conn_result in server.incoming() {
        match conn_result {
            Ok(mut stream) => {
                let peer_addr = stream.peer_addr()?;
                println!("Incoming connection from[{:?}]", peer_addr);
                let cloned_arc_bank = Arc::clone(&bank);
                let _ = thread::spawn(move || {
                    match handle_connection::<InMemoryOpsStorage, InMemoryState, String>(
                        &mut stream,
                        cloned_arc_bank,
                    ) {
                        Ok(()) => println!("Connection closed"),
                        Err(err) => {
                            eprintln!("An error[{:?}] occured while connection handling", err)
                        }
                    }
                });
            }
            Err(e) => {
                eprintln!("An error[{}] occurred while receiving tcp connection", e);
            }
        }
    }

    Ok(())
}
