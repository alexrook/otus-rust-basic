use std::{borrow::Cow, io::Read};

use crate::core::*;
pub trait Serializable<'a> {
    fn get_type_id(&self) -> u8;

    fn cow_bytes(&'a self) -> Cow<'a, [u8]>;

    fn size(&'a self) -> usize {
        self.cow_bytes().len()
    }
}

impl<'a> Serializable<'a> for AccountId {
    fn get_type_id(&self) -> u8 {
        42
    }

    fn cow_bytes(&'a self) -> Cow<'a, [u8]> {
        Cow::Borrowed(self.as_bytes())
    }
}

impl<'a> Serializable<'a> for Operation {
    fn get_type_id(&self) -> u8 {
        match self {
            Operation::Create(_) => 1,
            Operation::Deposit(_) => 2,
            Operation::Withdraw(_) => 3,
        }
    }

    fn size(&self) -> usize {
        match self {
            Operation::Create(account_id) => account_id.as_bytes().len(),
            Operation::Deposit(money) => money.get().to_be_bytes().len(),
            Operation::Withdraw(money) => money.get().to_be_bytes().len(),
        }
    }

    fn cow_bytes(&'a self) -> Cow<'a, [u8]> {
        match self {
            Operation::Create(account_id) => account_id.cow_bytes(),
            Operation::Deposit(money) => Cow::Owned(money.get().to_be_bytes().to_vec()),
            Operation::Withdraw(money) => Cow::Owned(money.get().to_be_bytes().to_vec()),
        }
    }
}

pub trait Deserializable<T, E> {
    fn read(type_id: u8, size: usize, data: &[u8]) -> Result<T, E>;
}

impl<E> Deserializable<String, E> for String
where
    E: From<String>,
{
    fn read(type_id: u8, size: usize, data: &[u8]) -> Result<String, E> {
        assert_eq!(AccountId::new().get_type_id(), type_id);
        std::str::from_utf8(data.take(size as u64).get_ref())
            .map(|str| str.to_string())
            .map_err(|err| format!("An error occured while string deserialization[{}]", err).into())
    }
}

impl<E> Deserializable<Operation, E> for Operation
where
    E: From<String>,
{
    fn read(type_id: u8, size: usize, data: &[u8]) -> Result<Operation, E> {
        match type_id {
            1 => {
                let account_id: AccountId = AccountId::read(type_id, size, data)?;
                Ok(Operation::Create(account_id))
            }
            2 => {
                assert_eq!(size, 4); //bcs NonZeroU32
                let mut array = [0u8; 4];
                array.copy_from_slice(data);
                let from_bytes_value = u32::from_be_bytes(array);
                let non_zero_money: NonZeroMoney = NonZeroMoney::new(from_bytes_value).ok_or(
                    "An error occured while from array to NonZeroMoney conversion".to_owned(),
                )?;
                Ok(Operation::Deposit(non_zero_money))
            }
            3 => {
                assert_eq!(size, 4); 
                let mut array = [0u8; 4];
                array.copy_from_slice(data);
                let from_bytes_value = u32::from_be_bytes(array);
                let non_zero_money: NonZeroMoney = NonZeroMoney::new(from_bytes_value).ok_or(
                    "An error occured while from array to NonZeroMoney conversion".to_owned(),
                )?;
                Ok(Operation::Withdraw(non_zero_money))
            }
            other => Err(format!("unsupported type_id[{}]", other).into()),
        }
    }
}
