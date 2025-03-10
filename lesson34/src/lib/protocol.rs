use std::{
    fmt::Debug,
    io::{Read, Write},
};

use crate::{
    constants::MAGIC_DATA_SIZE,
    core::{Account, Operation},
    ser_de::{Deserializable, Serializable},
};

#[derive(Debug, PartialEq, Eq)]
pub enum Protocol {
    Request(Operation),
    Response(Result<Vec<Account>, String>),
    Quit,
}

pub struct IO;

impl IO {
    pub fn read<E>(reader: &mut impl Read) -> Result<Protocol, E>
    where
        E: From<String> + Debug,
    {
        let mut magic_data = [0_u8; MAGIC_DATA_SIZE];
        reader
            .read_exact(&mut magic_data)
            .map_err(|err| E::from(err.to_string()))?;
        let total_size: u8 = MAGIC_DATA_SIZE as u8 + magic_data[1]; //total protocol message size
        let mut buf = vec![0; total_size as usize];
        buf[0] = magic_data[0]; //"восстанавливаем" первые два байта
        buf[1] = magic_data[1];
        reader //в остальные байты читаем сообщение
            .read_exact(&mut buf[MAGIC_DATA_SIZE..])
            .map_err(|err| E::from(err.to_string()))?;

        Protocol::deserialize(&buf).map(|(protocol, _)| protocol)
    }

    pub fn write<E>(writer: &mut impl Write, protocol: &Protocol) -> Result<(), E>
    where
        E: From<String>,
    {
        let bytes = protocol.serialize();
        writer
            .write_all(&bytes)
            .map_err(|err| E::from(err.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::core::*;
    use crate::ser_de::*;

    use super::*;

    #[test]
    fn io_read_request_should_work() {
        fn test(initial: Protocol) {
            let bytes: Vec<u8> = initial.serialize();

            let deserialized: DesResult<Protocol, String> = Protocol::deserialize(&bytes);
            assert!(deserialized.is_ok());

            let mut slice = &bytes[..];

            let actual: Result<Protocol, String> = IO::read(&mut slice);
            assert!(actual.is_ok());
            assert_eq!(initial, actual.unwrap());
        }

        test(Protocol::Request(Operation::Deposit(
            "acc1".to_string(),
            NonZeroMoney::MAX,
        )));

        test(Protocol::Request(Operation::Deposit(
            "acc1".to_string(),
            NonZeroMoney::MIN,
        )));

        test(Protocol::Request(Operation::Create("acc1".to_string())));

        test(Protocol::Request(Operation::Withdraw(
            "acc1".to_string(),
            NonZeroMoney::new(42).unwrap(),
        )));
    }

    #[test]
    fn io_read_response_should_work() {
        fn test(initial: Protocol) {
            let bytes: Vec<u8> = initial.serialize();

            let deserialized: DesResult<Protocol, String> = Protocol::deserialize(&bytes);
            assert!(deserialized.is_ok());

            let mut slice = &bytes[..];

            let actual: Result<Protocol, String> = IO::read(&mut slice);
            assert!(actual.is_ok());
            assert_eq!(initial, actual.unwrap());
        }

        test(Protocol::Response(Ok(vec![Account {
            account_id: "acc1".to_string(),
            balance: 42,
        }])));

        test(Protocol::Response(Ok(vec![
            Account {
                account_id: "acc1".to_string(),
                balance: 42,
            },
            Account {
                account_id: "acc2".to_string(),
                balance: 0,
            },
        ])));

        test(Protocol::Response(Err("an error".to_string())));
    }

    #[test]
    fn io_write_request_should_work() {
        fn test(initial: Protocol) {
            let mut writer: Vec<u8> = Vec::new();
            let actual: Result<(), String> = IO::write(&mut writer, &initial);
            assert!(actual.is_ok());

            let deserialized: DesResult<Protocol, String> = Protocol::deserialize(&writer);
            assert!(deserialized.is_ok());
            assert_eq!(initial, deserialized.unwrap().0);
        }

        test(Protocol::Request(Operation::Move {
            from: "acc1".to_string(),
            to: "acc2".to_string(),
            amount: NonZeroMoney::MIN,
        }));
        test(Protocol::Request(Operation::Deposit(
            "acc1".to_string(),
            NonZeroMoney::MAX,
        )));

        test(Protocol::Request(Operation::Deposit(
            "acc1".to_string(),
            NonZeroMoney::MIN,
        )));

        test(Protocol::Request(Operation::Create("acc1".to_string())));

        test(Protocol::Request(Operation::Withdraw(
            "acc1".to_string(),
            NonZeroMoney::new(42).unwrap(),
        )));
    }

    #[test]
    fn io_write_response_should_work() {
        fn test(initial: Protocol) {
            let mut writer: Vec<u8> = Vec::new();
            let actual: Result<(), String> = IO::write(&mut writer, &initial);
            assert!(actual.is_ok());

            let deserialized: DesResult<Protocol, String> = Protocol::deserialize(&writer);
            assert!(deserialized.is_ok());
            assert_eq!(initial, deserialized.unwrap().0);
        }

        test(Protocol::Response(Ok(vec![Account {
            account_id: "acc1".to_string(),
            balance: 42,
        }])));

        test(Protocol::Response(Ok(vec![
            Account {
                account_id: "acc1".to_string(),
                balance: 42,
            },
            Account {
                account_id: "acc2".to_string(),
                balance: 0,
            },
        ])));

        test(Protocol::Response(Err("an error".to_string())));
    }

    #[test]
    fn io_read_write_should_work() {
        fn test(initial: Protocol) {
            let mut buf: Vec<u8> = Vec::new();
            let ret: Result<(), String> = IO::write(&mut buf, &initial);
            assert!(ret.is_ok());

            let ret: Result<Protocol, String> = IO::read(&mut std::io::Cursor::new(buf));
            assert!(ret.is_ok());
            assert_eq!(initial, ret.unwrap());
        }

        test(Protocol::Request(Operation::Move {
            from: "acc1".to_string(),
            to: "acc2".to_string(),
            amount: NonZeroMoney::MIN,
        }));
        test(Protocol::Request(Operation::Deposit(
            "acc1".to_string(),
            NonZeroMoney::MAX,
        )));

        test(Protocol::Response(Ok(vec![Account {
            account_id: "acc1".to_string(),
            balance: 42,
        }])));

        test(Protocol::Response(Ok(vec![
            Account {
                account_id: "acc1".to_string(),
                balance: 42,
            },
            Account {
                account_id: "acc2".to_string(),
                balance: 0,
            },
        ])));

        test(Protocol::Response(Ok(Vec::new()))); //TODO: Should an empty answer be prohibited?
    }
}
