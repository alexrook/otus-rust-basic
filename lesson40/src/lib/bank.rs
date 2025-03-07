use std::{
    collections::{BTreeMap, HashMap, LinkedList},
    num::{NonZeroU128, NonZeroU32},
};
use thiserror::Error;

pub type AccountId = u128;
pub type Money = u32;
pub type NonZeroMoney = NonZeroU32;
pub type OpId = NonZeroU128;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum BankError {
    #[error("Something get wrong in the bank core facility[{0}]")]
    CoreError(String),
    #[error("This operation is'nt allowed[{0}]")]
    Prohibited(String),
    #[error("Bad request[{0}]")]
    BadRequest(String),
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Account {
    pub account_id: AccountId,
    pub balance: Money,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    /**
     * Перед выполнением любых операций по счёту, его необходимо создать.
     * Если счёт с именем Х существует, то создание нового счёта с именем Х - ошибка
     */
    Create(AccountId), //регистрация счёта

    /**Пополнение увеличивает количество денег на счете на указанную сумму.
     * Пополнение на ноль денежных единиц - ошибка.
     */
    Deposit(AccountId, NonZeroMoney), //пополнение
    /**
     * Снятие уменьшает количество денег на счете на указанную сумму.
     * Снятие нуля денежных единиц - ошибка.
     * Попытка снять больше чем есть на счете - ошибка.
     */
    Withdraw(AccountId, NonZeroMoney), //снятие
                                       //перевод реализован через сумму операций Withdraw + Deposit
}

impl Operation {
    fn account_id(&self) -> &AccountId {
        match self {
            Operation::Create(account_id) => account_id,
            Operation::Deposit(account_id, _) | Self::Withdraw(account_id, _) => account_id,
        }
    }
}

///Банк имеет хранилище операций по счетам клиентов
pub trait OpsStorage {
    fn transact(
        &mut self,
        ops: impl Iterator<Item = Operation>,
    ) -> Result<impl Iterator<Item = (OpId, &Operation)>, BankError>;

    fn persist(&mut self, op: Operation) -> Result<(OpId, &Operation), BankError> {
        self.transact(std::iter::once(op)).and_then(|mut iter| {
            iter.next().ok_or_else(|| {
                BankError::CoreError(
                    "a transaction should return at least one operation".to_owned(),
                )
            })
        })
    }
    //should be O(N), where N - account ops
    fn get_ops<'a, 'b>(
        &'a self,
        account_id: &'b AccountId,
    ) -> Result<impl Iterator<Item = (OpId, &'a Operation)>, BankError>;
    //should be O(M), where M - all ops
    fn get_history(&self) -> Result<impl Iterator<Item = (OpId, &Operation)>, BankError>;
}

///Банк имеет текущее "состояние" счетов клиентов
pub trait State {
    fn transact<'a, 'b>(
        &'a mut self,
        ops: impl Iterator<Item = &'b Operation>,
    ) -> Result<impl Iterator<Item = &'a Account> + 'a, BankError>;

    fn update<'a>(&'a mut self, op: &Operation) -> Result<&'a Account, BankError> {
        self.transact(std::iter::once(op)).and_then(|mut iter| {
            iter.next().ok_or_else(|| {
                BankError::CoreError("a transaction should return at least one account".to_owned())
            })
        })
    }
    //should be O(N), where N - account ops
    fn get_balance(&self, account_id: &AccountId) -> Result<&Account, BankError>;
}

#[derive(Debug, Default)]
pub struct Bank<T, S> {
    storage: T,
    state: S,
}

impl<T: OpsStorage, S: State> Bank<T, S> {
    pub fn new(storage: T, state: S) -> Bank<T, S> {
        Bank { storage, state }
    }

    pub fn from<'a>(history: impl Iterator<Item = (OpId, &'a Operation)>) -> Bank<T, S>
    where
        S: Default,
        T: Default,
    {
        let mut bank: Bank<T, S> = Bank::default();

        for (_, op) in history {
            let (_, op) = bank
                .storage
                .persist(op.clone())
                .unwrap_or_else(|_| panic!("something wrong with history operation[{:?}]", op));

            if let Err(err) = bank.state.update(op) {
                println!("History operation has an error[{:?}]", err)
            }
        }

        bank
    }

    //создание аккаунта
    pub fn create_account(&mut self, account_id: AccountId) -> Result<&Account, BankError> {
        let op = Operation::Create(account_id);
        let (_, op) = self.storage.persist(op)?;
        self.state.update(op)
    }

    //Клиент может получить свой баланс.
    pub fn get_balance(&self, account_id: &AccountId) -> Result<&Account, BankError> {
        self.state.get_balance(account_id)
    }

    //история операций по счету
    pub fn get_account_ops<'a, 'b>(
        &'a self,
        account_id: &'b AccountId,
    ) -> Result<impl Iterator<Item = (OpId, &'a Operation)> + 'b, BankError>
    where
        'a: 'b,
    {
        self.storage.get_ops(account_id)
    }
    //можно получить историю операций
    pub fn get_history(&self) -> Result<impl Iterator<Item = (OpId, &Operation)>, BankError> {
        self.storage.get_history()
    }

    //Клиент может пополнить свой баланс.
    pub fn deposit(
        &mut self,
        account_id: &AccountId,
        money: NonZeroMoney,
    ) -> Result<&Account, BankError> {
        let op = Operation::Deposit(*account_id, money);
        let (_, op) = self.storage.persist(op)?;
        self.state.update(op)
    }

    //Клиент может забрать деньги
    pub fn withdraw(
        &mut self,
        account_id: AccountId,
        money: NonZeroMoney,
    ) -> Result<&Account, BankError> {
        let op = Operation::Withdraw(account_id, money);
        let (_, op) = self.storage.persist(op)?;
        self.state.update(op)
    }

    //перемещение денег от счета на счет
    pub fn move_money(
        &mut self,
        from: AccountId,
        to: AccountId,
        money: NonZeroMoney,
    ) -> Result<(&Account, &Account), BankError> {
        if from == to {
            return Err(BankError::Prohibited(format!(
                "Sending funds to yourself[{to}] is prohibited"
            )));
        }

        let ops = vec![
            Operation::Withdraw(from, money),
            Operation::Deposit(to, money),
        ]
        .into_iter();
        //first save to storage
        let ops = self.storage.transact(ops)?.map(|(_, op)| op);

        //and then update state
        let mut accounts = self.state.transact(ops)?;
        let from = accounts.next().ok_or_else(|| {
            BankError::CoreError(
                "the operation did not return the required number of elements".to_owned(),
            )
        })?;
        let to = accounts.next().ok_or_else(|| {
            BankError::CoreError(
                "the operation did not return the required number of elements".to_owned(),
            )
        })?;
        Ok((from, to))
    }
}

//реализация State для банка в памяти
#[derive(Debug, Default)]
pub struct InMemoryState(HashMap<AccountId, Account>);

impl<'a> InMemoryState {
    fn push_to_col(
        &'a mut self,
        op: &'a Operation,
    ) -> Result<(&'a AccountId, &'a Account), BankError> {
        let account_id: &AccountId = match op {
            Operation::Create(account_id) if self.0.contains_key(account_id) => {
                return Err(BankError::BadRequest(format!(
                    "Bank already contains account_id[{}]",
                    account_id
                )));
            }

            Operation::Create(account_id) => {
                let new_account = Account {
                    account_id: *account_id,
                    balance: 0_u32,
                };
                self.0.insert(*account_id, new_account);
                account_id
            }

            Operation::Deposit(account_id, money) if self.0.contains_key(account_id) => {
                let account: &mut Account = self.0.get_mut(account_id).unwrap();
                account.balance += money.get();
                account_id
            }

            Operation::Withdraw(account_id, money) if self.0.contains_key(account_id) => {
                let account: &mut Account = self.0.get_mut(account_id).unwrap();
                if account.balance >= money.get() {
                    account.balance -= money.get();
                } else {
                    return Err(BankError::BadRequest("Insufficient funds".to_string()));
                }
                account_id
            }

            Operation::Withdraw(account_id, _) | Operation::Deposit(account_id, _) => {
                return Err(BankError::BadRequest(format!(
                    "Bank doesnt contain account_id[{}]",
                    account_id
                )))
            }
        };

        Ok(
            self.0
                .get(account_id)
                .map(|account| (account_id, account))
                .expect("something is wrong with your code"), //здесь expect потому, что, если такого аккаунта нет, то это ошибка в коде
        )
    }
}

impl State for InMemoryState {
    fn get_balance(&self, account_id: &AccountId) -> Result<&Account, BankError> {
        self.0.get(account_id).ok_or_else(|| {
            BankError::BadRequest(format!("Account[{}] not found in bank", account_id))
        })
    }

    fn transact<'a, 'b>(
        &'a mut self,
        ops: impl Iterator<Item = &'b Operation>,
    ) -> Result<impl Iterator<Item = &'a Account> + 'a, BankError> {
        let mut successful_accounts_ids = Vec::new();

        // Фаза 1: модификация данных
        for op in ops {
            match self.push_to_col(op) {
                Ok((account_id, _)) => successful_accounts_ids.push(*account_id), // Сохраняем ID аккаунтов
                Err(err) => return Err(err),
            }
        }

        // Фаза 2: чтение данных (поиск по ID, который мы сохранили)
        Ok(successful_accounts_ids.into_iter().map(|account_id| {
            self.0
                .get(&account_id)
                .expect("something is wrong with your code")
        }))
    }
}

//реализация хранилища операций банка в пмяти
#[derive(Debug)]
pub struct InMemoryOpsStorage {
    cur_key: OpId,
    by_ops_storage: BTreeMap<OpId, (AccountId, Operation)>,
    by_acc_storage: HashMap<AccountId, LinkedList<OpId>>,
}

impl Default for InMemoryOpsStorage {
    fn default() -> Self {
        Self {
            cur_key: OpId::MIN,
            by_ops_storage: BTreeMap::default(),
            by_acc_storage: HashMap::default(),
        }
    }
}

impl InMemoryOpsStorage {
    fn push_to_cols(&mut self, op: Operation) -> OpId {
        let new_key = self.cur_key.checked_add(1).unwrap();

        self.cur_key = new_key;

        let account_id = *op.account_id();

        self.by_ops_storage.insert(new_key, (account_id, op));

        self.by_acc_storage
            .entry(account_id)
            .and_modify(|list| list.push_back(new_key))
            .or_insert({
                let mut tmp = LinkedList::new();
                tmp.push_front(new_key);
                tmp
            });

        new_key
    }
}

impl OpsStorage for InMemoryOpsStorage {
    fn get_history(&self) -> Result<impl Iterator<Item = (OpId, &Operation)>, BankError> {
        Ok(self
            .by_ops_storage
            .iter()
            .map(|(op_id, (_, operation))| (*op_id, operation)))
    }

    fn get_ops(
        &self,
        account_id: &AccountId,
    ) -> Result<impl Iterator<Item = (OpId, &Operation)>, BankError> {
        self.by_acc_storage
                .get(account_id)//O(1)
                .map(|list| {
                    list.iter().map(|op_id|{ //O(N)
                        let (_, op)=self.by_ops_storage.get(op_id)//O(lgN)
                        .unwrap_or_else(||panic!("something get wrong with your code, because by_ops_storage doesn't contain value for op_id[{}]",op_id));
                        (*op_id,op)
                    })
                }).ok_or_else(||BankError::BadRequest(format!("There is no account[{}] in the bank",account_id)))
    }

    fn transact(
        &mut self,
        ops: impl Iterator<Item = Operation>,
    ) -> Result<impl Iterator<Item = (OpId, &Operation)>, BankError> {
        let mut vec = Vec::new();
        for op in ops {
            let op_id = self.push_to_cols(op);
            vec.push(op_id);
        }

        Ok(vec.into_iter().map(|op_id| {
            let (_, op) = self.by_ops_storage.get(&op_id).unwrap_or_else(|| {
                panic!(
                    "something wrong with your code, by_ops_storage should contain op_id[{}]",
                    op_id
                )
            });
            (op_id, op)
        }))
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn bank_should_create_account() {
        let mut bank: Bank<InMemoryOpsStorage, InMemoryState> =
            Bank::new(InMemoryOpsStorage::default(), InMemoryState::default());

        let acc_1 = 128;
        let acc_2 = 129;

        let ret = Bank::create_account(&mut bank, acc_1);
        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_1.clone(),
                balance: 0_u32
            })
        );

        let ret = Bank::create_account(&mut bank, acc_2);
        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_2.clone(),
                balance: 0_u32
            })
        );
        drop(ret);
    }

    #[test]
    fn bank_should_deposit_funds() {
        let mut bank: Bank<InMemoryOpsStorage, InMemoryState> =
            Bank::new(InMemoryOpsStorage::default(), InMemoryState::default());

        let acc_1 = 128;
        let acc_2 = 129;

        let _ = bank.create_account(acc_1);
        let _ = bank.create_account(acc_2);

        let ret: Result<&Account, BankError> = bank.deposit(&acc_1, NonZeroMoney::new(42).unwrap());

        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_1.clone(),
                balance: 42
            })
        );

        let ret: Result<&Account, BankError> = bank.deposit(&acc_2, NonZeroMoney::new(42).unwrap());

        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_2.clone(),
                balance: 42
            })
        );

        let ret: Result<&Account, BankError> = //acc_1 again
            bank.deposit(&acc_1, NonZeroMoney::new(42).unwrap());

        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_1.clone(),
                balance: 84
            })
        );
    }

    #[test]
    fn bank_should_withdraw_funds() {
        let mut bank: Bank<InMemoryOpsStorage, InMemoryState> =
            Bank::new(InMemoryOpsStorage::default(), InMemoryState::default());

        let acc_1 = 128;
        let acc_2 = 129;

        let _ = bank.create_account(acc_1);
        let _ = bank.create_account(acc_2);

        let ret: Result<&Account, BankError> = bank.deposit(&acc_1, NonZeroMoney::new(42).unwrap());
        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_1.clone(),
                balance: 42
            })
        );

        let ret: Result<&Account, BankError> = bank.deposit(&acc_2, NonZeroMoney::new(42).unwrap());
        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_2.clone(),
                balance: 42
            })
        );

        let ret: Result<&Account, BankError> = bank.withdraw(acc_1, NonZeroMoney::MIN);
        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_1.clone(),
                balance: 41
            })
        );

        let ret: Result<&Account, BankError> = bank.withdraw(acc_1, NonZeroMoney::new(41).unwrap());
        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_1.clone(),
                balance: 0
            })
        );

        let ret: Result<&Account, BankError> = bank.withdraw(acc_2, NonZeroMoney::MIN);
        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_2.clone(),
                balance: 41
            })
        );

        let ret: Result<&Account, BankError> = bank.withdraw(acc_2, NonZeroMoney::MAX);
        assert_eq!(
            ret,
            Err(BankError::BadRequest("Insufficient funds".to_string()))
        );
    }

    #[test]
    fn bank_should_move_funds() {
        let mut bank: Bank<InMemoryOpsStorage, InMemoryState> =
            Bank::new(InMemoryOpsStorage::default(), InMemoryState::default());

        let acc_1 = 128;
        let acc_2 = 129;
        let acc_3 = 130;

        let _ = bank.create_account(acc_1);
        let _ = bank.create_account(acc_2);
        let _ = bank.create_account(acc_3);

        let ret: Result<&Account, BankError> = bank.deposit(&acc_1, NonZeroMoney::new(42).unwrap());
        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_1,
                balance: 42
            })
        );
        //acc_2 is untouched
        let ret: Result<&Account, BankError> = bank.deposit(&acc_3, NonZeroMoney::new(21).unwrap());
        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_3,
                balance: 21
            })
        );

        let (from, to) = bank
            .move_money(acc_1, acc_2, NonZeroMoney::new(42).unwrap())
            .expect("should be Ok((from,to))");
        assert_eq!(
            from,
            &Account {
                account_id: acc_1,
                balance: 0
            }
        );
        assert_eq!(
            to,
            &Account {
                account_id: acc_2.clone(),
                balance: 42
            }
        );

        let ret: &Account = bank
            .get_balance(&acc_3)
            .expect("should be Ok(Account for acc_3)");
        assert_eq!(
            ret,
            &Account {
                account_id: acc_3.clone(),
                balance: 21
            }
        );

        let ret: &Account = bank
            .get_balance(&acc_2)
            .expect("should be Ok(Account for acc_2)");
        assert_eq!(
            ret,
            &Account {
                account_id: acc_2.clone(),
                balance: 42
            }
        );

        let ret: &Account = bank
            .get_balance(&acc_1)
            .expect("should be Ok(Account for acc_1)");
        assert_eq!(
            ret,
            &Account {
                account_id: acc_1.clone(),
                balance: 0
            }
        );
    }

    #[test]
    fn bank_should_get_balance() {
        let mut bank: Bank<InMemoryOpsStorage, InMemoryState> =
            Bank::new(InMemoryOpsStorage::default(), InMemoryState::default());

        let acc_1 = 128;
        let acc_2 = 129;
        let acc_3 = 130;

        let _ = bank.create_account(acc_1);
        let _ = bank.create_account(acc_2);
        let _ = bank.create_account(acc_3);

        let ret: Result<&Account, BankError> = bank.deposit(&acc_1, NonZeroMoney::new(42).unwrap());
        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_1.clone(),
                balance: 42
            })
        );

        let ret: Result<&Account, BankError> = bank.deposit(&acc_3, NonZeroMoney::new(21).unwrap());
        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_3.clone(),
                balance: 21
            })
        );

        let ret = bank.get_balance(&acc_1).expect("should be Account 1");
        assert_eq!(
            ret,
            &Account {
                account_id: acc_1.clone(),
                balance: 42
            }
        );
        let ret = bank.get_balance(&acc_2).expect("should be Account 2");
        assert_eq!(
            ret,
            &Account {
                account_id: acc_2.clone(),
                balance: 0
            }
        );
        let ret = bank.get_balance(&acc_3).expect("should be Account 3");
        assert_eq!(
            ret,
            &Account {
                account_id: acc_3.clone(),
                balance: 21
            }
        );
    }

    #[test]
    fn bank_should_get_history() {
        let mut bank: Bank<InMemoryOpsStorage, InMemoryState> =
            Bank::new(InMemoryOpsStorage::default(), InMemoryState::default());

        let acc_1 = 128;
        let acc_2 = 129;
        let acc_3 = 130;

        let _ = bank.create_account(acc_1);
        let _ = bank.create_account(acc_2);
        let _ = bank.create_account(acc_3);

        let ret: Result<&Account, BankError> = bank.deposit(&acc_1, NonZeroMoney::new(42).unwrap());
        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_1.clone(),
                balance: 42
            })
        );

        let ret: Result<&Account, BankError> = bank.deposit(&acc_3, NonZeroMoney::new(21).unwrap());
        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_3.clone(),
                balance: 21
            })
        );

        let history = bank.get_history().expect("Bank should get history");

        let clone_of_bank: Bank<InMemoryOpsStorage, InMemoryState> = Bank::from(history);

        let ret = clone_of_bank.get_balance(&acc_1);
        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_1.clone(),
                balance: 42
            })
        );

        let ret = clone_of_bank.get_balance(&acc_2);
        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_2.clone(),
                balance: 0
            })
        );

        let ret = clone_of_bank.get_balance(&acc_3);
        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_3.clone(),
                balance: 21
            })
        );
    }

    #[test]
    fn bank_should_get_account_ops() {
        let mut bank: Bank<InMemoryOpsStorage, InMemoryState> =
            Bank::new(InMemoryOpsStorage::default(), InMemoryState::default());

        let acc_1 = 128;
        let acc_2 = 129;

        let _ = bank.create_account(acc_1);
        let _ = bank.create_account(acc_2);

        let ret: Result<&Account, BankError> = bank.deposit(&acc_1, NonZeroMoney::new(42).unwrap());
        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_1.clone(),
                balance: 42
            })
        );

        let ret: Result<&Account, BankError> = bank.deposit(&acc_2, NonZeroMoney::new(21).unwrap());
        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_2.clone(),
                balance: 21
            })
        );

        let _ = bank
            .move_money(acc_1, acc_2, NonZeroMoney::new(42).unwrap())
            .expect("move should work");

        //let _: Vec<(_, _)> = iter.collect();

        let ret: Result<&Account, BankError> = bank.get_balance(&acc_2);
        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_2.clone(),
                balance: 63
            })
        );

        let ret: Result<&Account, BankError> = bank.get_balance(&acc_1);
        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_1.clone(),
                balance: 0
            })
        );

        let ret = bank.get_account_ops(&acc_1);
        assert!(ret.is_ok());
        assert_eq!(ret.unwrap().count(), 3); //Create + Deposit + Withdraw

        let ret = bank.get_account_ops(&acc_2);
        assert!(ret.is_ok());
        assert_eq!(ret.unwrap().count(), 3); //Create + Deposit + Deposit
    }
}
