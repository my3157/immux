use std::collections::{HashMap, HashSet};
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::Read;
use std::io::{BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::constants as Constants;
use crate::storage::buffer_parser::InstructionBufferParser;
use crate::storage::chain_height::ChainHeight;
use crate::storage::errors::{KVError, KVResult};
use crate::storage::instruction::{Instruction, InstructionError};
use crate::storage::kvkey::KVKey;
use crate::storage::kvvalue::KVValue;
use crate::storage::log_pointer::LogPointer;
use crate::storage::log_writer::LogWriter;
use crate::storage::transaction_manager::{TransactionId, TransactionManager};

pub struct LogKeyValueStore {
    reader: BufReader<File>,
    writer: LogWriter,
    key_pointer_map: HashMap<KVKey, HashMap<Option<TransactionId>, LogPointer>>,
    current_height: ChainHeight,
    transaction_manager: TransactionManager,
}

impl LogKeyValueStore {
    pub fn open(path: &PathBuf) -> KVResult<LogKeyValueStore> {
        create_dir_all(&path)?;

        let log_file_path = get_log_file_dir(&path);

        let writer_file_option = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file_path.to_owned())?;
        let writer = LogWriter::new(writer_file_option)?;

        let reader_file_option = OpenOptions::new()
            .read(true)
            .open(&log_file_path.to_owned())?;
        let mut reader = BufReader::new(reader_file_option);

        let (key_pointer_map, current_height, transaction_manager) =
            load_key_pointer_map(&mut reader, None)?;

        let engine = LogKeyValueStore {
            reader,
            writer,
            key_pointer_map,
            current_height,
            transaction_manager,
        };

        return Ok(engine);
    }

    pub fn set(
        &mut self,
        key: &KVKey,
        value: &KVValue,
        transaction_id: Option<TransactionId>,
    ) -> KVResult<()> {
        if key.as_bytes().len() > Constants::MAX_KEY_LENGTH {
            return Err(InstructionError::KeyExceedsMaxLength.into());
        }

        let instruction = {
            if let Some(transaction_id) = transaction_id {
                self.transaction_manager
                    .validate_transaction_id(&transaction_id)?;
                self.transaction_manager
                    .add_affected_keys(&transaction_id, &key);

                Instruction::TransactionalSet {
                    key: key.clone(),
                    value: value.clone(),
                    transaction_id,
                }
            } else {
                Instruction::Set {
                    key: key.clone(),
                    value: value.clone(),
                }
            }
        };

        let log_pointer = append_instruction(instruction, &mut self.writer)?;

        self.current_height.increment()?;

        update_key_pointer_map(key, &log_pointer, &mut self.key_pointer_map, transaction_id);

        return Ok(());
    }

    pub fn get(
        &mut self,
        key: &KVKey,
        transaction_id: Option<TransactionId>,
    ) -> KVResult<Option<KVValue>> {
        match self.key_pointer_map.get(&key) {
            None => Ok(None),
            Some(log_pointers) => {
                if let Some(log_pointer) = log_pointers.get(&transaction_id) {
                    self.reader.seek(SeekFrom::Start(log_pointer.pos))?;

                    let mut log_pointer_reader = self.reader.by_ref().take(log_pointer.len as u64);

                    let mut buffer = vec![0; log_pointer.len];
                    log_pointer_reader.read(&mut buffer)?;

                    let (instruction, _) = Instruction::parse(buffer.as_slice())?;

                    match instruction {
                        Instruction::Set { key: _, value } => Ok(Some(value)),
                        Instruction::RevertOne { key, height } => {
                            get_revert_value(&mut self.reader, &key, &height)
                        }
                        Instruction::RemoveOne { key: _ } => Ok(None),
                        Instruction::TransactionalSet {
                            key: _,
                            value,
                            transaction_id: _,
                        } => Ok(Some(value)),
                        Instruction::TransactionalRevertOne {
                            key,
                            height,
                            transaction_id: _,
                        } => get_revert_value(&mut self.reader, &key, &height),
                        Instruction::TransactionalRemoveOne {
                            key: _,
                            transaction_id: _,
                        } => Ok(None),
                        _ => Err(KVError::PointToUnexpectedInstruction),
                    }
                } else {
                    return if transaction_id.is_some() {
                        self.get(&key, None)
                    } else {
                        Ok(None)
                    };
                }
            }
        }
    }

    pub fn get_all_current(&mut self) -> KVResult<Vec<(KVKey, KVValue)>> {
        let mut result = vec![];

        for (_key, log_pointer) in &self.key_pointer_map {
            match log_pointer.get(&None) {
                None => {}
                Some(log_pointer) => {
                    self.reader.seek(SeekFrom::Start(log_pointer.pos))?;
                    let mut log_pointer_reader = self.reader.by_ref().take(log_pointer.len as u64);
                    let mut buffer = vec![0; log_pointer.len];
                    log_pointer_reader.read(&mut buffer)?;
                    let (instruction, _) = Instruction::parse(buffer.as_slice())?;

                    match instruction {
                        Instruction::Set { key, value } => {
                            result.push((key, value));
                        }
                        Instruction::RevertOne { key, height } => {
                            if let Some(value) = get_revert_value(&mut self.reader, &key, &height)?
                            {
                                result.push((key, value));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        return Ok(result);
    }

    pub fn revert_one(
        &mut self,
        key: &KVKey,
        height: &ChainHeight,
        transaction_id: Option<TransactionId>,
    ) -> KVResult<()> {
        if key.as_bytes().len() > Constants::MAX_KEY_LENGTH {
            return Err(InstructionError::KeyExceedsMaxLength.into());
        }

        if height > &self.current_height {
            return Err(KVError::RevertOutOfRange);
        }

        let instruction = {
            if let Some(transaction_id) = transaction_id {
                Instruction::TransactionalRevertOne {
                    key: key.clone(),
                    height: height.clone(),
                    transaction_id,
                }
            } else {
                Instruction::RevertOne {
                    key: key.clone(),
                    height: height.clone(),
                }
            }
        };

        if let Some(transaction_id) = transaction_id {
            self.transaction_manager
                .validate_transaction_id(&transaction_id)?;
            self.transaction_manager
                .add_affected_keys(&transaction_id, &key);
        }

        let log_pointer = append_instruction(instruction, &mut self.writer)?;

        self.current_height.increment()?;

        update_key_pointer_map(key, &log_pointer, &mut self.key_pointer_map, transaction_id);

        return Ok(());
    }

    pub fn revert_all(&mut self, height: &ChainHeight) -> KVResult<()> {
        if height > &self.current_height {
            return Err(KVError::RevertOutOfRange);
        }

        let instruction = Instruction::RevertAll {
            height: height.clone(),
        };

        append_instruction(instruction, &mut self.writer)?;

        self.current_height.increment()?;
        let (new_key_pointer_map, _, _) = load_key_pointer_map(&mut self.reader, Some(height))?;
        self.key_pointer_map = new_key_pointer_map;
        self.transaction_manager = TransactionManager::new();

        return Ok(());
    }

    pub fn remove_one(
        &mut self,
        key: &KVKey,
        transaction_id: Option<TransactionId>,
    ) -> KVResult<()> {
        if key.as_bytes().len() > Constants::MAX_KEY_LENGTH {
            return Err(InstructionError::KeyExceedsMaxLength.into());
        }

        let instruction = {
            if let Some(transaction_id) = transaction_id {
                Instruction::TransactionalRemoveOne {
                    key: key.clone(),
                    transaction_id,
                }
            } else {
                Instruction::RemoveOne { key: key.clone() }
            }
        };

        let log_pointer = append_instruction(instruction, &mut self.writer)?;

        if let Some(transaction_id) = transaction_id {
            self.transaction_manager
                .validate_transaction_id(&transaction_id)?;
            self.transaction_manager
                .add_affected_keys(&transaction_id, &key);
        }

        self.current_height.increment()?;

        update_key_pointer_map(key, &log_pointer, &mut self.key_pointer_map, transaction_id);

        return Ok(());
    }

    pub fn remove_all(&mut self) -> KVResult<()> {
        let instruction = Instruction::RemoveAll;

        append_instruction(instruction, &mut self.writer)?;

        for log_pointers in self.key_pointer_map.values_mut() {
            log_pointers.remove(&None);
        }

        self.current_height.increment()?;

        return Ok(());
    }

    pub fn inspect_all(&mut self) -> KVResult<Vec<(Instruction, ChainHeight)>> {
        let mut instruction_buffer: Vec<u8> = Vec::new();
        self.reader.seek(SeekFrom::Start(0))?;
        self.reader.read_to_end(&mut instruction_buffer)?;

        let instruction_buffer_parser = InstructionBufferParser::new(&instruction_buffer, 0);

        let instructions_with_height = instruction_buffer_parser
            .enumerate()
            .map(|(index, (instruction, _))| (instruction, ChainHeight::new((index) as u64)))
            .collect();
        return Ok(instructions_with_height);
    }

    pub fn inspect_one(&mut self, target_key: &KVKey) -> KVResult<Vec<(Instruction, ChainHeight)>> {
        let mut instruction_buffer: Vec<u8> = Vec::new();
        self.reader.seek(SeekFrom::Start(0))?;
        self.reader.read_to_end(&mut instruction_buffer)?;

        let instruction_buffer_parser = InstructionBufferParser::new(&instruction_buffer, 0);

        let mut appeared_key = HashSet::new();
        let result = instruction_buffer_parser
            .enumerate()
            .filter_map(|(index, (instruction, _))| match &instruction {
                Instruction::Set { key, value: _ } => {
                    appeared_key.insert(key.clone());

                    if target_key == key {
                        return Some((instruction, ChainHeight::new((index) as u64)));
                    } else {
                        return None;
                    }
                }
                Instruction::RevertOne { key, height: _ } => {
                    appeared_key.insert(key.clone());

                    if target_key == key {
                        return Some((instruction, ChainHeight::new((index) as u64)));
                    } else {
                        return None;
                    }
                }
                Instruction::RevertAll { height: _ } => {
                    if appeared_key.contains(&target_key) {
                        return Some((instruction, ChainHeight::new((index) as u64)));
                    } else {
                        return None;
                    }
                }
                Instruction::RemoveOne { key } => {
                    appeared_key.insert(key.clone());

                    if target_key == key {
                        return Some((instruction, ChainHeight::new((index) as u64)));
                    } else {
                        return None;
                    }
                }
                Instruction::RemoveAll => {
                    if appeared_key.contains(&target_key) {
                        return Some((instruction, ChainHeight::new((index) as u64)));
                    } else {
                        return None;
                    }
                }
                _ => None,
            })
            .collect();
        return Ok(result);
    }

    pub fn start_transaction(&mut self) -> KVResult<TransactionId> {
        let transaction_id = self.transaction_manager.generate_new_transaction_id()?;
        let instruction = Instruction::TransactionStart { transaction_id };
        append_instruction(instruction, &mut self.writer)?;

        self.transaction_manager
            .initialize_affected_keys(&transaction_id);

        self.current_height.increment()?;

        return Ok(transaction_id);
    }

    pub fn commit_transaction(&mut self, transaction_id: TransactionId) -> KVResult<()> {
        self.transaction_manager
            .validate_transaction_id(&transaction_id)?;

        let instruction = Instruction::TransactionCommit { transaction_id };
        append_instruction(instruction, &mut self.writer)?;

        update_committed_log_pointers(
            &mut self.transaction_manager,
            &mut self.key_pointer_map,
            transaction_id,
        );

        self.current_height.increment()?;

        return Ok(());
    }

    pub fn abort_transaction(&mut self, transaction_id: TransactionId) -> KVResult<()> {
        self.transaction_manager
            .validate_transaction_id(&transaction_id)?;

        let instruction = Instruction::TransactionAbort { transaction_id };
        append_instruction(instruction, &mut self.writer)?;

        update_aborted_log_pointers(
            &mut self.transaction_manager,
            &mut self.key_pointer_map,
            transaction_id,
        );

        self.current_height.increment()?;

        return Ok(());
    }
}

fn get_revert_value(
    reader: &mut BufReader<File>,
    key: &KVKey,
    height: &ChainHeight,
) -> KVResult<Option<KVValue>> {
    let mut instruction_buffer: Vec<u8> = Vec::new();
    reader.seek(SeekFrom::Start(0))?;
    reader.read_to_end(&mut instruction_buffer)?;
    let instruction_buffer_parser = InstructionBufferParser::new(&instruction_buffer, 0);

    let instructions: Vec<Instruction> = instruction_buffer_parser
        .map(|(instruction, _)| instruction)
        .collect();
    let value = recursive_find(&key, &instructions, &height)?;

    return Ok(value);
}

fn recursive_find(
    target_key: &KVKey,
    instructions: &Vec<Instruction>,
    target_height: &ChainHeight,
) -> KVResult<Option<KVValue>> {
    let target_instruction = &instructions.as_slice()[target_height.as_u64() as usize];

    match target_instruction {
        Instruction::Set { key, value } => {
            if target_key == key {
                return Ok(Some(value.to_owned()));
            } else {
                let next_target_height = &target_height.clone().decrement()?;
                return recursive_find(&target_key, &instructions, &next_target_height);
            }
        }
        Instruction::RevertOne { key, height } => {
            if target_key == key {
                return recursive_find(&target_key, &instructions, height);
            } else {
                let next_target_height = &target_height.clone().decrement()?;
                return recursive_find(&target_key, &instructions, &next_target_height);
            }
        }
        Instruction::RevertAll { height } => {
            return recursive_find(&target_key, &instructions, height);
        }
        Instruction::RemoveOne { key } => {
            if target_key == key {
                return Ok(None);
            } else {
                let next_target_height = &target_height.clone().decrement()?;
                return recursive_find(&target_key, &instructions, next_target_height);
            }
        }
        Instruction::RemoveAll => {
            return Ok(None);
        }
        _ => {
            let next_target_height = &target_height.clone().decrement()?;
            return recursive_find(&target_key, &instructions, &next_target_height);
        }
    }
}

fn load_key_pointer_map(
    mut reader: &mut BufReader<File>,
    target_height: Option<&ChainHeight>,
) -> KVResult<(
    HashMap<KVKey, HashMap<Option<TransactionId>, LogPointer>>,
    ChainHeight,
    TransactionManager,
)> {
    let mut instruction_buffer: Vec<u8> = Vec::new();
    let mut current_position = 0;
    let mut height = ChainHeight::new(0);
    let mut key_pointer_map: HashMap<KVKey, HashMap<Option<TransactionId>, LogPointer>> =
        HashMap::new();
    let mut transaction_manager = TransactionManager::new();

    reader.seek(SeekFrom::Start(0))?;
    reader.read_to_end(&mut instruction_buffer)?;
    let instruction_buffer_parser = InstructionBufferParser::new(&instruction_buffer, 0);

    for (instruction, instruction_length) in instruction_buffer_parser {
        match instruction {
            Instruction::Set { key, value: _ } => {
                let log_pointer = LogPointer::new(current_position, instruction_length);
                update_key_pointer_map(&key, &log_pointer, &mut key_pointer_map, None);
            }
            Instruction::RevertOne { key, height: _ } => {
                let log_pointer = LogPointer::new(current_position, instruction_length);
                update_key_pointer_map(&key, &log_pointer, &mut key_pointer_map, None);
            }
            Instruction::RevertAll { height } => {
                let (new_key_pointer_map, _, _) = load_key_pointer_map(&mut reader, Some(&height))?;
                transaction_manager = TransactionManager::new();
                key_pointer_map = new_key_pointer_map;
            }
            Instruction::RemoveOne { key } => {
                let log_pointer = LogPointer::new(current_position, instruction_length);
                update_key_pointer_map(&key, &log_pointer, &mut key_pointer_map, None);
            }
            Instruction::RemoveAll => {
                for log_pointers in key_pointer_map.values_mut() {
                    log_pointers.remove(&None);
                }
            }
            Instruction::TransactionStart { transaction_id } => {
                transaction_manager.initialize_affected_keys(&transaction_id);
                transaction_manager.update_transaction_id(&transaction_id);
            }
            Instruction::TransactionalSet {
                key,
                value: _,
                transaction_id,
            } => {
                let log_pointer = LogPointer::new(current_position, instruction_length);
                update_key_pointer_map(
                    &key,
                    &log_pointer,
                    &mut key_pointer_map,
                    Some(transaction_id),
                );
                transaction_manager.add_affected_keys(&transaction_id, &key);
            }
            Instruction::TransactionalRevertOne {
                key,
                height: _,
                transaction_id,
            } => {
                let log_pointer = LogPointer::new(current_position, instruction_length);
                update_key_pointer_map(
                    &key,
                    &log_pointer,
                    &mut key_pointer_map,
                    Some(transaction_id),
                );

                transaction_manager.add_affected_keys(&transaction_id, &key);
            }
            Instruction::TransactionalRemoveOne {
                key,
                transaction_id,
            } => {
                let log_pointer = LogPointer::new(current_position, instruction_length);
                update_key_pointer_map(
                    &key,
                    &log_pointer,
                    &mut key_pointer_map,
                    Some(transaction_id),
                );

                transaction_manager.add_affected_keys(&transaction_id, &key);
            }
            Instruction::TransactionCommit { transaction_id } => {
                update_committed_log_pointers(
                    &mut transaction_manager,
                    &mut key_pointer_map,
                    transaction_id,
                );
            }
            Instruction::TransactionAbort { transaction_id } => {
                update_aborted_log_pointers(
                    &mut transaction_manager,
                    &mut key_pointer_map,
                    transaction_id,
                );
            }
        }

        current_position += instruction_length as u64;

        if let Some(target_height) = target_height {
            if target_height == &height {
                break;
            }
        }

        height.increment()?;
    }

    return Ok((key_pointer_map, height, transaction_manager));
}

fn update_committed_log_pointers(
    transaction_manager: &mut TransactionManager,
    key_pointer_map: &mut HashMap<KVKey, HashMap<Option<TransactionId>, LogPointer>>,
    transaction_id: TransactionId,
) {
    let affected_keys = transaction_manager.get_affected_keys(&transaction_id);
    for key in affected_keys {
        if let Some(log_pointers) = key_pointer_map.get_mut(&key) {
            if let Some(target_log_pointer) = log_pointers.get(&Some(transaction_id)).cloned() {
                log_pointers.insert(None, target_log_pointer);
                log_pointers.remove(&Some(transaction_id));
            }
        }
    }

    transaction_manager.remove_transaction(&transaction_id);
}

fn update_aborted_log_pointers(
    transaction_manager: &mut TransactionManager,
    key_pointer_map: &mut HashMap<KVKey, HashMap<Option<TransactionId>, LogPointer>>,
    transaction_id: TransactionId,
) {
    let affected_keys = transaction_manager.get_affected_keys(&transaction_id);
    for key in affected_keys {
        if let Some(log_pointers) = key_pointer_map.get_mut(&key) {
            log_pointers.remove(&Some(transaction_id));
        }
    }

    transaction_manager.remove_transaction(&transaction_id);
}

fn update_key_pointer_map(
    key: &KVKey,
    log_pointer: &LogPointer,
    key_pointer_map: &mut HashMap<KVKey, HashMap<Option<TransactionId>, LogPointer>>,
    transaction_id: Option<TransactionId>,
) {
    if let Some(log_pointers) = key_pointer_map.get_mut(&key) {
        log_pointers.insert(transaction_id, log_pointer.clone());
    } else {
        let mut log_pointers = HashMap::new();
        log_pointers.insert(transaction_id, log_pointer.to_owned());
        key_pointer_map.insert(key.clone(), log_pointers);
    }
}

fn append_instruction(instruction: Instruction, writer: &mut LogWriter) -> KVResult<LogPointer> {
    let instruction_bytes: Vec<u8> = instruction.serialize();

    let current_pos = writer.get_current_pos();
    writer.write_all(instruction_bytes.as_slice())?;
    writer.flush()?;

    let log_pointer = LogPointer::new(current_pos, instruction_bytes.len());

    return Ok(log_pointer);
}

pub fn get_log_file_dir(dir: &PathBuf) -> PathBuf {
    let log_file_name = format!("{}.log", Constants::LOG_FILE_NAME);
    let log_path = dir.join(Path::new(&log_file_name));
    return log_path;
}
