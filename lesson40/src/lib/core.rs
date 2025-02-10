use std::{
    collections::{BTreeMap, HashMap, LinkedList},
    num::{NonZeroU128, NonZeroU32},
};

pub type AccountId = String;
pub type Money = u32;
pub type NonZeroMoney = NonZeroU32;
pub type Err = String;
pub type OpId = NonZeroU128;

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
    Move {
        from: AccountId,
        to: AccountId,
        amount: NonZeroMoney,
    },

    GetBalance(AccountId),
}

impl Operation {
    fn account_id(&self) -> &AccountId {
        match self {
            Operation::Create(account_id) => account_id,
            Operation::Deposit(account_id, _) | Self::Withdraw(account_id, _) => account_id,
            other => panic!(
                "your code should not get use other operations[{:?}] for states",
                other
            ),
        }
    }
}

///Банк имеет хранилище операций по счетам клиентов
pub trait OpsStorage {
    fn transact(
        &mut self,
        ops: impl Iterator<Item = Operation>,
    ) -> Result<impl Iterator<Item = (OpId, &Operation)>, Err>;

    fn persist(&mut self, op: Operation) -> Result<(OpId, &Operation), Err> {
        self.transact(std::iter::once(op)).and_then(|mut iter| {
            iter.next()
                .map(|(opt_id, op)| (opt_id, op))
                .ok_or("a transaction should return at least one operation".to_owned())
        })
    }
    //should be O(N), where N - account ops
    fn get_ops<'a, 'b>(
        &'a self,
        account_id: &'b AccountId,
    ) -> Result<impl Iterator<Item = (OpId, &'a Operation)>, Err>;
    //should be O(M), where M - all ops
    fn get_history(&self) -> Result<impl Iterator<Item = (OpId, &Operation)>, Err>;
}

///Банк имеет текущее "состояние" счетов клиентов
pub trait State {
    fn transact<'a, 'b>(
        &'a mut self,
        ops: impl Iterator<Item = &'b Operation>,
    ) -> Result<impl Iterator<Item = &'a Account> + 'a, Err>;

    fn update<'a, 'b>(&'a mut self, op: &'b Operation) -> Result<&'a Account, Err> {
        self.transact(std::iter::once(op)).and_then(|mut iter| {
            iter.next()
                .map(|acc| acc)
                .ok_or("a transaction should return at least one account".to_owned())
        })
    }
    //should be O(N), where N - account ops
    fn get_balance(&self, account_id: &AccountId) -> Result<&Account, Err>;
}

pub struct Bank<T: OpsStorage, S: State> {
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
        let mut bank = Bank {
            storage: T::default(),
            state: S::default(),
        };

        for (_, op) in history {
            let (_, op) = bank
                .storage
                .persist(op.clone())
                .expect(format!("something wrong with history operation[{:?}]", op).as_str());

            let ret = bank.state.update(op);

            if let Some(err) = ret.err() {
                println!("History operation has an error[{:?}]", err)
            }
        }

        bank
    }

    //создание аккаунта
    pub fn create_account(&mut self, account_id: &AccountId) -> Result<&Account, Err> {
        let op = Operation::Create(account_id.clone());
        let (_, op) = self.storage.persist(op)?;
        self.state.update(op)
    }

    //Клиент может получить свой баланс.
    pub fn get_balance(&self, account_id: &AccountId) -> Result<&Account, Err> {
        self.state.get_balance(account_id)
    }

    //история операций по счету
    pub fn get_account_ops<'a, 'b>(
        &'a self,
        account_id: &'b AccountId,
    ) -> Result<impl Iterator<Item = (OpId, &'a Operation)> + 'b, String>
    where
        'a: 'b,
    {
        self.storage.get_ops(account_id)
    }
    //можно получить историю операций
    pub fn get_history(&self) -> Result<impl Iterator<Item = (OpId, &Operation)>, String> {
        self.storage.get_history()
    }

    //Клиент может пополнить свой баланс.
    pub fn deposit(
        &mut self,
        account_id: &AccountId,
        money: NonZeroMoney,
    ) -> Result<&Account, Err> {
        let op = Operation::Deposit(account_id.clone(), money);
        let (_, op) = self.storage.persist(op)?;
        self.state.update(op)
    }

    //Клиент может забрать деньги
    pub fn withdraw(
        &mut self,
        account_id: &AccountId,
        money: NonZeroMoney,
    ) -> Result<&Account, Err> {
        let op = Operation::Withdraw(account_id.clone(), money);
        let (_, op) = self.storage.persist(op)?;
        self.state.update(op)
    }

    //перемещение денег от счета на счет
    pub fn move_money(
        &mut self,
        from: &AccountId,
        to: &AccountId,
        money: NonZeroMoney,
    ) -> Result<impl Iterator<Item = &Account>, Err> {
        if from.eq(to.as_str()) {
            return Err(format!("Sending funds to yourself is prohibited"));
        }

        let ops = vec![
            Operation::Withdraw(from.clone(), money),
            Operation::Deposit(to.clone(), money),
        ]
        .into_iter();
        //first save to storage
        let ops = self.storage.transact(ops)?.map(|(_, op)| op);

        //and then update state
        self.state.transact(ops)
    }
}

//реализация State для банка в памяти
#[derive(Debug, Default)]
pub struct InMemoryState(HashMap<AccountId, Account>);

impl<'a> InMemoryState {
    fn push_to_col(&'a mut self, op: &'a Operation) -> Result<(&'a AccountId, &'a Account), Err> {
        let account_id: &AccountId = match op {
            Operation::Create(account_id) if self.0.contains_key(account_id) => {
                return Err(format!("Bank already contains account_id[{}]", account_id));
            }

            Operation::Create(account_id) => {
                let new_account = Account {
                    account_id: account_id.clone(),
                    balance: 0_u32,
                };
                self.0.insert(account_id.clone(), new_account);
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
                    return Err("Insufficient funds".to_string());
                }
                account_id
            }

            Operation::Withdraw(account_id, _) | Operation::Deposit(account_id, _) => {
                return Err(format!("Bank doesnt contain account_id[{}]", account_id))
            }

            other => {
                panic!(
                    "your code should not put other operations[{:?}] into state",
                    other
                );
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
    fn get_balance(&self, account_id: &AccountId) -> Result<&Account, Err> {
        self.0
            .get(account_id)
            .ok_or(format!("Account[{}] not found in bank", account_id))
    }

    fn transact<'a, 'b>(
        &'a mut self,
        ops: impl Iterator<Item = &'b Operation>,
    ) -> Result<impl Iterator<Item = &'a Account> + 'a, Err> {
        let mut successful_accounts_ids = Vec::new();

        // Фаза 1: модификация данных
        for op in ops {
            match self.push_to_col(op) {
                Ok((account_id, _)) => successful_accounts_ids.push(account_id.clone()), // Сохраняем ID аккаунтов
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

        let account_id = op.account_id().clone();

        self.by_ops_storage
            .insert(new_key, (account_id.clone(), op));

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
    fn get_history(&self) -> Result<impl Iterator<Item = (OpId, &Operation)>, Err> {
        Ok(self
            .by_ops_storage
            .iter()
            .map(|(op_id, (_, operation))| (op_id.clone(), operation)))
    }

    fn get_ops(
        &self,
        account_id: &AccountId,
    ) -> Result<impl Iterator<Item = (OpId, &Operation)>, Err> {
        self.by_acc_storage
                .get(account_id)//O(1)
                .map(|list| {
                    list.iter().map(|op_id|{ //O(N)
                        let (_, op)=self.by_ops_storage.get(op_id)//O(lgN)
                        .expect(format!("something get wrong with your code, bsc by_ops_storage doesn't contain value for op_id[{}]",op_id).as_str());
                        (op_id.clone(),op)
                    })
                }).ok_or(format!("There is no account[{}] in the bank",account_id))
    }

    fn transact(
        &mut self,
        ops: impl Iterator<Item = Operation>,
    ) -> Result<impl Iterator<Item = (OpId, &Operation)>, Err> {
        let mut vec = Vec::new();
        for op in ops {
            let op_id = self.push_to_cols(op);
            vec.push(op_id);
        }

        Ok(vec.into_iter().map(|op_id| {
            let (_, op) = self.by_ops_storage.get(&op_id).expect(
                format!(
                    "something wrong with your code, by_ops_storage should contain op_id[{}]",
                    op_id
                )
                .as_str(),
            );
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

        let acc_1 = "Acc_1".to_string();
        let acc_2 = "Acc_2".to_string();

        let ret = Bank::create_account(&mut bank, &acc_1);
        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_1.clone(),
                balance: 0_u32
            })
        );

        let ret = Bank::create_account(&mut bank, &acc_2);
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

        let acc_1 = "Acc_1".to_string();
        let acc_2 = "Acc_2".to_string();

        let _ = bank.create_account(&acc_1);
        let _ = bank.create_account(&acc_2);

        let ret: Result<&Account, String> = bank.deposit(&acc_1, NonZeroMoney::new(42).unwrap());

        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_1.clone(),
                balance: 42
            })
        );

        let ret: Result<&Account, String> = bank.deposit(&acc_2, NonZeroMoney::new(42).unwrap());

        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_2.clone(),
                balance: 42
            })
        );

        let ret: Result<&Account, String> = //acc_1 again
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

        let acc_1 = "Acc_1".to_string();
        let acc_2 = "Acc_2".to_string();

        let _ = bank.create_account(&acc_1);
        let _ = bank.create_account(&acc_2);

        let ret: Result<&Account, String> = bank.deposit(&acc_1, NonZeroMoney::new(42).unwrap());
        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_1.clone(),
                balance: 42
            })
        );

        let ret: Result<&Account, String> = bank.deposit(&acc_2, NonZeroMoney::new(42).unwrap());
        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_2.clone(),
                balance: 42
            })
        );

        let ret: Result<&Account, String> = bank.withdraw(&acc_1, NonZeroMoney::MIN);
        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_1.clone(),
                balance: 41
            })
        );

        let ret: Result<&Account, String> = bank.withdraw(&acc_1, NonZeroMoney::new(41).unwrap());
        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_1.clone(),
                balance: 0
            })
        );

        let ret: Result<&Account, String> = bank.withdraw(&acc_2, NonZeroMoney::MIN);
        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_2.clone(),
                balance: 41
            })
        );

        let ret: Result<&Account, String> = bank.withdraw(&acc_2, NonZeroMoney::MAX);
        assert_eq!(ret, Err("Insufficient funds".to_string()));
    }

    #[test]
    fn bank_should_move_funds() {
        let mut bank: Bank<InMemoryOpsStorage, InMemoryState> =
            Bank::new(InMemoryOpsStorage::default(), InMemoryState::default());

        let acc_1 = "Acc_1".to_string();
        let acc_2 = "Acc_2".to_string();
        let acc_3 = "Acc_3".to_string();

        let _ = bank.create_account(&acc_1);
        let _ = bank.create_account(&acc_2);
        let _ = bank.create_account(&acc_3);

        let ret: Result<&Account, String> = bank.deposit(&acc_1, NonZeroMoney::new(42).unwrap());
        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_1.clone(),
                balance: 42
            })
        );
        //acc_2 is untouched
        let ret: Result<&Account, String> = bank.deposit(&acc_3, NonZeroMoney::new(21).unwrap());
        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_3.clone(),
                balance: 21
            })
        );

        let mut iter = bank
            .move_money(&acc_1, &acc_2, NonZeroMoney::new(42).unwrap())
            .expect("should be Ok(iter)");

        let first = iter.next().expect("should be the acc_1 with their Account");
        let second = iter.next().expect("should be the acc_2 with their Account");
        assert_eq!(
            first,
            &Account {
                account_id: acc_1.clone(),
                balance: 0
            }
        );
        assert_eq!(
            second,
            &Account {
                account_id: acc_2.clone(),
                balance: 42
            }
        );
        let _: Vec<_> = iter.collect(); //drain iter

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

        let acc_1 = "Acc_1".to_string();
        let acc_2 = "Acc_2".to_string();
        let acc_3 = "Acc_3".to_string();

        let _ = bank.create_account(&acc_1);
        let _ = bank.create_account(&acc_2);
        let _ = bank.create_account(&acc_3);

        let ret: Result<&Account, String> = bank.deposit(&acc_1, NonZeroMoney::new(42).unwrap());
        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_1.clone(),
                balance: 42
            })
        );

        let ret: Result<&Account, String> = bank.deposit(&acc_3, NonZeroMoney::new(21).unwrap());
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

        let acc_1 = "Acc_1".to_string();
        let acc_2 = "Acc_2".to_string();
        let acc_3 = "Acc_3".to_string();

        let _ = bank.create_account(&acc_1);
        let _ = bank.create_account(&acc_2);
        let _ = bank.create_account(&acc_3);

        let ret: Result<&Account, String> = bank.deposit(&acc_1, NonZeroMoney::new(42).unwrap());
        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_1.clone(),
                balance: 42
            })
        );

        let ret: Result<&Account, String> = bank.deposit(&acc_3, NonZeroMoney::new(21).unwrap());
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

        let acc_1 = "Acc_1".to_string();
        let acc_2 = "Acc_2".to_string();

        let _ = bank.create_account(&acc_1);
        let _ = bank.create_account(&acc_2);

        let ret: Result<&Account, String> = bank.deposit(&acc_1, NonZeroMoney::new(42).unwrap());
        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_1.clone(),
                balance: 42
            })
        );

        let ret: Result<&Account, String> = bank.deposit(&acc_2, NonZeroMoney::new(21).unwrap());
        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_2.clone(),
                balance: 21
            })
        );

        let _ = bank
            .move_money(&acc_1, &acc_2, NonZeroMoney::new(42).unwrap())
            .expect("move should work");

        //let _: Vec<(_, _)> = iter.collect();

        let ret: Result<&Account, String> = bank.get_balance(&acc_2);
        assert_eq!(
            ret,
            Ok(&Account {
                account_id: acc_2.clone(),
                balance: 63
            })
        );

        let ret: Result<&Account, String> = bank.get_balance(&acc_1);
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
