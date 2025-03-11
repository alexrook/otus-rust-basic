use std::{
    fmt::{Debug, Display},
    io::{self, Read, Write},
};

use serde::{Deserialize, Serialize};

use crate::bank::{AccountId, Money, NonZeroMoney};

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ClientRequest {
    Create(AccountId),                 //регистрация счёта
    Deposit(AccountId, NonZeroMoney),  //пополнение
    Withdraw(AccountId, NonZeroMoney), //снятие
    Move {
        //перевод
        from: AccountId,
        to: AccountId,
        amount: NonZeroMoney,
    },
    GetBalance(AccountId), //получение баланса
    Quit,                  //завершение сеанса
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountRef {
    pub account_id: String,
    pub balance: Money,
}

impl Display for AccountRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Account with id[{}] and funds[{}]",
            self.account_id, self.balance
        )
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServerResponse {
    AccountState(AccountRef), //Create, Deposit, Withdraw, GetBalance ops response
    FundsMovement {
        //Move op response
        from: AccountRef,
        to: AccountRef,
        amount: NonZeroMoney,
    },
    Error {
        message: String,
    },
    Bye,
}

pub fn write<T: Serialize>(stream: &mut impl Write, entity: &T) -> io::Result<()> {
    let encoded: Vec<u8> =
        bincode::serialize(&entity).map_err(|e| io::Error::new(io::ErrorKind::BrokenPipe, e))?;
    let len = (encoded.len() as u32).to_be_bytes();
    stream.write_all(&len)?;
    stream.write_all(&encoded)?;
    Ok(())
}

pub fn read<T>(stream: &mut impl Read) -> io::Result<T>
where
    for<'a> T: Deserialize<'a>,
{
    let mut size_buf = [0u8; 4]; // Буфер для длины
    stream.read_exact(&mut size_buf)?;
    let size = u32::from_be_bytes(size_buf) as usize; // Получаем размер пакета

    let mut data_buf = vec![0; size];
    stream.read_exact(&mut data_buf)?;

    bincode::deserialize(&data_buf).map_err(|e| io::Error::new(io::ErrorKind::BrokenPipe, e))
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use super::ClientRequest;
    use crate::{
        bank::NonZeroMoney,
        protocol::{AccountRef, ServerResponse},
    };
    use serde::{Deserialize, Serialize};

    fn test_base<T>(message: T)
    where
        T: Serialize + Eq + Debug,
        for<'a> T: Deserialize<'a>,
    {
        let encoded: Vec<u8> = bincode::serialize(&message).unwrap();
        let actual: Result<T, bincode::Error> = bincode::deserialize(&encoded);
        assert!(actual.is_ok());
        assert_eq!(actual.unwrap(), message);
    }

    #[test]
    fn test_client_marshalling() {
        test_base(ClientRequest::Create("acc1".to_owned()));
        test_base(ClientRequest::Quit);
    }

    #[test]
    fn test_server_marshalling() {
        test_base(ServerResponse::FundsMovement {
            from: AccountRef {
                account_id: "acc1".to_owned(),
                balance: 120,
            },
            to: AccountRef {
                account_id: "acc2".to_owned(),
                balance: 42,
            },
            amount: NonZeroMoney::new(23).unwrap(),
        });

        test_base(ServerResponse::Error {
            message: "an error".to_owned(),
        });
    }
}
