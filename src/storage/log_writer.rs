use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Result, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crate::storage::ecc::ECCMode;
use crate::storage::instruction::{pack_instruction, Instruction};
use crate::storage::log_pointer::LogPointer;
use crate::storage::log_version::LogVersion;

pub struct LogWriter {
    buf_writer: BufWriter<File>,
    ecc_mode: ECCMode,
    pos: u64,
}

impl LogWriter {
    pub fn new(file_path: &PathBuf, ecc_mode: ECCMode, db_version: LogVersion) -> Result<Self> {
        if !Path::new(file_path).exists() {
            let file = File::create(file_path)?;
            let mut buf_writer = BufWriter::new(file);
            let db_version_bytes = db_version.marshal();
            buf_writer.write(&db_version_bytes)?;
            buf_writer.flush()?;

            let initial_pos = buf_writer.seek(SeekFrom::Current(0))?;

            Ok(LogWriter {
                buf_writer,
                ecc_mode,
                pos: initial_pos,
            })
        } else {
            let file = OpenOptions::new().append(true).open(file_path)?;
            let mut buf_writer = BufWriter::new(file);
            let initial_pos = buf_writer.seek(SeekFrom::Current(0))?;

            Ok(LogWriter {
                buf_writer,
                ecc_mode,
                pos: initial_pos,
            })
        }
    }

    pub fn append_instruction(&mut self, instruction: &Instruction) -> Result<LogPointer> {
        let pack_bytes = pack_instruction(instruction, self.ecc_mode);
        let pos_before_writing = self.pos;
        self.write_all(&pack_bytes)?;
        self.flush()?;

        let pointer_to_instruction = LogPointer::new(pos_before_writing, pack_bytes.len());

        return Ok(pointer_to_instruction);
    }

    fn write_all(&mut self, data: &[u8]) -> Result<()> {
        self.buf_writer.write_all(data)?;
        self.pos += data.len() as u64;
        return Ok(());
    }

    fn flush(&mut self) -> Result<()> {
        self.buf_writer.flush()
    }
}
