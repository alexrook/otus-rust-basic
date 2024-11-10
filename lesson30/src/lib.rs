use std::{
    collections::{BTreeMap, HashMap, LinkedList},
    num::{NonZeroU128, NonZeroU32},
};

type AccountId = String;
type Money = u32;
type NonZeroMoney = NonZeroU32;
type Err = String;
type OpId = NonZeroU128;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Account {
    pub balance: Money,
}

#[derive(Debug, Clone)]
pub enum Operation {
    /**
     * Перед выполнением любых операций по счёту, его необходимо создать.
     * Если счёт с именем Х существует, то создание нового счёта с именем Х - ошибка
     */
    Create(AccountId), //регистрация счёта

    /**Пополнение увеличивает количество денег на счете на указанную сумму.
     * Пополнение на ноль денежных единиц - ошибка.
     */
    Charge(NonZeroMoney), //пополнение
    /**
     * Снятие уменьшает количество денег на счете на указанную сумму.
     * Снятие нуля денежных единиц - ошибка.
     * Попытка снять больше чем есть на счете - ошибка.
     */
    Withdraw(NonZeroMoney), //снятие

                            // Перевод уменьшает баланс отправителя и увеличивает баланс получателя на указанную сумму.
                            //   Перевод нуля денежных единиц - ошибка.
                            //   Перевод самому себе - ошибка.
                            // Если сумма перевода больше баланса отправителя - ошибка.
}

//  Каждая операция (регистрация счёта, пополнение, снятие, перевод) должна сохраняться.
//  Каждая успешная операция возвращает уникальный идентификатор,
//  по которому данные об этой операции могут быть в дальнейшем запрошены.
//  Можно получить всю историю операций.
//  Можно получить историю операций связанных с конкретным счётом.
//  Операции должны храниться в порядке их выполнения.
//  Есть возможность восстановить состояние счетов,
//  повторно выполнив все операции из истории в новом экземпляре банка.
//  После этого новый экземпляр банка должен совпадать с тем,
//  историю которого мы использовали
pub trait OpsStorage {
    fn transact(
        &mut self,
        ops: impl Iterator<Item = (AccountId, Operation)>,
    ) -> Result<impl Iterator<Item = (OpId, AccountId, &Operation)>, Err>;

    fn persist(&mut self, account_id: AccountId, op: Operation) -> Result<(OpId, &Operation), Err> {
        self.transact(vec![(account_id, op)].into_iter())
            .and_then(|mut iter| {
                iter.next()
                    .map(|(opt_id, _, op)| (opt_id, op))
                    .ok_or("a transaction should return at least one operation".to_owned())
            })
    }
    //should be O(N), where N - account ops
    fn get_ops(
        &self,
        account_id: &AccountId,
    ) -> Result<impl Iterator<Item = (OpId, &Operation)>, Err>;
    //should be O(M), where M - all ops
    fn get_history(&self) -> Result<impl Iterator<Item = (OpId, (AccountId, &Operation))>, Err>;
}

pub trait State {
    fn transact<'a, 'b>(
        &'a mut self,
        ops: impl Iterator<Item = (AccountId, &'b Operation)>,
    ) -> Result<impl Iterator<Item = (AccountId, &'a Account)>, Err>;

    fn update<'a, 'b>(
        &'a mut self,
        account_id: AccountId,
        op: &'b Operation,
    ) -> Result<&'a Account, Err> {
        self.transact(vec![(account_id, op)].into_iter())
            .and_then(|mut iter| {
                iter.next()
                    .map(|(_, acc)| acc)
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
        Bank {
            storage: storage,
            state: state,
        }
    }
}

impl<T: OpsStorage, S: State> Bank<T, S> {
    //создание аккаунта
    pub fn create_account(&mut self, account_id: AccountId) -> Result<&Account, Err> {
        let op = Operation::Create(account_id.clone());
        let (_, op) = self.storage.persist(account_id.clone(), op)?;
        self.state.update(account_id, op)
    }

    //Клиент может получить свой баланс.
    pub fn get_balance(&self, account_id: &AccountId) -> Result<&Account, Err> {
        self.state.get_balance(account_id)
    }

    //Клиент может пополнить свой баланс.
    pub fn deposit(&mut self, account_id: AccountId, money: NonZeroMoney) -> Result<&Account, Err> {
        let op = Operation::Charge(money);
        let (_, op) = self.storage.persist(account_id.clone(), op)?;
        self.state.update(account_id, op)
    }

    //Клиент может забрать деньги
    pub fn withdraw(
        &mut self,
        account_id: AccountId,
        money: NonZeroMoney,
    ) -> Result<&Account, Err> {
        let op = Operation::Withdraw(money);
        let (_, op) = self.storage.persist(account_id.clone(), op)?;
        self.state.update(account_id, op)
    }

    //перемещение денег от счета на счет
    pub fn move_money(
        &mut self,
        from: AccountId,
        to: AccountId,
        money: NonZeroMoney,
    ) -> Result<impl Iterator<Item = (AccountId, &Account)>, Err> {
        let ops = vec![
            (from, Operation::Withdraw(money)),
            (to, Operation::Charge(money)),
        ]
        .into_iter();
        //first save to storage
        let ops = self
            .storage
            .transact(ops)?
            .map(|(_, account_id, op)| (account_id, op));

        //and then update state
        self.state.transact(ops)
    }
}

#[derive(Debug, Default)]
pub struct InMemoryState(HashMap<AccountId, Account>);

impl<'a> InMemoryState {
    fn push_to_col(
        &'a mut self,
        account_id: AccountId,
        op: &'a Operation,
    ) -> Result<(AccountId, &'a Account), Err> {
        match op {
            Operation::Create(_) if self.0.contains_key(&account_id) => {
                return Err(format!("Bank already contains account_id[{}]", account_id));
            }

            Operation::Create(_) => {
                let new_account = Account { balance: 0_u32 };
                self.0.insert(account_id.clone(), new_account);
            }

            Operation::Charge(money) if self.0.contains_key(&account_id) => {
                // self.0
                //     .entry(account_id) //поглощает ключ
                //     .and_modify(|account| account.balance += money.get());
                let account: &mut Account = self.0.get_mut(&account_id).unwrap();
                account.balance += money.get()
            }

            Operation::Withdraw(money) if self.0.contains_key(&account_id) => {
                let account: &mut Account = self.0.get_mut(&account_id).unwrap();
                if account.balance >= money.get() {
                    account.balance -= money.get()
                } else {
                    return Err("Insufficient funds".to_string());
                }
            }

            _ => return Err(format!("Bank doesnt contain account_id[{}]", account_id)),
        }

        Ok(
            self.0
                .get(&account_id)
                .map(|account| (account_id, account))
                .expect("err"), //здесь expect потому, что, если такого аккаунта нет, то это ошибка в коде
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
        ops: impl Iterator<Item = (AccountId, &'b Operation)>,
    ) -> Result<impl Iterator<Item = (AccountId, &'a Account)>, Err> {
        let mut vec = Vec::new();
        for (account_id, op) in ops {
            let (account_id, _) = self.push_to_col(account_id, op)?;
            vec.push(account_id);
        }

        Ok(vec.into_iter().map(|account_id| {
            let account = self.0.get(&account_id).expect("err");
            (account_id, account)
        }))
    }
}

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
    fn push_to_cols(&mut self, account_id: AccountId, op: Operation) -> OpId {
        let new_key = self.cur_key.checked_add(1).unwrap();
        self.cur_key = new_key;
        self.by_ops_storage
            .insert(new_key, (account_id.clone(), op));

        self.by_acc_storage
            .entry(account_id.clone())
            .and_modify(|list| list.push_back(new_key));

        new_key
    }
}

impl OpsStorage for InMemoryOpsStorage {
    fn get_history(&self) -> Result<impl Iterator<Item = (OpId, (AccountId, &Operation))>, Err> {
        Ok(self
            .by_ops_storage
            .iter()
            .map(|(op_id, (account_id, operation))| {
                (op_id.clone(), (account_id.clone(), operation))
            }))
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
        ops: impl Iterator<Item = (AccountId, Operation)>,
    ) -> Result<impl Iterator<Item = (OpId, AccountId, &Operation)>, Err> {
        let mut vec = Vec::new();
        for (account_id, op) in ops {
            let op_id = self.push_to_cols(account_id, op);
            vec.push(op_id);
        }

        Ok(vec.into_iter().map(|op_id| {
            let (account_id, op) = self.by_ops_storage.get(&op_id).expect(
                format!(
                    "something wrong with your code, by_ops_storage should contain op_id[{}]",
                    op_id
                )
                .as_str(),
            );
            (op_id, account_id.clone(), op)
        }))
    }
}

mod test {

    use super::*;

    #[test]
    fn bank_should_create_account() {
        let mut bank: Bank<InMemoryOpsStorage, InMemoryState> =
            Bank::new(InMemoryOpsStorage::default(), InMemoryState::default());

        let acc_1 = "Acc_1".to_string();
        let acc_2 = "Acc_2".to_string();

        let ret = Bank::create_account(&mut bank, acc_1);
        assert_eq!(ret, Ok(&Account { balance: 0_u32 }));

        let ret = Bank::create_account(&mut bank, acc_2);
        assert_eq!(ret, Ok(&Account { balance: 0_u32 }));
        drop(ret);
    }

    #[test]
    fn bank_should_deposit_funds() {
        let mut bank: Bank<InMemoryOpsStorage, InMemoryState> =
            Bank::new(InMemoryOpsStorage::default(), InMemoryState::default());

        let acc_1 = "Acc_1".to_string();
        let acc_2 = "Acc_2".to_string();

        let _ = bank.create_account(acc_1.clone());
        let _ = bank.create_account(acc_2.clone());

        let ret: Result<&Account, String> =
            bank.deposit(acc_1.clone(), NonZeroMoney::MIN.checked_add(41).unwrap());

        assert_eq!(ret, Ok(&Account { balance: 42 }));

        let ret: Result<&Account, String> =
            bank.deposit(acc_2.clone(), NonZeroMoney::MIN.checked_add(41).unwrap());

        assert_eq!(ret, Ok(&Account { balance: 42 }));

        let ret: Result<&Account, String> = //acc_1 again
            bank.deposit(acc_1.clone(), NonZeroMoney::MIN.checked_add(41).unwrap());

        assert_eq!(ret, Ok(&Account { balance: 84 }));
    }

    #[test]
    fn bank_should_withdraw_funds() {
        let mut bank: Bank<InMemoryOpsStorage, InMemoryState> =
            Bank::new(InMemoryOpsStorage::default(), InMemoryState::default());

        let acc_1 = "Acc_1".to_string();
        let acc_2 = "Acc_2".to_string();

        let _ = bank.create_account(acc_1.clone());
        let _ = bank.create_account(acc_2.clone());

        let ret: Result<&Account, String> =
            bank.deposit(acc_1.clone(), NonZeroMoney::MIN.checked_add(41).unwrap());
        assert_eq!(ret, Ok(&Account { balance: 42 }));

        let ret: Result<&Account, String> =
            bank.deposit(acc_2.clone(), NonZeroMoney::MIN.checked_add(41).unwrap());
        assert_eq!(ret, Ok(&Account { balance: 42 }));

        let ret: Result<&Account, String> = bank.withdraw(acc_1.clone(), NonZeroMoney::MIN);
        assert_eq!(ret, Ok(&Account { balance: 41 }));

        let ret: Result<&Account, String> =
            bank.withdraw(acc_1.clone(), NonZeroMoney::new(41).unwrap());
        assert_eq!(ret, Ok(&Account { balance: 0 }));

        let ret: Result<&Account, String> = bank.withdraw(acc_2.clone(), NonZeroMoney::MIN);
        assert_eq!(ret, Ok(&Account { balance: 41 }));

        let ret: Result<&Account, String> = bank.withdraw(acc_2.clone(), NonZeroMoney::MAX);
        assert_eq!(ret, Err("Insufficient funds".to_string()));
    }

    #[test]
    fn bank_should_move_funds() {
        let mut bank: Bank<InMemoryOpsStorage, InMemoryState> =
            Bank::new(InMemoryOpsStorage::default(), InMemoryState::default());

        let acc_1 = "Acc_1".to_string();
        let acc_2 = "Acc_2".to_string();
        let acc_3 = "Acc_3".to_string();

        let _ = bank.create_account(acc_1.clone());
        let _ = bank.create_account(acc_2.clone());
        let _ = bank.create_account(acc_3.clone());

        let ret: Result<&Account, String> =
            bank.deposit(acc_1.clone(), NonZeroMoney::MIN.checked_add(41).unwrap());
        assert_eq!(ret, Ok(&Account { balance: 42 }));

        let ret: Result<&Account, String> =
            bank.deposit(acc_3.clone(), NonZeroMoney::new(21).unwrap());
        assert_eq!(ret, Ok(&Account { balance: 21 }));
        {
            let mut iter = bank
                .move_money(acc_1.clone(), acc_2.clone(), NonZeroMoney::new(42).unwrap())
                .expect("should be Ok(iter)");

            let (_, first) = iter.next().expect("should be the acc_1 their Account");
            let (_, second) = iter.next().expect("should be the acc_2 their Account");
            assert_eq!(first, &Account { balance: 0 });
            assert_eq!(second, &Account { balance: 42 });
        }

        let ret: &Account = bank
            .get_balance(&acc_3)
            .expect("should be Ok(Account for acc_3)");
        assert_eq!(ret, &Account { balance: 21 });
    }

    #[test]
    fn bank_get_balance() {
        let mut bank: Bank<InMemoryOpsStorage, InMemoryState> =
            Bank::new(InMemoryOpsStorage::default(), InMemoryState::default());

        let acc_1 = "Acc_1".to_string();
        let acc_2 = "Acc_2".to_string();
        let acc_3 = "Acc_3".to_string();

        let _ = bank.create_account(acc_1.clone());
        let _ = bank.create_account(acc_2.clone());
        let _ = bank.create_account(acc_3.clone());

        let ret: Result<&Account, String> =
            bank.deposit(acc_1.clone(), NonZeroMoney::new(42).unwrap());
        assert_eq!(ret, Ok(&Account { balance: 42 }));

        let ret: Result<&Account, String> =
            bank.deposit(acc_3.clone(), NonZeroMoney::new(21).unwrap());
        assert_eq!(ret, Ok(&Account { balance: 21 }));

        let ret = bank.get_balance(&acc_1).expect("should be Account 1");
        assert_eq!(ret, &Account { balance: 42 });
        let ret = bank.get_balance(&acc_2).expect("should be Account 2");
        assert_eq!(ret, &Account { balance: 0 });
        let ret = bank.get_balance(&acc_3).expect("should be Account 3");
        assert_eq!(ret, &Account { balance: 21 });
    }
}
