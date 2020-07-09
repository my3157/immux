use std::convert::TryFrom;
use std::path::PathBuf;

use crate::storage::chain_height::ChainHeight;
use crate::storage::executor::errors::ExecutorResult;
use crate::storage::executor::instruction::Instruction;
use crate::storage::executor::unit_content::{UnitContent, UnitContentError};
use crate::storage::executor::unit_key::UnitKey;
use crate::storage::kv::LogKeyValueStore;
use crate::storage::kvkey::KVKey;
use crate::storage::kvvalue::KVValue;
use crate::storage::transaction_manager::TransactionId;

pub struct Executor {
    store_engine: LogKeyValueStore,
}

impl Executor {
    pub fn open(path: &PathBuf) -> ExecutorResult<Executor> {
        let store_engine = LogKeyValueStore::open(path)?;
        let executor = Executor { store_engine };
        return Ok(executor);
    }

    pub fn set(&mut self, key: &UnitKey, value: &UnitContent, transaction_id: Option<TransactionId>) -> ExecutorResult<()> {
        let kv_key = KVKey::from(key);
        let kv_value = KVValue::from(value);
        self.store_engine.set(&kv_key, &kv_value, transaction_id)?;
        return Ok(());
    }

    pub fn get(&mut self, key: &UnitKey, transaction_id: Option<TransactionId>) -> ExecutorResult<Option<UnitContent>> {
        let kv_key = KVKey::from(key);
        match self.store_engine.get(&kv_key, transaction_id)? {
            None => Ok(None),
            Some(kv_value) => {
                let (content, _) = UnitContent::parse(kv_value.as_bytes())?;
                return Ok(Some(content));
            }
        }
    }

    pub fn revert_one(&mut self, key: &UnitKey, height: &ChainHeight, transaction_id: Option<TransactionId>) -> ExecutorResult<()> {
        let kv_key = KVKey::from(key);
        self.store_engine.revert_one(&kv_key, height, transaction_id)?;
        return Ok(());
    }

    pub fn revert_all(&mut self, height: &ChainHeight) -> ExecutorResult<()> {
        self.store_engine.revert_all(height)?;
        return Ok(());
    }

    pub fn remove_one(&mut self, key: &UnitKey, transaction_id: Option<TransactionId>) -> ExecutorResult<()> {
        let kv_key = KVKey::from(key);
        self.store_engine.remove_one(&kv_key, transaction_id)?;
        return Ok(());
    }

    pub fn remove_all(&mut self) -> ExecutorResult<()> {
        self.store_engine.remove_all()?;
        return Ok(());
    }

    pub fn inspect_all(&mut self) -> ExecutorResult<Vec<(Instruction, ChainHeight)>> {
        let result: Result<Vec<_>, UnitContentError> = self
            .store_engine
            .inspect_all()?
            .iter()
            .map(|(command, height)| {
                let instruction = Instruction::try_from(command)?;
                Ok((instruction, height.to_owned()))
            })
            .collect();

        return Ok(result?);
    }

    pub fn inspect_one(&mut self, target_key: &UnitKey) -> ExecutorResult<Vec<(Instruction, ChainHeight)>> {
        let kv_key = KVKey::from(target_key);
        let result: Result<Vec<_>, UnitContentError> = self
            .store_engine
            .inspect_one(&kv_key)?
            .iter()
            .map(|(command, height)| {
                let instruction = Instruction::try_from(command)?;
                Ok((instruction, height.to_owned()))
            })
            .collect();

        return Ok(result?);
    }

    pub fn start_transaction(&mut self) -> ExecutorResult<TransactionId> {
        let transaction_id = self.store_engine.start_transaction()?;
        return Ok(transaction_id);
    }

    pub fn commit_transaction(&mut self, transaction_id: TransactionId) -> ExecutorResult<()> {
        self.store_engine.commit_transaction(transaction_id)?;
        return Ok(());
    }

    pub fn abort_transaction(&mut self, transaction_id: TransactionId) -> ExecutorResult<()> {
        self.store_engine.abort_transaction(transaction_id)?;
        return Ok(());
    }
}