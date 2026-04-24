use serde_json::{Number, Value};

use crate::error::EncodingError;
use crate::models::DataType;

pub fn decode_value_to_json(dtype: &DataType, bytes: &[u8]) -> Result<Value, EncodingError> {
    match dtype {
        DataType::Float32 => {
            let value = f32::from_ne_bytes(parse_fixed::<4>(bytes)?);
            Number::from_f64(value as f64)
                .map(Value::Number)
                .ok_or_else(|| {
                    EncodingError::Storage(format!("cannot represent {value} as JSON number"))
                })
        }
        DataType::Float64 => {
            let value = f64::from_ne_bytes(parse_fixed::<8>(bytes)?);
            Number::from_f64(value).map(Value::Number).ok_or_else(|| {
                EncodingError::Storage(format!("cannot represent {value} as JSON number"))
            })
        }
        DataType::Int8 => Ok(Value::Number(
            i8::from_ne_bytes(parse_fixed::<1>(bytes)?).into(),
        )),
        DataType::Int16 => Ok(Value::Number(
            i16::from_ne_bytes(parse_fixed::<2>(bytes)?).into(),
        )),
        DataType::Int32 => Ok(Value::Number(
            i32::from_ne_bytes(parse_fixed::<4>(bytes)?).into(),
        )),
        DataType::Int64 => Ok(Value::Number(
            i64::from_ne_bytes(parse_fixed::<8>(bytes)?).into(),
        )),
        DataType::UInt8 => Ok(Value::Number(
            u8::from_ne_bytes(parse_fixed::<1>(bytes)?).into(),
        )),
        DataType::UInt16 => Ok(Value::Number(
            u16::from_ne_bytes(parse_fixed::<2>(bytes)?).into(),
        )),
        DataType::UInt32 => Ok(Value::Number(
            u32::from_ne_bytes(parse_fixed::<4>(bytes)?).into(),
        )),
        DataType::UInt64 => Ok(Value::Number(
            u64::from_ne_bytes(parse_fixed::<8>(bytes)?).into(),
        )),
    }
}

pub fn format_value(dtype: &DataType, bytes: &[u8]) -> Result<String, EncodingError> {
    decode_value_to_json(dtype, bytes).map(|value| value.to_string())
}

fn parse_fixed<const N: usize>(bytes: &[u8]) -> Result<[u8; N], EncodingError> {
    if bytes.len() < N {
        return Err(EncodingError::Storage(format!(
            "not enough bytes to decode value: expected at least {N}, got {}",
            bytes.len()
        )));
    }

    let mut arr = [0_u8; N];
    arr.copy_from_slice(&bytes[..N]);
    Ok(arr)
}
