use common::bank::Account;
use common::bank::{Bank, InMemoryOpsStorage, InMemoryState, Operation, OpsStorage, State};
use common::protocol::{self, AccountRef, ClientRequest, ServerResponse};
use ftail::Ftail;
use std::io::{self, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

fn close(stream: &mut TcpStream) -> io::Result<()> {
    write_response(stream, &ServerResponse::Bye)
        .map_err(|e| io::Error::new(io::ErrorKind::BrokenPipe, e))?;
    stream.flush()?;
    stream.shutdown(std::net::Shutdown::Both)?;
    Ok(())
}

fn read_request(stream: &mut TcpStream) -> io::Result<ClientRequest> {
    protocol::read(stream)
}

fn write_response(stream: &mut TcpStream, response: &ServerResponse) -> io::Result<()> {
    protocol::write(stream, response)?;
    stream.flush()
}

fn handle_connection<T, S>(stream: &mut TcpStream, bank: Arc<Mutex<Bank<T, S>>>) -> io::Result<()>
where
    T: OpsStorage,
    S: State,
{
    loop {
        let client_request: ClientRequest = read_request(stream)?;
        log::debug!("Operation[{:?}] request received", client_request);
        let op = match client_request {
            ClientRequest::Quit => {
                log::debug!("Quit command received");
                break close(stream);
            }
            ClientRequest::Create(account_id) => Operation::Create(account_id),
            ClientRequest::Deposit(account_id, amount) => Operation::Deposit(account_id, amount),
            ClientRequest::GetBalance(account_id) => Operation::GetBalance(account_id),

            ClientRequest::Move { from, to, amount } => Operation::Move { from, to, amount },

            ClientRequest::Withdraw(accout_id, amount) => Operation::Withdraw(accout_id, amount),
        };

        let mut guard = bank
            .lock()
            .expect("It's called PoisonError, so I prefer to panic here");

        let response = bank_deal(&mut *guard, op);
        drop(guard);

        write_response(stream, &response)
            .map_err(|e| io::Error::new(io::ErrorKind::BrokenPipe, e))?;
    }
}

fn bank_deal<T, S>(bank: &mut Bank<T, S>, op: Operation) -> ServerResponse
where
    T: OpsStorage,
    S: State,
{
    fn map_ret(ret: Result<&Account, String>) -> Result<ServerResponse, String> {
        ret.map(|account| {
            ServerResponse::AccountState(AccountRef {
                account_id: account.account_id.clone(),
                balance: account.balance,
            })
        })
    }

    let ret = match op {
        Operation::Create(acc) => map_ret(bank.create_account(acc)),
        Operation::Deposit(acc, amount) => map_ret(bank.deposit(acc, amount)),
        Operation::Withdraw(acc, amount) => map_ret(bank.withdraw(acc, amount)),
        Operation::GetBalance(acc) => map_ret(bank.get_balance(&acc)),
        Operation::Move { from, to, amount } => {
            bank.move_money(from, to, amount).and_then(|mut accounts| {
                let from = accounts.next().ok_or(
                    "the operation did not return the required number of elements".to_owned(),
                )?;
                let to = accounts.next().ok_or(
                    "the operation did not return the required number of elements".to_owned(),
                )?;
                Ok(ServerResponse::FundsMovement {
                    from: AccountRef {
                        account_id: from.account_id.clone(),
                        balance: from.balance,
                    },
                    to: AccountRef {
                        account_id: to.account_id.clone(),
                        balance: to.balance,
                    },
                    amount,
                })
            })
        }
    };
    ret.unwrap_or_else(|e| ServerResponse::Error { message: e })
}

fn main() -> io::Result<()> {
    Ftail::new()
        .console(log::LevelFilter::max())
        .init()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let server = TcpListener::bind("127.0.0.1:8080")?;

    log::debug!("Server is listening on 127.0.0.1:8080");

    let bank: Arc<Mutex<Bank<InMemoryOpsStorage, InMemoryState>>> = Arc::new(Mutex::new(
        Bank::new(InMemoryOpsStorage::default(), InMemoryState::default()),
    ));

    for conn_result in server.incoming() {
        match conn_result {
            Ok(mut stream) => {
                let peer_addr = stream.peer_addr()?;
                log::debug!("Incoming connection from[{:?}]", peer_addr);
                let cloned_arc_bank = Arc::clone(&bank);
                let _ = thread::spawn(move || {
                    match handle_connection::<InMemoryOpsStorage, InMemoryState>(
                        &mut stream,
                        cloned_arc_bank,
                    ) {
                        Ok(()) => log::debug!("Connection closed"),
                        Err(err) => {
                            log::error!("An error[{:?}] occured while connection handling", err)
                        }
                    }
                });
            }
            Err(e) => {
                log::error!("An error[{}] occurred while receiving tcp connection", e);
            }
        }
    }

    Ok(())
}
