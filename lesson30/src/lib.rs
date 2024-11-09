use std::{
    collections::{BTreeMap, HashMap, LinkedList},
    marker::PhantomData,
    num::{NonZeroU128, NonZeroU32},
};

type AccountId = String;
type Money = u32;
type NonZeroMoney = NonZeroU32;
type Err = String;
type OpId = NonZeroU128;

#[derive(Debug, Default)]
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
pub trait OpsStorage<'a> {
    fn transact(
        &'a mut self,
        ops: impl Iterator<Item = (AccountId, Operation)>,
    ) -> Result<impl Iterator<Item = (OpId, AccountId, &'a Operation)>, Err>;

    fn persist(
        &'a mut self,
        account_id: AccountId,
        op: Operation,
    ) -> Result<(OpId, &'a Operation), Err> {
        self.transact(vec![(account_id, op)].into_iter())
            .and_then(|mut iter| {
                iter.next()
                    .map(|(opt_id, _, op)| (opt_id, op))
                    .ok_or("a transaction should return at least one operation".to_owned())
            })
    }
    //should be O(N), where N - account ops
    fn get_ops(
        &'a self,
        account_id: &AccountId,
    ) -> Result<impl Iterator<Item = (OpId, &'a Operation)>, Err>;
    //should be O(M), where M - all ops
    fn get_history(
        &'a self,
    ) -> Result<impl Iterator<Item = (OpId, (AccountId, &'a Operation))>, Err>;
}

pub trait State<'a> {
    fn transact(
        &'a mut self,
        ops: impl Iterator<Item = (AccountId, &'a Operation)>,
    ) -> Result<impl Iterator<Item = (AccountId, &'a Account)>, Err>;

    fn update(&'a mut self, account_id: AccountId, op: &'a Operation) -> Result<&'a Account, Err> {
        self.transact(vec![(account_id, op)].into_iter())
            .and_then(|mut iter| {
                iter.next()
                    .map(|(_, acc)| acc)
                    .ok_or("a transaction should return at least one account".to_owned())
            })
    }
    //should be O(N), where N - account ops
    fn get_balance(&'a self, account_id: &AccountId) -> Result<&'a Account, Err>;
}

pub struct Bank<'a, T: OpsStorage<'a>, S: State<'a>> {
    storage: T,
    state: S,
    _ignored: &'a PhantomData<()>,
}

impl<'a, T: OpsStorage<'a>, S: State<'a>> Bank<'a, T, S> {
    //создание аккаунта
    pub fn create_account(&'a mut self, account_id: AccountId) -> Result<&Account, Err> {
        let op = Operation::Create(account_id.clone());
        let (_, op) = self.storage.persist(account_id.clone(), op)?;
        self.state.update(account_id, op)
    }

    //Клиент может получить свой баланс.
    pub fn get_balance(&'a self, account_id: &AccountId) -> Result<&'a Account, Err> {
        self.state.get_balance(account_id)
    }

    //Клиент может пополнить свой баланс.
    pub fn deposit(
        &'a mut self,
        account_id: AccountId,
        money: NonZeroMoney,
    ) -> Result<&Account, Err> {
        let op = Operation::Charge(money);
        let (_, op) = self.storage.persist(account_id.clone(), op)?;
        self.state.update(account_id, op)
    }

    //Клиент может забрать деньги
    pub fn withdraw(
        &'a mut self,
        account_id: AccountId,
        money: NonZeroMoney,
    ) -> Result<&Account, Err> {
        let op = Operation::Withdraw(money);
        let (_, op) = self.storage.persist(account_id.clone(), op)?;
        self.state.update(account_id, op)
    }

    //перемещение денег от счета на счет
    pub fn move_money(
        &'a mut self,
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
                return Err("Err".to_string()); //TODO: message
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
                    return Err("Err".to_string()) //TODO: message
                }
            }

            _ => return Err("Err".to_string()), //TODO: message
        }

        Ok(
            self.0
                .get(&account_id)
                .map(|account| (account_id, account))
                .expect("err"), //здесь expect потому, что, если такого аккаунта нет, то это ошибка в коде
        )
    }
}

impl<'a> State<'a> for InMemoryState {
    fn get_balance(&'a self, account_id: &AccountId) -> Result<&'a Account, Err> {
        self.0
            .get(account_id)
            .ok_or(format!("Account[{}] not found in bank", account_id))
    }

    fn transact(
        &'a mut self,
        ops: impl Iterator<Item = (AccountId, &'a Operation)>,
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

impl<'a> InMemoryOpsStorage {
    fn push_to_cols(&'a mut self, account_id: AccountId, op: Operation) -> OpId {
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

impl<'a> OpsStorage<'a> for InMemoryOpsStorage {
    fn get_history(
        &'a self,
    ) -> Result<impl Iterator<Item = (OpId, (AccountId, &'a Operation))>, Err> {
        Ok(self
            .by_ops_storage
            .iter()
            .map(|(op_id, (account_id, operation))| {
                (op_id.clone(), (account_id.clone(), operation))
            }))
    }

    fn get_ops(
        &'a self,
        account_id: &AccountId,
    ) -> Result<impl Iterator<Item = (OpId, &'a Operation)>, Err> {
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
        &'a mut self,
        ops: impl Iterator<Item = (AccountId, Operation)>,
    ) -> Result<impl Iterator<Item = (OpId, AccountId, &'a Operation)>, Err> {
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

        // let mut results = Vec::new();

        // // Собираем временные операции
        // let temp_storage: Vec<_> = ops
        //     .map(|(account_id, op)| {
        //         let new_key = self.cur_key.checked_add(1).unwrap();
        //         self.cur_key = new_key;

        //         // Запись временной операции
        //         (new_key, account_id.clone(), op)
        //     })
        //     .collect();

        // // Теперь добавляем операции в by_acc_storage и by_ops_storage
        // for (op_id, account_id, op) in &temp_storage {
        //     // Добавляем в by_ops_storage
        //     self.by_ops_storage.insert(*op_id, (account_id.clone(), op.clone()));

        //     // Добавляем в by_acc_storage
        //     self.by_acc_storage
        //         .entry(account_id.clone())
        //         .or_insert_with(LinkedList::new)
        //         .push_back(*op_id);
        // }

        // // Возвращаем итератор с результатами
        // for (op_id, account_id, op) in temp_storage {
        //     let operation_ref = self.by_ops_storage.get(&op_id).unwrap();
        //     results.push((op_id, account_id, &operation_ref.1));
        // }

        // // Возвращаем итератор, который содержит все результаты
        // Ok(results.into_iter())
    }
}
