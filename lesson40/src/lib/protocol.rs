use std::fmt::Debug;

use crate::core::{Account, AccountId, NonZeroMoney, Operation};

#[derive(Debug, PartialEq, Eq)]
pub enum ClientRequest<'a> {
    Create(&'a AccountId),                 //регистрация счёта
    Deposit(&'a AccountId, NonZeroMoney),  //пополнение
    Withdraw(&'a AccountId, NonZeroMoney), //снятие
    Move {
        //перевод
        from: &'a AccountId,
        to: &'a AccountId,
        amount: NonZeroMoney,
    },
    GetBalance(&'a AccountId),//получение баланса
    Quit, //завершение сеанса
}

pub enum ServerResponse{
    AccountState(Account), //Create, Deposit, Withdraw, GetBalance ops response
    FundsMovement{ //Move op response
        from:Account,
        to:Account
    }
}