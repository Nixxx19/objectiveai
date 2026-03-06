use std::fmt::Display;

pub trait PathElement: Copy + Into<u128> + TryFrom<u128> + Ord + Display {}

macro_rules! impl_path_element {
    ($($t:ty),*) => {
        $(impl PathElement for $t {})*
    };
}

impl_path_element!(u8, u16, u32, u64, u128);

const MAX_LEN: usize = 16;
const MAX_VAL: u128 = 254;

fn validate_path<T: PathElement>(path: &[T]) -> Result<(), String> {
    if path.len() > MAX_LEN {
        return Err(format!(
            "path length {} exceeds maximum of {MAX_LEN}",
            path.len()
        ));
    }
    for (i, &v) in path.iter().enumerate() {
        if v.into() > MAX_VAL {
            return Err(format!("path[{i}] value {v} exceeds maximum of {MAX_VAL}"));
        }
    }
    Ok(())
}

/// Bijective base-255 encoding: each value v is stored as v+1 (digits 1-255).
/// Since there is no zero digit, different-length paths always produce
/// different u128 values (e.g. [] → 0, [0] → 1, [0,0] → 256).
pub fn path_to_u128<T: PathElement>(path: &[T]) -> Result<u128, String> {
    validate_path(path)?;
    let mut result: u128 = 0;
    for &v in path {
        result = result * 255 + v.into() + 1;
    }
    Ok(result)
}

pub fn u128_to_path<T: PathElement>(mut encoded: u128) -> Result<Vec<T>, String> {
    let mut path = Vec::new();
    while encoded > 0 {
        encoded -= 1;
        let digit = encoded % 255;
        path.push(
            T::try_from(digit)
                .map_err(|_| format!("value {digit} out of range for target type"))?,
        );
        encoded /= 255;
    }
    if path.len() > MAX_LEN {
        return Err(format!(
            "decoded length {} exceeds maximum of {MAX_LEN}",
            path.len()
        ));
    }
    path.reverse();
    Ok(path)
}

pub fn u128_to_b62(v: u128) -> String {
    base62::encode(v)
}

pub fn b62_to_u128(s: &str) -> Result<u128, String> {
    base62::decode(s).map_err(|e| format!("invalid base62: {e}"))
}

pub fn path_to_b62<T: PathElement>(path: &[T]) -> Result<String, String> {
    path_to_u128(path).map(u128_to_b62)
}

pub fn b62_to_path<T: PathElement>(s: &str) -> Result<Vec<T>, String> {
    b62_to_u128(s).and_then(u128_to_path)
}

