use std::collections::HashMap;
use std::fmt;

use crate::constants as Constants;
use crate::storage::kvkey::KVKey;
use crate::storage::log_pointer::LogPointer;
use crate::utils::varint::varint_encode;

pub type Snapshot = HashMap<KVKey, HashMap<Option<TransactionId>, LogPointer>>;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq)]
pub struct TransactionId(u64);

impl TransactionId {
    pub fn new(data: u64) -> Self {
        Self(data)
    }

    pub fn increment(&mut self) -> Result<Self, TransactionManagerError> {
        if self.0 >= Constants::MAX_TRANSACTION_ID {
            return Err(TransactionManagerError::TransactionIdOutOfRange);
        }
        self.0 += 1;
        return Ok(Self(self.0));
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }

    pub fn serialize(&self) -> Vec<u8> {
        varint_encode(self.as_u64())
    }
}

#[derive(Debug, PartialEq)]
pub enum TransactionManagerError {
    TransactionIdOutOfRange,
    TransactionNotAlive,
}

impl fmt::Display for TransactionManagerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionManagerError::TransactionIdOutOfRange => write!(
                f,
                "{}",
                "TransactionManagerError::TransactionIdOutOfRange".to_string()
            ),
            TransactionManagerError::TransactionNotAlive => write!(
                f,
                "{}",
                "TransactionManagerError::TransactionNotAlive".to_string()
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransactionMetaData {
    pub affected_keys: Vec<KVKey>,
    pub snapshot: Snapshot,
}

impl TransactionMetaData {
    pub fn new(affected_keys: Vec<KVKey>, snapshot: Snapshot) -> TransactionMetaData {
        TransactionMetaData {
            affected_keys,
            snapshot,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransactionManager {
    pub current_transaction_id: TransactionId,
    pub transactions: HashMap<TransactionId, TransactionMetaData>,
}

impl TransactionManager {
    pub fn new() -> TransactionManager {
        TransactionManager {
            current_transaction_id: TransactionId::new(0),
            transactions: HashMap::new(),
        }
    }

    pub fn generate_new_transaction_id(
        &mut self,
    ) -> Result<TransactionId, TransactionManagerError> {
        let next_transaction_id = self.current_transaction_id.increment()?;
        return Ok(next_transaction_id);
    }

    pub fn update_transaction_id(&mut self, transaction_id: &TransactionId) {
        self.current_transaction_id = transaction_id.to_owned();
    }

    pub fn add_affected_keys(
        &mut self,
        transaction_id: &TransactionId,
        key: &KVKey,
    ) -> Result<(), TransactionManagerError> {
        if let Some(transaction_meta_data) = self.transactions.get_mut(&transaction_id) {
            transaction_meta_data.affected_keys.push(key.to_owned());
            return Ok(());
        } else {
            return Err(TransactionManagerError::TransactionNotAlive);
        }
    }

    pub fn initialize_transaction(&mut self, transaction_id: &TransactionId, snapshot: Snapshot) {
        let transaction_meta_data = TransactionMetaData::new(vec![], snapshot);
        self.transactions
            .insert(transaction_id.clone(), transaction_meta_data);
    }

    pub fn validate_transaction_id(
        &self,
        transaction_id: &TransactionId,
    ) -> Result<(), TransactionManagerError> {
        return if self.transactions.contains_key(&transaction_id) {
            Ok(())
        } else {
            Err(TransactionManagerError::TransactionNotAlive)
        };
    }

    pub fn get_affected_keys(&self, transaction_id: &TransactionId) -> Vec<KVKey> {
        if let Some(transaction_meta_data) = self.transactions.get(&transaction_id) {
            let keys = &transaction_meta_data.affected_keys;
            keys.to_owned()
        } else {
            vec![]
        }
    }

    pub fn remove_transaction(&mut self, transaction_id: &TransactionId) {
        self.transactions.remove(&transaction_id);
    }

    pub fn get_transaction_meta_data(
        &self,
        transaction_id: &TransactionId,
    ) -> Result<TransactionMetaData, TransactionManagerError> {
        if let Some(transaction_meta_data) = self.transactions.get(transaction_id) {
            Ok(transaction_meta_data.clone())
        } else {
            Err(TransactionManagerError::TransactionNotAlive)
        }
    }

    pub fn update_transaction_meta_data(
        &mut self,
        key: &KVKey,
        log_pointer: &LogPointer,
        transaction_id: &TransactionId,
    ) -> Result<(), TransactionManagerError> {
        if let Some(transaction_meta_data) = self.transactions.get_mut(&transaction_id) {
            if let Some(log_pointers) = transaction_meta_data.snapshot.get_mut(&key) {
                log_pointers.insert(Some(transaction_id.clone()), log_pointer.clone());
            } else {
                let mut log_pointers = HashMap::new();
                log_pointers.insert(Some(transaction_id.clone()), log_pointer.clone());
                transaction_meta_data
                    .snapshot
                    .insert(key.clone(), log_pointers);
            }
            Ok(())
        } else {
            Err(TransactionManagerError::TransactionNotAlive)
        }
    }
}
