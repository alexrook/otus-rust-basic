use std::{net::SocketAddr, sync::Arc};

use common::{
    bank::{Bank, BankError, InMemoryOpsStorage, InMemoryState, OpsStorage, State},
    protocol::{AccountRef, ClientRequest, ServerResponse},
};
use serde::Serialize;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufStream},
    net::{TcpListener, TcpStream},
    sync::RwLock,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let bank = Bank::new(InMemoryOpsStorage::default(), InMemoryState::default());

    let state = Arc::new(RwLock::new(bank));

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    loop {
        let (stream, addr) = listener.accept().await?;
        let bank_ref = Arc::clone(&state);

        tokio::spawn(async move {
            match client_loop(stream, addr, bank_ref).await {
                Ok(()) => println!("[{addr}] Client disconnected"),
                Err(err) => println!("[{addr}] Client error: {err}"),
            }
        });
    }
}

async fn write_response<T>(stream: &mut BufStream<TcpStream>, message: T) -> anyhow::Result<()>
where
    T: Serialize,
{
    let encoded: Vec<u8> = bincode::serialize(&message).unwrap();
    let len = (encoded.len() as u32).to_be_bytes(); // Записываем размер в big-endian
    stream.write_all(&len).await?; // Отправляем 4 байта длины
    stream.write_all(&encoded).await?; // Отправляем само сообщение
    Ok(())
}

async fn client_loop<T: OpsStorage, S: State>(
    stream: TcpStream,
    addr: SocketAddr,
    bank_ref: Arc<RwLock<Bank<T, S>>>,
) -> anyhow::Result<()> {
    let mut stream = BufStream::new(stream);

    loop {
        let mut size_buf = [0u8; 4]; // Буфер для длины
        stream.read_exact(&mut size_buf).await?; // Читаем ровно 4 байта
        let size = u32::from_be_bytes(size_buf) as usize; // Получаем размер пакета

        let mut data_buf = vec![0; size];
        stream.read_exact(&mut data_buf).await?; // Читаем ровно `size` байтов

        let client_request = ClientRequest::deserialize(&data_buf)?;

        match client_request {
            ClientRequest::Create(account_id) => {
                let mut guard = bank_ref.write().await;
                let maybe_ret = guard.create_account(&account_id);

                let response = match maybe_ret {
                    Ok(acc) => ServerResponse::AccountState(AccountRef {
                        account_id: acc.account_id.clone(),
                        balance: acc.balance,
                    }),

                    Err(bank_err) => ServerResponse::Error {
                        message: bank_err.to_string(),
                    },
                };

                drop(guard);
                write_response(&mut stream, response).await?;
            }
            ClientRequest::Deposit(account_id, amount) => {
                let mut guard = bank_ref.write().await;
                let maybe_ret = guard.deposit(&account_id, amount);
                let response: ServerResponse = match maybe_ret {
                    Ok(acc) => ServerResponse::AccountState(AccountRef {
                        account_id,
                        balance: acc.balance,
                    }),
                    Err(bank_err) => ServerResponse::Error {
                        message: bank_err.to_string(),
                    },
                };
                drop(guard);
                write_response(&mut stream, response).await?;
            }
            ClientRequest::Withdraw(account_id, amount) => {
                let mut guard = bank_ref.write().await;
                let maybe_ret = guard.withdraw(&account_id, amount);
                let response: ServerResponse = match maybe_ret {
                    Ok(acc) => ServerResponse::AccountState(AccountRef {
                        account_id,
                        balance: acc.balance,
                    }),
                    Err(bank_err) => ServerResponse::Error {
                        message: bank_err.to_string(),
                    },
                };
                drop(guard);
                write_response(&mut stream, response).await?;
            }
            ClientRequest::GetBalance(account_id) => {
                let guard = bank_ref.read().await;
                let maybe_ret = guard.get_balance(&account_id);
                let response: ServerResponse = match maybe_ret {
                    Ok(acc) => ServerResponse::AccountState(AccountRef {
                        account_id,
                        balance: acc.balance,
                    }),
                    Err(bank_err) => ServerResponse::Error {
                        message: bank_err.to_string(),
                    },
                };
                drop(guard);
                write_response(&mut stream, response).await?;
            }
            ClientRequest::Move { from, to, amount } => {
                let mut guard = bank_ref.write().await;
                let maybe_ret = guard.move_money(&from, &to, amount).and_then(|mut iter| {
                    let from = iter.next().ok_or(BankError::CoreError(
                        "the operation did not return the required number of elements".to_owned(),
                    ))?;
                    let to = iter.next().ok_or(BankError::CoreError(
                        "the operation did not return the required number of elements".to_owned(),
                    ))?;
                    Ok((from, to))
                });

                let response: ServerResponse = match maybe_ret {
                    Ok((from, to)) => ServerResponse::FundsMovement {
                        from: AccountRef {
                            account_id: from.account_id.clone(),
                            balance: from.balance,
                        },
                        to: AccountRef {
                            account_id: to.account_id.clone(),
                            balance: to.balance,
                        },
                    },
                    Err(bank_err) => ServerResponse::Error {
                        message: bank_err.to_string(),
                    },
                };

                drop(guard);
                write_response(&mut stream, response).await?;
            }
            ClientRequest::Quit => {
                break;
            }
        }
    }

    Ok(())
}
