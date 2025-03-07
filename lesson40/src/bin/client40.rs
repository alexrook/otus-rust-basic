use common::{
    bank::NonZeroMoney,
    protocol::{AccountRef, ClientRequest, ServerResponse},
};

use ftail::Ftail;

use std::{
    io,
    io::{ErrorKind, Read, Write},
    net::TcpStream,
};

fn write_request_sync(stream: &mut TcpStream, req: &ClientRequest) -> anyhow::Result<()> {
    let encoded: Vec<u8> = req.serialize()?;
    let len = (encoded.len() as u32).to_be_bytes();
    stream.write_all(&len)?;
    stream.write_all(&encoded)?;
    Ok(())
}

fn read_response_sync(stream: &mut TcpStream) -> anyhow::Result<ServerResponse> {
    let mut size_buf = [0u8; 4]; // Буфер для длины
    stream.read_exact(&mut size_buf)?;
    let size = u32::from_be_bytes(size_buf) as usize; // Получаем размер пакета

    let mut data_buf = vec![0; size];
    stream.read_exact(&mut data_buf)?;

    Ok(ServerResponse::deserialize(&data_buf)?)
}

fn handle_connection(stream: &mut TcpStream, requests: Vec<ClientRequest>) -> anyhow::Result<()> {
    for req in requests {
        log::info!("Sending request[{:?}] to server", req);
        write_request_sync(stream, &req)?;

        let response = read_response_sync(stream)?;

        match response {
            ServerResponse::AccountState(AccountRef {
                account_id,
                balance,
            }) => {
                log::info!("Account[{}] state changed{}", account_id, balance)
            }

            ServerResponse::FundsMovement { from, to } => {
                log::info!("Funds moved from[{}] to[{}]", from, to)
            }

            ServerResponse::Error { message } => {
                log::error!("An error[{}] occurred", message)
            }

            ServerResponse::Bye => {
                log::info!("the server said Goodbye");
                stream.shutdown(std::net::Shutdown::Both)?;
                break;
            }
        }
    }

    Ok(())
}

//Если запускать клиент несколько раз, можно увидеть (ожидаемые) бизнес-ошибки при создании аккаунтов.
fn main() -> anyhow::Result<()> {
    Ftail::new().console(log::LevelFilter::max()).init()?; //trace

    if let Ok(mut stream) = TcpStream::connect("127.0.0.1:8080") {
        log::debug!("Connected to the server");
        let commands = vec![
            ClientRequest::Create(128), //BE при повторных запусках
            ClientRequest::GetBalance(128),
            ClientRequest::Deposit(128, NonZeroMoney::new(42).unwrap()),
            ClientRequest::Withdraw(128, NonZeroMoney::new(12).unwrap()),
            ClientRequest::GetBalance(128),
            ClientRequest::Create(129), //BE при повторных запусках
            //должна быть ошибка в логе даже при первом запуске
            ClientRequest::Withdraw(128, NonZeroMoney::new(142).unwrap()),
            ClientRequest::Move {
                from: 128,
                to: 129,
                amount: NonZeroMoney::new(12).unwrap(),
            },
            ClientRequest::GetBalance(128),
            ClientRequest::GetBalance(129),
            ClientRequest::Quit,
        ];

        handle_connection(&mut stream, commands)
    } else {
        let msg = "Couldn't connect to server";
        log::error!("{}", msg);
        Err(anyhow::Error::new(io::Error::new(
            ErrorKind::BrokenPipe,
            msg,
        )))
    }
}
