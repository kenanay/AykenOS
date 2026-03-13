//! BCIB (Binary CLI Instruction Buffer) v0.2
//! DSL-uyumlu, hafif header + opcode set (data/ui/ai) ile stub executor.

use std::convert::TryFrom;

// --- Header ---

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BcibHeader {
    pub magic: [u8; 4],
    pub version: u16,
    pub instr_count: u16,
}

pub const BCIB_MAGIC: [u8; 4] = *b"BCIB";
pub const BCIB_VERSION: u16 = 2; // dok?mantasyonda 0.2

impl BcibHeader {
    pub fn new(instr_count: u16) -> Self {
        Self {
            magic: BCIB_MAGIC,
            version: BCIB_VERSION,
            instr_count,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.magic == BCIB_MAGIC && self.version == BCIB_VERSION
    }
}

// --- Opcodes ---

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BcibOpcode {
    Nop = 0x00,
    DataCreate = 0x10,
    DataAdd = 0x11,
    DataQuery = 0x12,
    UiRender = 0x20,
    AiAsk = 0x30,
    End = 0xFF,
}

impl TryFrom<u8> for BcibOpcode {
    type Error = DecodeError;
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0x00 => Ok(BcibOpcode::Nop),
            0x10 => Ok(BcibOpcode::DataCreate),
            0x11 => Ok(BcibOpcode::DataAdd),
            0x12 => Ok(BcibOpcode::DataQuery),
            0x20 => Ok(BcibOpcode::UiRender),
            0x30 => Ok(BcibOpcode::AiAsk),
            0xFF => Ok(BcibOpcode::End),
            _ => Err(DecodeError::InvalidOpcode(v)),
        }
    }
}

// --- Instruction ---

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BcibInstruction {
    pub opcode: BcibOpcode,
    pub flags: u8,
    pub args: [u16; 2],
}

impl BcibInstruction {
    pub fn new(opcode: BcibOpcode, flags: u8, args: [u16; 2]) -> Self {
        Self {
            opcode,
            flags,
            args,
        }
    }

    pub fn nop() -> Self {
        Self::new(BcibOpcode::Nop, 0, [0, 0])
    }
    pub fn end() -> Self {
        Self::new(BcibOpcode::End, 0, [0, 0])
    }
    pub fn data_create(target_idx: u16, schema_idx: u16) -> Self {
        Self::new(BcibOpcode::DataCreate, 0, [target_idx, schema_idx])
    }
    pub fn data_add(target_idx: u16, payload_idx: u16) -> Self {
        Self::new(BcibOpcode::DataAdd, 0, [target_idx, payload_idx])
    }
    pub fn data_query(target_idx: u16, filter_idx: u16) -> Self {
        Self::new(BcibOpcode::DataQuery, 0, [target_idx, filter_idx])
    }
    pub fn ui_render(scene_idx: u16) -> Self {
        Self::new(BcibOpcode::UiRender, 0, [scene_idx, 0])
    }
    pub fn ai_ask(prompt_idx: u16) -> Self {
        Self::new(BcibOpcode::AiAsk, 0, [prompt_idx, 0])
    }
}

// --- Buffer ---

#[derive(Debug, Default)]
pub struct BcibBuffer {
    instructions: Vec<BcibInstruction>,
}

impl BcibBuffer {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
        }
    }
    pub fn len(&self) -> usize {
        self.instructions.len()
    }
    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }

    pub fn add(&mut self, instr: BcibInstruction) -> usize {
        let idx = self.instructions.len();
        self.instructions.push(instr);
        idx
    }

    pub fn encode(&self) -> Vec<u8> {
        use std::{mem, ptr};
        let instr_count = self.instructions.len() as u16;
        let header = BcibHeader::new(instr_count);
        let header_size = mem::size_of::<BcibHeader>();
        let instr_size = mem::size_of::<BcibInstruction>();
        let total_size = header_size + instr_size * self.instructions.len();
        let mut buf = vec![0u8; total_size];

        unsafe {
            ptr::copy_nonoverlapping(
                &header as *const _ as *const u8,
                buf.as_mut_ptr(),
                header_size,
            );
            let mut p = buf.as_mut_ptr().add(header_size);
            for instr in &self.instructions {
                ptr::copy_nonoverlapping(instr as *const _ as *const u8, p, instr_size);
                p = p.add(instr_size);
            }
        }
        buf
    }

    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        use std::{mem, slice};
        let header_size = mem::size_of::<BcibHeader>();
        if buf.len() < header_size {
            return Err(DecodeError::BufferTooSmall);
        }
        let header: &BcibHeader = unsafe { &*(buf.as_ptr() as *const BcibHeader) };
        if !header.is_valid() {
            return Err(DecodeError::InvalidHeader);
        }
        let instr_size = mem::size_of::<BcibInstruction>();
        let expected_size = header_size + instr_size * header.instr_count as usize;
        if buf.len() < expected_size {
            return Err(DecodeError::CorruptLayout);
        }
        let raw_instrs: &[BcibInstruction] = unsafe {
            slice::from_raw_parts(
                buf.as_ptr().add(header_size) as *const BcibInstruction,
                header.instr_count as usize,
            )
        };
        // Validate opcodes
        let mut instructions = Vec::with_capacity(raw_instrs.len());
        for instr in raw_instrs {
            let opcode = BcibOpcode::try_from(instr.opcode as u8)?;
            instructions.push(BcibInstruction {
                opcode,
                flags: instr.flags,
                args: instr.args,
            });
        }
        Ok(Self { instructions })
    }

    pub fn step(&self, pc: &mut usize) -> Result<bool, String> {
        if *pc >= self.instructions.len() {
            return Ok(false);
        }
        let instr = self.instructions[*pc];
        *pc += 1;
        match instr.opcode {
            BcibOpcode::Nop => {}
            BcibOpcode::DataCreate => println!(
                "BCIB: data.create target={} schema={}",
                instr.args[0], instr.args[1]
            ),
            BcibOpcode::DataAdd => println!(
                "BCIB: data.add target={} payload={} ",
                instr.args[0], instr.args[1]
            ),
            BcibOpcode::DataQuery => println!(
                "BCIB: data.query target={} filter={}",
                instr.args[0], instr.args[1]
            ),
            BcibOpcode::UiRender => println!("BCIB: ui.render scene={}", instr.args[0]),
            BcibOpcode::AiAsk => println!("BCIB: ai.ask prompt={}", instr.args[0]),
            BcibOpcode::End => return Ok(false),
        }
        Ok(true)
    }

    pub fn execute(&self) -> Result<(), String> {
        let mut pc = 0;
        while self.step(&mut pc)? {}
        Ok(())
    }
}

// --- Errors ---

#[derive(Debug)]
pub enum DecodeError {
    BufferTooSmall,
    InvalidHeader,
    InvalidOpcode(u8),
    CorruptLayout,
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeError::BufferTooSmall => write!(f, "Buffer too small"),
            DecodeError::InvalidHeader => write!(f, "Invalid header"),
            DecodeError::InvalidOpcode(op) => write!(f, "Invalid opcode: {:#04x}", op),
            DecodeError::CorruptLayout => write!(f, "Corrupt layout"),
        }
    }
}

impl std::error::Error for DecodeError {}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        let mut buf = BcibBuffer::new();
        buf.add(BcibInstruction::data_create(1, 2));
        buf.add(BcibInstruction::data_add(1, 3));
        buf.add(BcibInstruction::data_query(1, 4));
        buf.add(BcibInstruction::ui_render(5));
        buf.add(BcibInstruction::ai_ask(6));
        buf.add(BcibInstruction::end());

        let bytes = buf.encode();
        let decoded = BcibBuffer::decode(&bytes).expect("decode failed");
        assert_eq!(decoded.len(), 6);
        decoded.execute().expect("execute failed");
    }

    #[test]
    fn invalid_header_magic() {
        let mut bytes = BcibBuffer::new().encode();
        bytes[0] = 0; // break magic
        let err = BcibBuffer::decode(&bytes).unwrap_err();
        assert!(matches!(err, DecodeError::InvalidHeader));
    }

    #[test]
    fn invalid_opcode_detected() {
        // craft buffer with bad opcode by patching encoded bytes (avoid unsafe)
        let mut buf = BcibBuffer::new();
        buf.add(BcibInstruction::nop());
        let mut bytes = buf.encode();

        // Header is fixed-size; the first instruction opcode is immediately after header.
        let hdr_size = std::mem::size_of::<BcibHeader>();
        bytes[hdr_size] = 0xAB; // invalid opcode

        let err = BcibBuffer::decode(&bytes).unwrap_err();
        assert!(matches!(err, DecodeError::InvalidOpcode(0xAB)));
    }
}
