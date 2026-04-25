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

/// Length, in base62 characters, of a path suffix in a function name.
/// Detection of a real path-suffix (vs a user-chosen suffix like
/// `"-v"`) uses this — the encoder is unchanged and produces
/// variable-length output, but only segments of *exactly*
/// [`PATH_SUFFIX_LEN`] base62 characters are treated as paths in
/// `child_name`/`reindex_name`.
pub const PATH_SUFFIX_LEN: usize = 6;

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

/// True iff `s` *looks like* a path suffix: exactly [`PATH_SUFFIX_LEN`]
/// characters, all of which are valid base62 (`0-9`, `A-Z`, `a-z`).
/// This is the gate `child_name`/`reindex_name` use before attempting
/// to decode — it short-circuits user-chosen suffixes like `-v` or
/// `-final` that would otherwise be successfully decoded as base62
/// integers and mistakenly extended.
pub fn looks_like_path_suffix(s: &str) -> bool {
    s.len() == PATH_SUFFIX_LEN
        && s.bytes()
            .all(|b| b.is_ascii_digit() || b.is_ascii_uppercase() || b.is_ascii_lowercase())
}

