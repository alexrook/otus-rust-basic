use std::{fmt::Debug, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::core::{AccountId, Money, NonZeroMoney};

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

impl ClientRequest {
    pub fn serialize(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    pub fn deserialize(encoded: &[u8]) -> Result<ClientRequest, bincode::Error> {
        bincode::deserialize(&encoded)
    }
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

impl ServerResponse {
    pub fn serialize(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    pub fn deserialize(encoded: &[u8]) -> Result<ServerResponse, bincode::Error> {
        bincode::deserialize(&encoded)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use super::ClientRequest;
    use crate::{
        core::NonZeroMoney,
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
