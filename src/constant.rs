use crate::error::Error;

// Area ID
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum Area {
    ProcessInput = 0x81,
    ProcessOutput = 0x82,
    /// Merkers are address registers within the CPU.
    /// The number of available flag bytes depends on the respective CPU and can be taken from the technical data.
    /// You can use flag bits, flag bytes, flag words or flag double words in a PLC program.
    Merker = 0x83,
    /// German thing, means building blocks
    /// This is your storage  
    DataBausteine = 0x84,
    Counter = 0x1C,
    Timer = 0x1D,
    Unknown,
}

// Word Length
pub const WL_BIT: i32 = 0x01; //Bit (inside a word)
pub const WL_BYTE: i32 = 0x02; //Byte (8 bit)
pub const WL_CHAR: i32 = 0x03;
pub const WL_WORD: i32 = 0x04; //Word (16 bit)
pub const WL_INT: i32 = 0x05;
pub const WL_DWORD: i32 = 0x06; //Double Word (32 bit)
pub const WL_DINT: i32 = 0x07; //Double Int (32 bit -2147483648 to +2147483647)
pub const WL_REAL: i32 = 0x08; //Real (32 bit float)
pub const WL_COUNTER: i32 = 0x1C; //Counter (16 bit)
pub const WL_TIMER: i32 = 0x1D; //Timer (16 bit)

//dataSize to number of byte accordingly
pub(crate) fn data_size_byte(word_length: i32) -> i32 {
    match word_length {
        WL_BIT | WL_BYTE | WL_CHAR => 1,
        WL_WORD | WL_INT | WL_COUNTER | WL_TIMER => 2,
        WL_DWORD | WL_DINT | WL_REAL => 4,
        _ => 0,
    }
}

// PLC Status
#[derive(Debug)]
pub enum CpuStatus {
    Unknown = 0,
    StopByUser = 3,
    Stop = 4,
    Run = 8,
}

impl CpuStatus {
    pub(crate) fn from_u8(value: u8) -> Result<CpuStatus, Error> {
        match value {
            0 => Ok(CpuStatus::Unknown),
            3 => Ok(CpuStatus::StopByUser),
            4 => Ok(CpuStatus::Stop),
            8 => Ok(CpuStatus::Run),
            _ => Err(Error::InvalidCpuStatus(value)),
        }
    }
}



#[derive(Debug, Clone)]
pub enum SubBlockType {
    Ob = 0x08,
    Db = 0x0A,
    Sdb = 0x0B,
    Fc = 0x0C,
    Sfc = 0x0D,
    Fb = 0x0E,
    Sfb = 0x0F,
}

impl SubBlockType {
    pub(crate) fn from_u8(value: u8) -> Result<Self, Error> {
        match value {
            0x08 => Ok(SubBlockType::Ob),
            0x0A => Ok(SubBlockType::Db),
            0x0B => Ok(SubBlockType::Sdb),
            0x0C => Ok(SubBlockType::Fc),
            0x0D => Ok(SubBlockType::Sfc),
            0x0E => Ok(SubBlockType::Fb),
            0x0F => Ok(SubBlockType::Sfb),
            _ => Err(Error::InvalidBlockType(value)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum BlockLang {
    Awl = 0x01,
    Kop = 0x02,
    Fup = 0x03,
    Scl = 0x04,
    Db = 0x05,
    Graph = 0x06,
}

impl BlockLang {
    pub(crate) fn from_u8(value: u8) -> Result<Self, Error> {
        match value {
            0x01 => Ok(BlockLang::Awl),
            0x02 => Ok(BlockLang::Kop),
            0x03 => Ok(BlockLang::Fup),
            0x04 => Ok(BlockLang::Scl),
            0x05 => Ok(BlockLang::Db),
            0x06 => Ok(BlockLang::Graph),
            _ => Err(Error::InvalidCpuStatus(value)),
        }
    }
}

//size header
pub(crate) const SIZE_HEADER_READ: i32 = 31; // Header Size when Reading
pub(crate) const SIZE_HEADER_WRITE: i32 = 35; // Header Size when Writing

// Result transport size
pub(crate) const TS_RES_BIT: u8 = 3;
pub(crate) const TS_RES_BYTE: u8 = 4;
#[allow(dead_code)]
pub(crate) const TS_RES_INT: u8 = 5;
pub(crate) const TS_RES_REAL: u8 = 7;
pub(crate) const TS_RES_OCTET: u8 = 9;
