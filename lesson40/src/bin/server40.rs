use std::{
    net::SocketAddr,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use common::{
    bank::{Account, Bank, BankError, InMemoryOpsStorage, InMemoryState, OpsStorage, State},
    protocol::{AccountRef, ClientRequest, ServerResponse},
};

use ftail::Ftail;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufStream},
    net::{TcpListener, TcpStream},
    sync::RwLock,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Ftail::new().console(log::LevelFilter::max()).init()?;

    let bank = Bank::new(InMemoryOpsStorage::default(), InMemoryState::default());

    let state = Arc::new(RwLock::new(bank));

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    log::debug!("is listening on localhost:8080");

    loop {
        let (stream, addr) = listener.accept().await?;
        log::debug!("New client connected");
        let bank_ref = Arc::clone(&state);

        tokio::spawn(async move {
            match client_loop(addr, stream, bank_ref).await {
                Ok(()) => log::debug!("[{addr}] Client disconnected"),
                Err(err) => log::error!("[{addr}] Client error: {err}"),
            }
        });
    }
}

async fn write_response(
    client_addr: SocketAddr,
    stream: &mut BufStream<TcpStream>,
    response: ServerResponse,
) -> anyhow::Result<()> {
    let msg = format!("Sending response[{:?}] to client[{client_addr}]", response);
    if let ServerResponse::Error { message: _ } = response {
        log::error!("{msg}");
    } else {
        log::info!("{msg}");
    }

    let encoded: Vec<u8> = response.serialize()?;
    let len = (encoded.len() as u32).to_be_bytes(); // Записываем размер в big-endian
    stream.write_all(&len).await?; // Отправляем 4 байта длины
    stream.write_all(&encoded).await?; // Отправляем само сообщение
    stream.flush().await?;
    Ok(())
}

fn process_request<T: OpsStorage, S: State, B>(
    client_request: ClientRequest,
    bank_ref: &mut B,
) -> Result<Option<ServerResponse>, BankError>
where
    B: DerefMut<Target = Bank<T, S>>,
{
    fn to_account_state(account: &Account) -> Option<ServerResponse> {
        Some(ServerResponse::AccountState(AccountRef {
            account_id: account.account_id,
            balance: account.balance,
        }))
    }
    match client_request {
        ClientRequest::Create(account_id) => {
            bank_ref.create_account(account_id).map(to_account_state)
        }

        ClientRequest::Deposit(account_id, amount) => {
            bank_ref.deposit(&account_id, amount).map(to_account_state)
        }

        ClientRequest::Withdraw(account_id, amount) => {
            bank_ref.withdraw(account_id, amount).map(to_account_state)
        }

        ClientRequest::GetBalance(account_id) => {
            bank_ref.get_balance(&account_id).map(to_account_state)
        }

        ClientRequest::Move { from, to, amount } => {
            bank_ref.move_money(from, to, amount).map(|(from, to)| {
                Some(ServerResponse::FundsMovement {
                    from: AccountRef {
                        account_id: from.account_id.clone(),
                        balance: from.balance,
                    },
                    to: AccountRef {
                        account_id: to.account_id.clone(),
                        balance: to.balance,
                    },
                })
            })
        }

        ClientRequest::Quit => Ok(None),
    }
}

async fn client_loop<T: OpsStorage, S: State>(
    client_addr: SocketAddr,
    stream: TcpStream,
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

        log::info!(
            "A client[{client_addr}] request[{:?}] recevied",
            client_request
        );

        let mut guard = bank_ref.write().await;

        let maybe_response = process_request(client_request, &mut guard);
        drop(guard);

        match maybe_response {
            //успешная операция в банке
            Ok(Some(response)) => write_response(client_addr, &mut stream, response).await?,
            //ошибочная операция в банке
            Err(bank_err) => {
                write_response(
                    client_addr,
                    &mut stream,
                    ServerResponse::Error {
                        message: bank_err.to_string(),
                    },
                )
                .await?
            }
            //клиент вышел из чата :-)
            Ok(None) => {
                log::info!("Client[{client_addr}] disconnection");
                break;
            }
        }
    }

    Ok(())
}
