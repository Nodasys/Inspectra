//! Scanner engine with optimizations

use crate::types::DataType;

/// Convert value to bytes based on data type
pub fn value_to_bytes(value: &str, data_type: DataType) -> Result<Vec<u8>, String> {
    match data_type {
        DataType::I8 => value
            .parse::<i8>()
            .map(|v| v.to_le_bytes().to_vec())
            .map_err(|e| e.to_string()),
        DataType::U8 => value
            .parse::<u8>()
            .map(|v| vec![v])
            .map_err(|e| e.to_string()),
        DataType::I16 => value
            .parse::<i16>()
            .map(|v| v.to_le_bytes().to_vec())
            .map_err(|e| e.to_string()),
        DataType::U16 => value
            .parse::<u16>()
            .map(|v| v.to_le_bytes().to_vec())
            .map_err(|e| e.to_string()),
        DataType::I32 => value
            .parse::<i32>()
            .map(|v| v.to_le_bytes().to_vec())
            .map_err(|e| e.to_string()),
        DataType::U32 => value
            .parse::<u32>()
            .map(|v| v.to_le_bytes().to_vec())
            .map_err(|e| e.to_string()),
        DataType::I64 => value
            .parse::<i64>()
            .map(|v| v.to_le_bytes().to_vec())
            .map_err(|e| e.to_string()),
        DataType::U64 => value
            .parse::<u64>()
            .map(|v| v.to_le_bytes().to_vec())
            .map_err(|e| e.to_string()),
        DataType::F32 => value
            .parse::<f32>()
            .map(|v| v.to_le_bytes().to_vec())
            .map_err(|e| e.to_string()),
        DataType::F64 => value
            .parse::<f64>()
            .map(|v| v.to_le_bytes().to_vec())
            .map_err(|e| e.to_string()),
        DataType::String => Ok(value.as_bytes().to_vec()),
        DataType::WString => Ok(value.encode_utf16().flat_map(|c| c.to_le_bytes()).collect()),
        DataType::Bytes => {
            // Parse hex string like "01 02 03 FF"
            let bytes: Result<Vec<u8>, _> = value
                .split_whitespace()
                .map(|s| u8::from_str_radix(s, 16))
                .collect();
            bytes.map_err(|e| e.to_string())
        }
    }
}

/// Convert bytes to display string
pub fn bytes_to_string(bytes: &[u8], data_type: DataType) -> String {
    match data_type {
        DataType::I8 if bytes.len() >= 1 => i8::from_le_bytes([bytes[0]]).to_string(),
        DataType::U8 if bytes.len() >= 1 => bytes[0].to_string(),
        DataType::I16 if bytes.len() >= 2 => {
            i16::from_le_bytes([bytes[0], bytes[1]]).to_string()
        }
        DataType::U16 if bytes.len() >= 2 => {
            u16::from_le_bytes([bytes[0], bytes[1]]).to_string()
        }
        DataType::I32 if bytes.len() >= 4 => {
            i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]).to_string()
        }
        DataType::U32 if bytes.len() >= 4 => {
            u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]).to_string()
        }
        DataType::I64 if bytes.len() >= 8 => i64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ])
        .to_string(),
        DataType::U64 if bytes.len() >= 8 => u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ])
        .to_string(),
        DataType::F32 if bytes.len() >= 4 => {
            f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]).to_string()
        }
        DataType::F64 if bytes.len() >= 8 => f64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ])
        .to_string(),
        DataType::String => String::from_utf8_lossy(bytes).to_string(),
        DataType::WString => {
            let u16_bytes: Vec<u16> = bytes
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            String::from_utf16_lossy(&u16_bytes)
        }
        DataType::Bytes => bytes
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" "),
        _ => "Invalid".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_conversion() {
        let bytes = value_to_bytes("42", DataType::I32).unwrap();
        assert_eq!(bytes, vec![42, 0, 0, 0]);

        let value = bytes_to_string(&bytes, DataType::I32);
        assert_eq!(value, "42");
    }

    #[test]
    fn test_float_conversion() {
        let bytes = value_to_bytes("3.14", DataType::F32).unwrap();
        let value = bytes_to_string(&bytes, DataType::F32);
        assert!(value.starts_with("3.14"));
    }
}
