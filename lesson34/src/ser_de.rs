use std::fmt::Debug;

use crate::core::*;

use crate::constants::*;
use crate::protocol::Protocol;

#[derive(Debug)]
pub struct Cursor {
    pub pos: usize,
}

pub type DesResult<T, E> = std::result::Result<(T, Cursor), E>;

pub trait Serializable {
    fn serialize(&self) -> Vec<u8>;

    fn for_simple(self_type_id_mark: TypeIdMark, bytes: &[u8]) -> Vec<u8> {
        let all_inner_len: usize = bytes.len();
        assert!(all_inner_len < u8::MAX as usize);

        let mut ret: Vec<u8> =
            Vec::with_capacity(size_of::<TypeIdMark>() + size_of::<u8>() + all_inner_len);

        ret.push(self_type_id_mark);
        ret.push(all_inner_len as u8);
        ret.extend_from_slice(bytes);
        ret
    }

    //TODO: macro impl for tuples ?
    fn for_tuple_2<I1, I2>(self_type_id_mark: TypeIdMark, inner: (&I1, &I2)) -> Vec<u8>
    where
        I1: Serializable,
        I2: Serializable,
    {
        let (i1, i2) = inner;
        //I1
        let mut i1: Vec<u8> = i1.serialize();
        //I2
        let mut i2: Vec<u8> = i2.serialize();
        //Len
        let all_inner_len: usize = i1.len() + i2.len();
        let full_len = size_of::<TypeIdMark>() + size_of::<u8>() + all_inner_len;
        assert!(full_len < u8::MAX as usize);
        //Push
        let mut ret: Vec<u8> = Vec::with_capacity(full_len);
        ret.push(self_type_id_mark);
        ret.push(all_inner_len as u8);
        ret.append(&mut i1);
        ret.append(&mut i2);
        ret
    }

    fn for_tuple_3<I1, I2, I3>(self_type_id_mark: TypeIdMark, inner: (&I1, &I2, &I3)) -> Vec<u8>
    where
        I1: Serializable,
        I2: Serializable,
        I3: Serializable,
    {
        let (i1, i2, i3) = inner;
        //I1
        let mut i1: Vec<u8> = i1.serialize();
        //I2
        let mut i2: Vec<u8> = i2.serialize();
        //I3
        let mut i3: Vec<u8> = i3.serialize();
        //Len
        let all_inner_len: usize = i1.len() + i2.len() + i3.len();
        let full_len: usize = size_of::<TypeIdMark>() + size_of::<u8>() + all_inner_len;
        assert!(full_len < u8::MAX as usize);
        //Push
        let mut ret: Vec<u8> = Vec::with_capacity(full_len);
        ret.push(self_type_id_mark);
        ret.push(all_inner_len as u8);
        ret.append(&mut i1);
        ret.append(&mut i2);
        ret.append(&mut i3);
        ret
    }

    fn for_one<I>(self_type_id_mark: TypeIdMark, inner: &I) -> Vec<u8>
    where
        I: Serializable,
    {
        let mut inner: Vec<u8> = inner.serialize();
        //Len
        let all_inner_len: usize = inner.len();
        let full_len: usize = size_of::<TypeIdMark>() + size_of::<u8>() + all_inner_len;
        assert!(full_len < u8::MAX as usize);
        //Push
        let mut ret = Vec::with_capacity(full_len);
        ret.push(self_type_id_mark);
        ret.push(all_inner_len as u8);

        ret.append(&mut inner);
        ret
    }
}

impl Serializable for AccountId {
    fn serialize(&self) -> Vec<u8> {
        let str_bytes: &[u8] = self.as_bytes();
        assert!(str_bytes.len() < MAX_ACCOUNT_ID_LEN); //I think 128 bits is good enough.
        Self::for_simple(TYPE_ID_ACCOUNT_ID, str_bytes)
    }
}

impl Serializable for Money {
    fn serialize(&self) -> Vec<u8> {
        let be_bytes: [u8; 4] = self.to_be_bytes();
        Self::for_simple(TYPE_ID_MONEY, &be_bytes)
    }
}

impl Serializable for NonZeroMoney {
    fn serialize(&self) -> Vec<u8> {
        let be_bytes: [u8; 4] = self.get().to_be_bytes();
        Self::for_simple(TYPE_ID_NONZERO_MONEY, &be_bytes)
    }
}

impl<T: Serializable, E: Serializable> Serializable for Result<T, E> {
    fn serialize(&self) -> Vec<u8> {
        match self {
            Ok(inner) => Self::for_one(TYPE_ID_RESULT_OK, inner),
            Err(err) => Self::for_one(TYPE_ID_RESULT_ERR, err),
        }
    }
}

impl<T: Serializable> Serializable for Vec<T> {
    fn serialize(&self) -> Vec<u8> {
        {
            let mut all_el_bytes = Vec::new();
            for el in self {
                let mut el_bytes = el.serialize();
                all_el_bytes.append(&mut el_bytes);
            }

            //Len
            let all_inner_len: usize = all_el_bytes.len();
            let full_len = size_of::<TypeIdMark>() + size_of::<u8>() + all_inner_len;
            assert!(full_len < u8::MAX as usize);
            //Push
            let mut ret = Vec::with_capacity(full_len);
            ret.push(TYPE_ID_VEC);
            ret.push(all_inner_len as u8);
            ret.append(&mut all_el_bytes);
            ret
        }
    }
}

impl Serializable for Account {
    fn serialize(&self) -> Vec<u8> {
        Self::for_tuple_2(TYPE_ID_ACCOUNT, (&self.account_id, &self.balance))
    }
}

impl Serializable for Operation {
    fn serialize(&self) -> Vec<u8> {
        match self {
            Operation::Create(account_id) => Self::for_one(TYPE_ID_OPERATION_CREATE, account_id),

            Operation::Deposit(account_id, money) => {
                Self::for_tuple_2(TYPE_ID_OPERATION_DEPOSIT, (account_id, money))
            }

            Operation::Withdraw(account_id, money) => {
                Self::for_tuple_2(TYPE_ID_OPERATION_WITHDRAW, (account_id, money))
            }

            Operation::Move { from, to, amount } => {
                Self::for_tuple_3(TYPE_ID_OPERATION_MOVE, (from, to, amount))
            }

            Operation::GetBalance(account_id) => {
                Self::for_one(TYPE_ID_OPERATION_GETBALANCE, account_id)
            }
        }
    }
}

impl Serializable for Protocol {
    fn serialize(&self) -> Vec<u8> {
        match self {
            Protocol::Request(op) => Self::for_one(TYPE_ID_PROTOCOL_REQUEST, op),
            Protocol::Response(result) => Self::for_one(TYPE_ID_PROTOCOL_RESPONSE, result),
        }
    }
}

// impl<T> Serializable for &T
// where
//     T: Serializable,
// {
//     fn serialize(&self) -> Vec<u8> {
//         (*self).serialize()
//     }
// }

pub trait Deserializable<T, E> {
    fn deserialize(data: &[u8]) -> DesResult<T, E>;

    fn unmarshall<C, F, I>(is_expected_type_id: C, data: &[u8], unmarshaller: F) -> DesResult<T, E>
    where
        E: From<String>,
        C: FnOnce(&TypeIdMark) -> bool,
        I: Debug,
        F: FnOnce(&TypeIdMark, &[u8]) -> std::result::Result<T, I>,
    {
        let type_id: &TypeIdMark = data
            .get(0)
            .filter(|type_id| is_expected_type_id(type_id))
            .ok_or(E::from(format!(
                "The first byte in data isn't specified or isn't equal to the expected type id"
            )))?;

        let len: usize = data.get(1).map(|x| *x as usize).ok_or(E::from(
            "The second byte in data should be equal next data length".to_string(),
        ))?;

        let shift = 1 + 1; // first + second
        let end = shift + len;
        data.get(shift..end)
            .ok_or(E::from(format!(
                "The array should contain enough bytes for get operation[{}..{}]",
                shift, end
            )))
            .and_then(|next: &[u8]| {
                unmarshaller(type_id, next).map_err(|err| {
                    format!("An error occurred while string deserialization[{:?}]", err).into()
                })
            })
            .map(|ret| (ret, Cursor { pos: end }))
    }
}

impl<E> Deserializable<String, E> for AccountId
where
    E: From<String>,
{
    fn deserialize(data: &[u8]) -> DesResult<String, E> {
        Self::unmarshall(
            |type_id| *type_id == TYPE_ID_ACCOUNT_ID,
            data,
            |_, next: &[u8]| std::str::from_utf8(next).map(|str| str.to_string()),
        )
    }
}

impl<E> Deserializable<Money, E> for Money
where
    E: From<String> + Debug,
{
    fn deserialize(data: &[u8]) -> DesResult<Money, E> {
        Self::unmarshall(
            |type_id| *type_id == TYPE_ID_MONEY,
            data,
            |_, next: &[u8]| {
                let mut array = [0u8; 4];
                array.copy_from_slice(next);
                Ok::<u32, E>(u32::from_be_bytes(array))
            },
        )
    }
}

impl<E> Deserializable<NonZeroMoney, E> for NonZeroMoney
where
    E: From<String> + Debug,
{
    fn deserialize(data: &[u8]) -> DesResult<NonZeroMoney, E> {
        Self::unmarshall(
            |type_id| *type_id == TYPE_ID_NONZERO_MONEY,
            data,
            |_, next: &[u8]| {
                let mut array = [0u8; 4];
                array.copy_from_slice(next);
                let from_bytes_value = u32::from_be_bytes(array);
                NonZeroMoney::new(from_bytes_value).ok_or(E::from(
                    "An error occured while from array to NonZeroMoney conversion".to_owned(),
                ))
            },
        )
    }
}

impl<E> Deserializable<Account, E> for Account
where
    E: From<String> + Debug,
{
    fn deserialize(data: &[u8]) -> DesResult<Account, E> {
        Self::unmarshall(
            |type_id| *type_id == TYPE_ID_ACCOUNT,
            data,
            |_, next: &[u8]| {
                let (account_id, cursor) = AccountId::deserialize(next)?;
                let next = next
                    .get(cursor.pos..)
                    .ok_or(format!("unsufficient bytes for money reading"))?;
                let (balance, _) = Money::deserialize(&next)?;
                Ok::<Account, E>(Account {
                    account_id,
                    balance,
                })
            },
        )
    }
}

impl<R1, R2, E> Deserializable<Result<R1, R2>, E> for Result<R1, R2>
where
    E: From<String> + Debug,
    R1: Deserializable<R1, E>,
    R2: Deserializable<R2, E>,
{
    fn deserialize(data: &[u8]) -> DesResult<Result<R1, R2>, E> {
        Self::unmarshall(
            |type_id| vec![TYPE_ID_RESULT_OK, TYPE_ID_RESULT_ERR].contains(type_id),
            data,
            |type_id, next: &[u8]| match *type_id {
                TYPE_ID_RESULT_OK => {
                    let (inner, _) = R1::deserialize(next)?;
                    //deserialization result
                    Ok(
                        Ok(inner), //wrapped Ok Result
                    )
                }

                TYPE_ID_RESULT_ERR => {
                    let (inner, _) = R2::deserialize(next)?;
                    Ok(
                        Err(inner), //wrapped Err Result
                    )
                }

                other => Err::<Result<R1, R2>, E>(format!("unsupported type_id[{}]", other).into()),
            },
        )
    }
}

impl<T, E> Deserializable<Vec<T>, E> for Vec<T>
where
    T: Deserializable<T, E>,
    E: From<String> + Debug,
{
    fn deserialize(data: &[u8]) -> DesResult<Vec<T>, E> {
        Self::unmarshall(
            |type_id| *type_id == TYPE_ID_VEC,
            data,
            |_, next: &[u8]| {
                let mut ret = Vec::new();
                let mut next = next;
                let mut pos = 0;
                while pos < next.len() {
                    next = next
                        .get(pos..)
                        .ok_or(format!("unsufficient bytes for next elem reading"))?;
                    let (elem, cursor) = T::deserialize(next)?;
                    ret.push(elem);
                    pos = cursor.pos;
                }
                Ok::<Vec<T>, E>(ret)
            },
        )
    }
}

impl<E> Deserializable<Operation, E> for Operation
where
    E: From<String> + Debug,
{
    fn deserialize(data: &[u8]) -> DesResult<Operation, E> {
        Self::unmarshall(
            |type_id| {
                [
                    TYPE_ID_OPERATION_CREATE,
                    TYPE_ID_OPERATION_DEPOSIT,
                    TYPE_ID_OPERATION_WITHDRAW,
                    TYPE_ID_OPERATION_MOVE,
                    TYPE_ID_OPERATION_GETBALANCE,
                ]
                .contains(type_id)
            },
            data,
            |type_id, next| match *type_id {
                TYPE_ID_OPERATION_CREATE => AccountId::deserialize(next)
                    .map(|(account_id, _)| Operation::Create(account_id)),

                TYPE_ID_OPERATION_DEPOSIT => {
                    let (account_id, cursor) = AccountId::deserialize(next)?;
                    let next = next
                        .get(cursor.pos..)
                        .ok_or(format!("unsufficient bytes for money reading"))?;
                    let (non_zero_money, _) = NonZeroMoney::deserialize(&next)?;
                    Ok(Operation::Deposit(account_id, non_zero_money))
                }

                TYPE_ID_OPERATION_WITHDRAW => {
                    let (account_id, cursor) = AccountId::deserialize(next)?;
                    let next = next
                        .get(cursor.pos..)
                        .ok_or(format!("unsufficient bytes for money reading"))?;
                    let (non_zero_money, _) = NonZeroMoney::deserialize(&next)?;
                    Ok(Operation::Withdraw(account_id, non_zero_money))
                }

                TYPE_ID_OPERATION_MOVE => {
                    let (from, cursor) = AccountId::deserialize(next)?;
                    let next = next
                        .get(cursor.pos..)
                        .ok_or(format!("unsufficient bytes for account id reading"))?;
                    let (to, cursor) = AccountId::deserialize(next)?;
                    let next = next
                        .get(cursor.pos..)
                        .ok_or(format!("unsufficient bytes for money reading"))?;

                    let (amount, _) = NonZeroMoney::deserialize(&next)?;
                    Ok(Operation::Move { from, to, amount })
                }

                TYPE_ID_OPERATION_GETBALANCE => AccountId::deserialize(next)
                    .map(|(account_id, _)| Operation::GetBalance(account_id)),

                other => Err::<Operation, E>(format!("unsupported type_id[{}]", other).into()),
            },
        )
    }
}

impl<E> Deserializable<Protocol, E> for Protocol
where
    E: From<String> + Debug,
{
    fn deserialize(data: &[u8]) -> DesResult<Protocol, E> {
        Self::unmarshall(
            |type_id| [TYPE_ID_PROTOCOL_REQUEST, TYPE_ID_PROTOCOL_RESPONSE].contains(type_id),
            data,
            |type_id, next| match *type_id {
                TYPE_ID_PROTOCOL_REQUEST => {
                    let (operation, _) = Operation::deserialize(next)?;
                    Ok(Protocol::Request(operation))
                }
                TYPE_ID_PROTOCOL_RESPONSE => {
                    let (result, _) = Result::<Vec<Account>, String>::deserialize(next)?;
                    Ok(Protocol::Response(result))
                }
                other => Err::<Protocol, E>(format!("unsupported type_id[{}]", other).into()),
            },
        )
    }
}

#[cfg(test)]
mod serialize_tests {

    use super::*;

    #[test]
    fn serialize_account_id_should_work() {
        fn test(initial: String) {
            let serialized: Vec<u8> = initial.serialize();

            assert_eq!(
                serialized.len(),
                size_of::<TypeIdMark>() + size_of::<u8>() + initial.as_bytes().len()
            );
        }
        test("Hello Rust".to_string());
        test("".to_string());
        test("a".to_string());
        test("b".to_string());
    }

    #[test]
    fn serialize_non_zero_money_should_work() {
        fn test(initial: NonZeroMoney) {
            let serialized: Vec<u8> = initial.serialize();

            assert_eq!(
                serialized.len(),
                size_of::<TypeIdMark>() + size_of::<u8>() + size_of::<NonZeroMoney>()
            );
            //     🠉 money type        +    🠉 len      +    🠉 bytes(4)
        }

        test(NonZeroMoney::MIN);
        test(NonZeroMoney::MAX);
        test(NonZeroMoney::new(42).unwrap());
    }

    #[test]
    fn serialize_result_should_work() {
        fn test<S: Serializable + Clone>(inner: S) {
            let inner_bytes_len = inner.serialize().len();
            let initial: Result<S, S> = Ok(inner.clone());
            assert_eq!(
                initial.serialize().len(),
                size_of::<TypeIdMark>() + size_of::<u8>() + inner_bytes_len
            );
            let initial: Result<S, S> = Err(inner);
            assert_eq!(
                initial.serialize().len(),
                size_of::<TypeIdMark>() + size_of::<u8>() + inner_bytes_len
            );
            //     🠉 Result type        +    🠉 len      +    🠉 inner bytes
        }

        test(NonZeroMoney::MIN);
        test(NonZeroMoney::MAX);
        test("Hello Rust".to_string());
        test("".to_string());
    }

    #[test]
    fn serialize_vec_should_work() {
        fn test<E: Serializable + Clone>(initial: Vec<E>) {
            let bytes = initial.serialize();
            let all_bytes_len = bytes.len();
            let inner_bytes_len = bytes.len() - (1/*u8 len*/ + 1/*u8 len*/);
            assert_eq!(
                all_bytes_len,
                size_of::<TypeIdMark>() + size_of::<u8>() + inner_bytes_len
            );
            //     🠉 Result type        +    🠉 len      +    🠉 inner bytes
        }

        test(vec![
            NonZeroMoney::MIN,
            NonZeroMoney::MAX,
            NonZeroMoney::new(42).unwrap(),
        ]);
        test(vec![
            "Hello Rust".to_string(),
            "".to_string(),
            "acc1".to_string(),
        ]);
    }

    #[test]
    fn serialize_operation_create_should_work() {
        let acc = "acc1".to_string();
        let initial = Operation::Create(acc.clone());

        let serialized: Vec<u8> = initial.serialize();

        assert_eq!(
            serialized.len(),
            size_of::<TypeIdMark>()       //operation type
                + size_of::<u8>()         //size of whole type
                + size_of::<TypeIdMark>() //account id type
                + size_of::<u8>()       //size of account id 
                + acc.as_bytes().len() //account bytes
        );
    }

    #[test]
    fn serialize_operation_deposit_should_work() {
        let acc = "acc1".to_string();

        let money = NonZeroMoney::new(42).unwrap();
        let initial = Operation::Deposit(acc.clone(), money);

        let serialized: Vec<u8> = initial.serialize();

        assert_eq!(
            serialized.len(),
            size_of::<TypeIdMark>() //type of operation
             + size_of::<u8>()      //size of whole type
             + size_of::<TypeIdMark>() //account id type
             + size_of::<u8>()       //size of account id 
             + acc.as_bytes().len() //account bytes
             + size_of::<TypeIdMark>() //type of money
             + size_of::<u8>()      //size of money type
             + size_of::<NonZeroMoney>()
        );
    }
    #[test]
    fn serialize_operation_withdraw_should_work() {
        let acc = "acc1".to_string();
        let money = NonZeroMoney::new(42 * 42).unwrap();
        let initial = Operation::Withdraw(acc.clone(), money);

        let serialized: Vec<u8> = initial.serialize();

        assert_eq!(
            //the same as Deposit
            serialized.len(),
            size_of::<TypeIdMark>() //type of operation
                 + size_of::<u8>()      //size of whole type
                 + size_of::<TypeIdMark>() //account id type
                 + size_of::<u8>()       //size of account id 
                 + acc.as_bytes().len() //account bytes
                 + size_of::<TypeIdMark>() //type of money
                 + size_of::<u8>()      //size of money type
                 + size_of::<NonZeroMoney>()
        )
    }

    #[test]
    fn serialize_operation_move_should_work() {
        let acc1 = "acc1".to_string();
        let acc2 = "acc2".to_string();
        let test = |amount: NonZeroMoney| {
            let initial = Operation::Move {
                from: acc1.clone(),
                to: acc2.clone(),
                amount,
            };
            let serialized: Vec<u8> = initial.serialize();
            assert_eq!(
                serialized.len(),
                //Move header
                size_of::<TypeIdMark>() //type of operation
                     + size_of::<u8>()      //size of whole type
                     //Move.from data
                     + size_of::<TypeIdMark>() //account id type
                     + size_of::<u8>()       //size of account id 
                     + acc1.as_bytes().len() //account bytes
                     //Move.to data
                     + size_of::<TypeIdMark>() //account id type
                     + size_of::<u8>()       //size of account id 
                     + acc2.as_bytes().len() //account bytes
                     ////Move.amount data
                     + size_of::<TypeIdMark>() //type of money
                     + size_of::<u8>()      //size of money type
                     + size_of::<NonZeroMoney>()
            )
        };

        test(NonZeroMoney::MIN);
        test(NonZeroMoney::MAX);
        test(NonZeroMoney::new(42).unwrap());
        test(NonZeroMoney::new(42 * 42).unwrap());
    }

    #[test]
    fn serialize_operation_getbalance_should_work() {
        let acc = "acc1".to_string();
        let initial = Operation::GetBalance(acc.clone());
        let serialized: Vec<u8> = initial.serialize();

        assert_eq!(
            serialized.len(),
            size_of::<TypeIdMark>()       //operation type
                + size_of::<u8>()         //size of whole type
                + size_of::<TypeIdMark>() //account id type
                + size_of::<u8>()       //size of account id 
                + acc.as_bytes().len() //account bytes
        );
    }
}

//Deserializable
#[cfg(test)]
pub mod deserialize_tests {

    use super::*;

    #[test]
    fn deserialize_account_id_should_work() {
        fn test(initial: AccountId) {
            let serialized: Vec<u8> = initial.serialize();
            let actual: DesResult<String, String> = AccountId::deserialize(&serialized);

            assert!(actual.is_ok());
            assert_eq!(initial, actual.unwrap().0);
        }

        test("Hello Rust".to_string());
        test("".to_string());
        test("a".to_string());
        test("b".to_string());
    }

    #[test]
    fn deserialize_money_should_work() {
        fn test(initial: Money) {
            let serialized: Vec<u8> = initial.serialize();

            let actual: DesResult<Money, String> = Money::deserialize(&serialized);

            assert!(actual.is_ok());
            assert_eq!(initial, actual.unwrap().0);
        }

        test(Money::MIN);
        test(Money::MAX);
        test(42);
        test(42 * 42);
    }

    #[test]
    fn deserialize_account_should_work() {
        fn test(balance: Money, acc: String) {
            let initial = Account {
                account_id: acc,
                balance,
            };

            let serialized: Vec<u8> = initial.serialize();

            let actual: DesResult<Account, String> = Account::deserialize(&serialized);

            assert!(actual.is_ok());
            assert_eq!(initial, actual.unwrap().0);
        }

        test(Money::MIN, "".to_string());
        test(Money::MAX, "Hello Rust".to_string());
        test(42, "43".into());
        test(42 * 42, "42".into());
    }

    #[test]
    fn deserialize_non_zero_money_should_work() {
        fn test(initial: NonZeroMoney) {
            let serialized: Vec<u8> = initial.serialize();

            let actual: DesResult<NonZeroMoney, String> = NonZeroMoney::deserialize(&serialized);

            assert!(actual.is_ok());
            assert_eq!(initial, actual.unwrap().0);
        }

        test(NonZeroMoney::MIN);
        test(NonZeroMoney::MAX);
        test(NonZeroMoney::new(42).unwrap());
        test(NonZeroMoney::new(42 * 42).unwrap());
    }

    #[test]
    fn deserialize_operation_create_should_work() {
        fn test(acc: AccountId) {
            let initial = Operation::Create(acc);
            let serialized: Vec<u8> = initial.serialize();

            let actual: DesResult<Operation, String> = Operation::deserialize(&serialized);

            assert!(actual.is_ok());
            assert_eq!(initial, actual.unwrap().0);
        }

        test("Hello Rust".to_string());
        test("".to_string());
        test("a".to_string());
        test("b".to_string());
    }

    #[test]
    fn deserialize_operation_deposit_should_work() {
        let acc = "acc".to_string();
        let test = |money: NonZeroMoney| {
            let initial = Operation::Deposit(acc.clone(), money);
            let serialized: Vec<u8> = initial.serialize();
            let actual: DesResult<Operation, String> = Operation::deserialize(&serialized);
            assert!(actual.is_ok());
            assert_eq!(initial, actual.unwrap().0);
        };

        test(NonZeroMoney::MIN);
        test(NonZeroMoney::MAX);
        test(NonZeroMoney::new(42).unwrap());
        test(NonZeroMoney::new(42 * 42).unwrap());
    }

    #[test]
    fn deserialize_operation_withdraw_should_work() {
        let acc = "acc".to_string();
        let test = |money: NonZeroMoney| {
            let initial = Operation::Withdraw(acc.clone(), money);
            let serialized: Vec<u8> = initial.serialize();
            let actual: DesResult<Operation, String> = Operation::deserialize(&serialized);
            assert!(actual.is_ok());
            assert_eq!(initial, actual.unwrap().0);
        };

        test(NonZeroMoney::MIN);
        test(NonZeroMoney::MAX);
        test(NonZeroMoney::new(42).unwrap());
    }

    #[test]
    fn deserialize_operation_move_should_work() {
        let acc1 = "a".to_string();
        let acc2 = "b".to_string();
        let test = |money: NonZeroMoney| {
            let initial = Operation::Move {
                from: acc1.clone(),
                to: acc2.clone(),
                amount: money,
            };
            let serialized: Vec<u8> = initial.serialize();
            let actual: DesResult<Operation, String> = Operation::deserialize(&serialized);
            assert!(actual.is_ok());
            assert_eq!(initial, actual.unwrap().0);
        };

        test(NonZeroMoney::MIN);
        test(NonZeroMoney::MAX);
        test(NonZeroMoney::new(42).unwrap());
    }

    #[test]
    fn deserialize_operation_getbalance_should_work() {
        let test = |account_id: AccountId| {
            let initial = Operation::GetBalance(account_id);
            let serialized: Vec<u8> = initial.serialize();
            let actual: DesResult<Operation, String> = Operation::deserialize(&serialized);
            assert!(actual.is_ok());
            assert_eq!(initial, actual.unwrap().0);
        };

        test("".to_string());
        test("42".to_string());
        test("Hello Rust".to_string());
    }

    #[test]
    fn deserialize_result_should_work() {
        fn test<T>(inner: T)
        where
            T: Clone + Eq + Debug,
            T: Serializable,
            T: Deserializable<T, String>,
        {
            //Ok
            let initial: Result<T, T> = Ok(inner.clone());
            let serialized: Vec<u8> = initial.serialize();

            let actual: DesResult<Result<T, T>, String> = Result::deserialize(&serialized);

            assert!(actual.is_ok()); //deserialization result
            assert_eq!(initial, actual.unwrap().0);

            //Err
            let initial: Result<T, T> = Err(inner);
            let serialized: Vec<u8> = initial.serialize();

            let actual: DesResult<Result<T, T>, String> = Result::deserialize(&serialized);

            assert!(actual.is_ok()); //deserialization result
            assert_eq!(initial, actual.unwrap().0)
        }

        test(NonZeroMoney::MIN);
        test(NonZeroMoney::MAX);
        test(AccountId::from("acc1"));
        test(Operation::Create("acc3".to_string()));
        test(Operation::Deposit(
            "acc1".to_string(),
            NonZeroMoney::new(42).unwrap(),
        ));
        test(Operation::Withdraw(
            "acc2".to_string(),
            NonZeroMoney::new(42).unwrap(),
        ));
    }

    #[test]
    fn deserialize_vec_should_work() {
        fn assert_vec_eq<E>(left: Vec<E>, right: Vec<E>)
        where
            E: Eq,
        {
            let ret: bool = left.len() == right.len() &&  //размеры равны
                left.iter().zip(right.iter())//a также,
                .all(|(a, b)| a == b); //элементы равны
            assert!(ret);
        }

        fn test<T>(initial: Vec<T>)
        where
            T: Eq,
            T: Serializable,
            T: Deserializable<T, String>,
        {
            let serialized: Vec<u8> = initial.serialize();

            let actual: DesResult<Vec<T>, String> = Vec::<T>::deserialize(&serialized);

            assert!(actual.is_ok());
            let actual = actual.unwrap().0;

            assert_vec_eq(initial, actual);
        }

        test::<NonZeroMoney>(vec![
            NonZeroMoney::MIN,
            NonZeroMoney::MAX,
            NonZeroMoney::new(42).unwrap(),
        ]);

        test::<NonZeroMoney>(Vec::new()); //an empty Vec

        test(vec![
            "Hello Rust".to_string(),
            "".to_string(),
            "acc1".to_string(),
        ]);

        test(vec![
            Operation::Create("acc3".to_string()),
            Operation::Deposit("acc1".to_string(), NonZeroMoney::new(42).unwrap()),
            Operation::Withdraw("acc2".to_string(), NonZeroMoney::new(42).unwrap()),
        ]);
    }

    #[test]
    fn deserialize_protocol_request_should_work() {
        fn test(op: Operation) {
            let initial: Protocol = Protocol::Request(op);
            let serialized = initial.serialize();

            let actual: DesResult<Protocol, String> = Protocol::deserialize(&serialized);
            assert!(actual.is_ok());
            assert_eq!(initial, actual.unwrap().0);
        }

        test(Operation::Create("acc3".to_string()));
        test(Operation::Deposit(
            "acc1".to_string(),
            NonZeroMoney::new(42).unwrap(),
        ));
        test(Operation::Withdraw(
            "acc2".to_string(),
            NonZeroMoney::new(42).unwrap(),
        ));

        test(Operation::Move {
            from: "acc2".to_string(),
            to: "acc1".to_string(),
            amount: NonZeroMoney::new(42).unwrap(),
        });

        test(Operation::GetBalance("acc1".to_string()));
    }

    #[test]
    fn deserialize_protocol_response_should_work() {
        fn test(accs: Vec<Account>) {
            let initial: Protocol = Protocol::Response(Ok(accs));
            let serialized = initial.serialize();
            let actual: DesResult<Protocol, String> = Protocol::deserialize(&serialized);

            assert!(actual.is_ok());
            let (actual, _) = actual.unwrap();
            assert_eq!(initial, actual);
        }

        test(vec![
            Account {
                account_id: "acc1".to_string(),
                balance: Money::MIN,
            },
            Account {
                account_id: "acc2".to_string(),
                balance: Money::MAX,
            },
            Account {
                account_id: "acc1".to_string(),
                balance: 0 as Money,
            },
            Account {
                account_id: "acc1".to_string(),
                balance: 42 as Money,
            },
        ]);

        test(Vec::new());
    }
}
