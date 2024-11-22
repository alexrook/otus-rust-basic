use crate::core::*;

const MAX_ACCOUNT_ID_LEN: usize = 16;

type TypeIdMark = u8;
const TYPE_ID_ACCOUNT_ID: TypeIdMark = 42;
const TYPE_ID_NONZERO_MONEY: TypeIdMark = 52;
const TYPE_ID_OPERATION_CREATE: TypeIdMark = 1;
const TYPE_ID_OPERATION_DEPOSIT: TypeIdMark = 2;
const TYPE_ID_OPERATION_WITHDRAW: TypeIdMark = 3;

pub struct Cursor {
    pub pos: usize,
}

type Result<T, E> = std::result::Result<(T, Cursor), E>;

pub trait Serializable {
    fn serialize(&self) -> Vec<u8>;
}

impl Serializable for AccountId {
    fn serialize(&self) -> Vec<u8> {
        let str_bytes: &[u8] = self.as_bytes();
        assert!(str_bytes.len() < MAX_ACCOUNT_ID_LEN); //I think 128 bits is good enough.
        let mut ret: Vec<u8> =
            Vec::with_capacity(size_of::<TypeIdMark>() + size_of::<u8>() + str_bytes.len());
        ret.push(TYPE_ID_ACCOUNT_ID);
        ret.push(str_bytes.len() as u8);
        ret.extend_from_slice(str_bytes);
        ret
    }
}

impl Serializable for NonZeroMoney {
    fn serialize(&self) -> Vec<u8> {
        const SIZE_OF_NON_ZERO_MONEY: usize = size_of::<[u8; 4]>();
        let be_bytes: [u8; SIZE_OF_NON_ZERO_MONEY as usize] = self.get().to_be_bytes();
        let mut ret: Vec<u8> =
            Vec::with_capacity(size_of::<TypeIdMark>() + size_of::<u8>() + SIZE_OF_NON_ZERO_MONEY);
        ret.push(TYPE_ID_NONZERO_MONEY);
        ret.push(4); //
        ret.extend_from_slice(&be_bytes);
        ret
    }
}

impl Serializable for Operation {
    fn serialize(&self) -> Vec<u8> {
        match self {
            Operation::Create(account_id) => {
                let mut account_id_bytes: Vec<u8> = account_id.serialize();
                let len_account_id_bytes: usize = account_id_bytes.len();
                assert!(len_account_id_bytes < u8::MAX as usize);
                let mut ret: Vec<u8> = Vec::with_capacity(
                    size_of::<TypeIdMark>() + size_of::<u8>() + len_account_id_bytes,
                );

                ret.push(TYPE_ID_OPERATION_CREATE);
                ret.push(len_account_id_bytes as u8);
                ret.append(&mut account_id_bytes);

                ret
            }
            Operation::Deposit(money) => {
                let mut money_bytes: Vec<u8> = money.serialize();
                let len_total: usize = money_bytes.len();
                assert!(len_total < u8::MAX as usize);
                let mut ret: Vec<u8> =
                    Vec::with_capacity(size_of::<TypeIdMark>() + money_bytes.len());
                ret.push(TYPE_ID_OPERATION_DEPOSIT);
                ret.push(len_total as u8);
                ret.append(&mut money_bytes);
                ret
            }
            Operation::Withdraw(money) => {
                let mut money_bytes: Vec<u8> = money.serialize();
                let len_total: usize = money_bytes.len();
                assert!(len_total < u8::MAX as usize);
                let mut ret: Vec<u8> =
                    Vec::with_capacity(size_of::<TypeIdMark>() + money_bytes.len());
                ret.push(TYPE_ID_OPERATION_WITHDRAW);
                ret.push(len_total as u8);
                ret.append(&mut money_bytes);
                ret
            }
        }
    }
}

pub trait Deserializable<T, E> {
    fn deserialize(data: &[u8]) -> Result<T, E>;
}

impl<E> Deserializable<String, E> for AccountId
where
    E: From<String>,
{
    fn deserialize(data: &[u8]) -> Result<String, E> {
        data.get(0)
            .map(|type_id| *type_id == TYPE_ID_ACCOUNT_ID)
            .filter(|b| *b)
            .ok_or(E::from(
                format!("The first byte in array isn't specified or isn't equal to the expected[{}] type id",TYPE_ID_ACCOUNT_ID)                    ,
            ))?;

        let len: usize = data.get(1).map(|x| *x as usize).ok_or(E::from(
            "The second byte in array should be equal next data length".to_string(),
        ))?;

        let shift = 1 + 1; // first + second
        let end = shift + len;

        data.get(shift..end)
            .ok_or(E::from(format!(
                "The array should contain enough bytes for get operation[{}..{}]",
                shift, end
            )))
            .and_then(|next: &[u8]| {
                std::str::from_utf8(next).map_err(|err| {
                    format!("An error occurred while string deserialization[{}]", err).into()
                })
            })
            .map(|str| (str.to_string(), Cursor { pos: end }))
    }
}

// impl<E> Deserializable<Operation, E> for Operation
// where
//     E: From<String>,
// {
//     fn read(type_id: u8, size: usize, data: &[u8]) -> Result<Operation, E> {
//         match type_id {
//             1 => {
//                 let account_id: AccountId = AccountId::read(type_id, size, data)?;
//                 Ok(Operation::Create(account_id))
//             }
//             2 => {
//                 assert_eq!(size, 4); //bcs NonZeroU32
//                 let mut array = [0u8; 4];
//                 array.copy_from_slice(data);
//                 let from_bytes_value = u32::from_be_bytes(array);
//                 let non_zero_money: NonZeroMoney = NonZeroMoney::new(from_bytes_value).ok_or(
//                     "An error occured while from array to NonZeroMoney conversion".to_owned(),
//                 )?;
//                 Ok(Operation::Deposit(non_zero_money))
//             }
//             3 => {
//                 assert_eq!(size, 4);
//                 let mut array = [0u8; 4];
//                 array.copy_from_slice(data);
//                 let from_bytes_value = u32::from_be_bytes(array);
//                 let non_zero_money: NonZeroMoney = NonZeroMoney::new(from_bytes_value).ok_or(
//                     "An error occured while from array to NonZeroMoney conversion".to_owned(),
//                 )?;
//                 Ok(Operation::Withdraw(non_zero_money))
//             }
//             other => Err(format!("unsupported type_id[{}]", other).into()),
//         }
//     }
// }

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn serialize_account_id_should_work() {
        let initial: AccountId = "Hello Rust".to_string();

        let serialized: Vec<u8> = initial.serialize();

        assert_eq!(
            serialized.len(),
            size_of::<TypeIdMark>() + size_of::<u8>() + initial.as_bytes().len()
        );

        let initial: AccountId = "".to_string();

        let serialized: Vec<u8> = initial.serialize();

        assert_eq!(
            serialized.len(),
            size_of::<TypeIdMark>() + size_of::<u8>() + initial.as_bytes().len()
        );
    }

    #[test]
    fn serialize_non_zero_money_should_work() {
        let initial: NonZeroMoney = NonZeroMoney::MAX;

        let serialized: Vec<u8> = initial.serialize();

        assert_eq!(
            serialized.len(),
            size_of::<TypeIdMark>() + size_of::<u8>() + size_of::<NonZeroMoney>()
        );
        //     🠉 money type        +    🠉 len      +    🠉 bytes(4)

        let initial: NonZeroMoney = NonZeroMoney::MIN;

        let serialized: Vec<u8> = initial.serialize();

        assert_eq!(
            serialized.len(),
            size_of::<TypeIdMark>() + size_of::<u8>() + size_of::<NonZeroMoney>()
        );

        let initial: NonZeroMoney = NonZeroMoney::new(42).unwrap();

        let serialized: Vec<u8> = initial.serialize();

        assert_eq!(
            serialized.len(),
            size_of::<TypeIdMark>() + size_of::<u8>() + size_of::<NonZeroMoney>()
        );
    }

    #[test]
    fn serialize_operation_should_work() {
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

        let money = NonZeroMoney::new(42).unwrap();
        let initial = Operation::Deposit(money);

        let serialized: Vec<u8> = initial.serialize();

        assert_eq!(
            serialized.len(),
            size_of::<TypeIdMark>() //type of operation
             + size_of::<u8>()      //size of whole type
             + size_of::<TypeIdMark>() //type of money
             + size_of::<u8>()      //size of money type
             + money.get().to_be_bytes().len()
        );

        let money = NonZeroMoney::new(42 * 42).unwrap();
        let initial = Operation::Withdraw(money);

        let serialized: Vec<u8> = initial.serialize();

        assert_eq!(
            serialized.len(),
            size_of::<TypeIdMark>() //type of operation
             + size_of::<u8>()      //size of whole type
             + size_of::<TypeIdMark>() //type of money
             + size_of::<u8>()      //size of money type
             + money.get().to_be_bytes().len()
        );
    }

    #[test]
    fn deserialize_account_id_should_work() {
        let initial: AccountId = "Hello Rust".to_string();

        let serialized: Vec<u8> = initial.serialize();

        let actual: Result<String, String> = AccountId::deserialize(&serialized);

        assert!(actual.is_ok());

        assert_eq!(initial, actual.unwrap().0);
    }
}
