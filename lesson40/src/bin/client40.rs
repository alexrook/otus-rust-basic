use common::{
    bank::NonZeroMoney,
    protocol::{AccountRef, ClientRequest, ServerResponse},
};

use ftail::Ftail;

use std::{
    fmt::Debug,
    io,
    io::{ErrorKind, Read, Write},
    net::TcpStream,
};

use anyhow;

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

fn handle_connection<E>(stream: &mut TcpStream, requests: Vec<ClientRequest>) -> anyhow::Result<()>
where
    E: From<String> + Into<String> + Debug,
{
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

fn main() -> anyhow::Result<()> {
    Ftail::new().console(log::LevelFilter::max()).init()?; //trace

    if let Ok(mut stream) = TcpStream::connect("127.0.0.1:8080") {
        log::debug!("Connected to the server");
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

        handle_connection::<String>(&mut stream, commands)
    } else {
        let msg = "Couldn't connect to server";
        log::error!("{}", msg);
        Err(anyhow::Error::new(io::Error::new(
            ErrorKind::BrokenPipe,
            msg,
        )))
    }
}
