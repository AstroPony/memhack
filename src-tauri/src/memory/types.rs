use serde::{Deserialize, Serialize};

/// Supported value types for scanning and editing
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum ScanValue {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    StringUtf8(String),
    StringUtf16(String),
    ByteArray(Vec<u8>),
}

impl ScanValue {
    /// Byte size of this value type
    pub fn byte_size(&self) -> usize {
        match self {
            ScanValue::I8(_) | ScanValue::U8(_) => 1,
            ScanValue::I16(_) | ScanValue::U16(_) => 2,
            ScanValue::I32(_) | ScanValue::U32(_) | ScanValue::F32(_) => 4,
            ScanValue::I64(_) | ScanValue::U64(_) | ScanValue::F64(_) => 8,
            ScanValue::StringUtf8(s) => s.len(),
            ScanValue::StringUtf16(s) => s.encode_utf16().count() * 2,
            ScanValue::ByteArray(b) => b.len(),
        }
    }

    /// Convert value to its little-endian byte representation
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            ScanValue::I8(v) => v.to_le_bytes().to_vec(),
            ScanValue::I16(v) => v.to_le_bytes().to_vec(),
            ScanValue::I32(v) => v.to_le_bytes().to_vec(),
            ScanValue::I64(v) => v.to_le_bytes().to_vec(),
            ScanValue::U8(v) => v.to_le_bytes().to_vec(),
            ScanValue::U16(v) => v.to_le_bytes().to_vec(),
            ScanValue::U32(v) => v.to_le_bytes().to_vec(),
            ScanValue::U64(v) => v.to_le_bytes().to_vec(),
            ScanValue::F32(v) => v.to_le_bytes().to_vec(),
            ScanValue::F64(v) => v.to_le_bytes().to_vec(),
            ScanValue::StringUtf8(s) => s.as_bytes().to_vec(),
            ScanValue::StringUtf16(s) => s
                .encode_utf16()
                .flat_map(|c| c.to_le_bytes())
                .collect(),
            ScanValue::ByteArray(b) => b.clone(),
        }
    }

    /// Read this value type from a byte buffer at the given offset
    pub fn read_from_buffer(&self, buf: &[u8], offset: usize) -> Option<ScanValue> {
        let size = self.byte_size();
        if offset + size > buf.len() {
            return None;
        }
        let slice = &buf[offset..offset + size];

        Some(match self {
            ScanValue::I8(_) => ScanValue::I8(i8::from_le_bytes(slice.try_into().ok()?)),
            ScanValue::I16(_) => ScanValue::I16(i16::from_le_bytes(slice.try_into().ok()?)),
            ScanValue::I32(_) => ScanValue::I32(i32::from_le_bytes(slice.try_into().ok()?)),
            ScanValue::I64(_) => ScanValue::I64(i64::from_le_bytes(slice.try_into().ok()?)),
            ScanValue::U8(_) => ScanValue::U8(u8::from_le_bytes(slice.try_into().ok()?)),
            ScanValue::U16(_) => ScanValue::U16(u16::from_le_bytes(slice.try_into().ok()?)),
            ScanValue::U32(_) => ScanValue::U32(u32::from_le_bytes(slice.try_into().ok()?)),
            ScanValue::U64(_) => ScanValue::U64(u64::from_le_bytes(slice.try_into().ok()?)),
            ScanValue::F32(_) => ScanValue::F32(f32::from_le_bytes(slice.try_into().ok()?)),
            ScanValue::F64(_) => ScanValue::F64(f64::from_le_bytes(slice.try_into().ok()?)),
            ScanValue::StringUtf8(_) => {
                ScanValue::StringUtf8(String::from_utf8_lossy(slice).to_string())
            }
            ScanValue::StringUtf16(_) => {
                let u16s: Vec<u16> = slice
                    .chunks_exact(2)
                    .map(|c| u16::from_le_bytes([c[0], c[1]]))
                    .collect();
                ScanValue::StringUtf16(String::from_utf16_lossy(&u16s))
            }
            ScanValue::ByteArray(_) => ScanValue::ByteArray(slice.to_vec()),
        })
    }

    /// Check if this value matches a byte pattern at the given offset
    pub fn matches_bytes(&self, buf: &[u8], offset: usize) -> bool {
        let target = self.to_bytes();
        let size = target.len();
        if offset + size > buf.len() {
            return false;
        }
        // For floats, use approximate comparison
        match self {
            ScanValue::F32(target_val) => {
                if let Ok(bytes) = buf[offset..offset + 4].try_into() {
                    let found = f32::from_le_bytes(bytes);
                    (found - target_val).abs() < 0.001
                } else {
                    false
                }
            }
            ScanValue::F64(target_val) => {
                if let Ok(bytes) = buf[offset..offset + 8].try_into() {
                    let found = f64::from_le_bytes(bytes);
                    (found - target_val).abs() < 0.0001
                } else {
                    false
                }
            }
            _ => buf[offset..offset + size] == target[..],
        }
    }
}

/// How to compare values during a "next scan"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScanFilter {
    ExactValue(ScanValue),
    Changed,
    Unchanged,
    Increased,
    Decreased,
    IncreasedBy(ScanValue),
    DecreasedBy(ScanValue),
    GreaterThan(ScanValue),
    LessThan(ScanValue),
    Between(ScanValue, ScanValue),
}

/// A found memory address with its current value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoundAddress {
    pub address: u64,
    pub current_value: ScanValue,
    /// Raw bytes stored for "changed/unchanged" comparison
    pub previous_bytes: Vec<u8>,
}

/// An entry in the user's saved address table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableEntry {
    pub label: String,
    pub address: u64,
    pub value_type: ValueType,
    pub frozen: bool,
    pub freeze_value: Option<ScanValue>,
}

/// Value type selector (without holding an actual value)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValueType {
    I8, I16, I32, I64,
    U8, U16, U32, U64,
    F32, F64,
    StringUtf8(usize),  // max length
    StringUtf16(usize),
    ByteArray(usize),
}

impl ValueType {
    pub fn byte_size(&self) -> usize {
        match self {
            ValueType::I8 | ValueType::U8 => 1,
            ValueType::I16 | ValueType::U16 => 2,
            ValueType::I32 | ValueType::U32 | ValueType::F32 => 4,
            ValueType::I64 | ValueType::U64 | ValueType::F64 => 8,
            ValueType::StringUtf8(n) => *n,
            ValueType::StringUtf16(n) => *n,
            ValueType::ByteArray(n) => *n,
        }
    }

    /// Create a zero/default ScanValue of this type
    pub fn default_value(&self) -> ScanValue {
        match self {
            ValueType::I8 => ScanValue::I8(0),
            ValueType::I16 => ScanValue::I16(0),
            ValueType::I32 => ScanValue::I32(0),
            ValueType::I64 => ScanValue::I64(0),
            ValueType::U8 => ScanValue::U8(0),
            ValueType::U16 => ScanValue::U16(0),
            ValueType::U32 => ScanValue::U32(0),
            ValueType::U64 => ScanValue::U64(0),
            ValueType::F32 => ScanValue::F32(0.0),
            ValueType::F64 => ScanValue::F64(0.0),
            ValueType::StringUtf8(_) => ScanValue::StringUtf8(String::new()),
            ValueType::StringUtf16(_) => ScanValue::StringUtf16(String::new()),
            ValueType::ByteArray(n) => ScanValue::ByteArray(vec![0; *n]),
        }
    }
}

/// Info about a running process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
}

/// A readable memory region in the target process
#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub base_address: u64,
    pub size: u64,
}

/// Scan session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanState {
    pub results: Vec<FoundAddress>,
    pub scan_count: u32,
    pub value_type: ValueType,
}
