//! Arbitrary helpers for types that don't natively implement `Arbitrary`.

/// Generates an arbitrary `IndexMap<K, V>` for use with `#[arbitrary(with = "...")]`.
pub fn arbitrary_indexmap<'a, K, V>(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<indexmap::IndexMap<K, V>>
where
    K: arbitrary::Arbitrary<'a> + std::hash::Hash + Eq,
    V: arbitrary::Arbitrary<'a>,
{
    let len = u.int_in_range(0..=4)?;
    let mut map = indexmap::IndexMap::with_capacity(len);
    for _ in 0..len {
        map.insert(u.arbitrary()?, u.arbitrary()?);
    }
    Ok(map)
}

/// Generates an arbitrary `Option<IndexMap<K, V>>` for use with `#[arbitrary(with = "...")]`.
pub fn arbitrary_option_indexmap<'a, K, V>(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Option<indexmap::IndexMap<K, V>>>
where
    K: arbitrary::Arbitrary<'a> + std::hash::Hash + Eq,
    V: arbitrary::Arbitrary<'a>,
{
    if u.arbitrary::<bool>()? {
        Ok(Some(arbitrary_indexmap(u)?))
    } else {
        Ok(None)
    }
}

/// Generates an arbitrary `serde_json::Value` for use with `#[arbitrary(with = "...")]`.
pub fn arbitrary_json_value(u: &mut arbitrary::Unstructured) -> arbitrary::Result<serde_json::Value> {
    // Limit recursion depth to avoid stack overflow
    arbitrary_json_value_depth(u, 3)
}

fn arbitrary_json_value_depth(
    u: &mut arbitrary::Unstructured,
    depth: u8,
) -> arbitrary::Result<serde_json::Value> {
    if depth == 0 {
        // At max depth, only produce leaf values
        return match u.int_in_range(0..=3)? {
            0 => Ok(serde_json::Value::Null),
            1 => Ok(serde_json::Value::Bool(u.arbitrary()?)),
            2 => Ok(serde_json::Value::Number(
                serde_json::Number::from(u.arbitrary::<i64>()?),
            )),
            _ => Ok(serde_json::Value::String(u.arbitrary()?)),
        };
    }

    match u.int_in_range(0..=5)? {
        0 => Ok(serde_json::Value::Null),
        1 => Ok(serde_json::Value::Bool(u.arbitrary()?)),
        2 => {
            // Integer or float
            if u.arbitrary()? {
                Ok(serde_json::Value::Number(
                    serde_json::Number::from(u.arbitrary::<i64>()?),
                ))
            } else {
                let f: f64 = u.arbitrary()?;
                Ok(serde_json::Number::from_f64(f)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null))
            }
        }
        3 => Ok(serde_json::Value::String(u.arbitrary()?)),
        4 => {
            // Array (bounded length)
            let len = u.int_in_range(0..=4)?;
            let mut arr = Vec::with_capacity(len);
            for _ in 0..len {
                arr.push(arbitrary_json_value_depth(u, depth - 1)?);
            }
            Ok(serde_json::Value::Array(arr))
        }
        _ => {
            // Object (bounded length)
            let len = u.int_in_range(0..=4)?;
            let mut map = serde_json::Map::with_capacity(len);
            for _ in 0..len {
                let key: String = u.arbitrary()?;
                let val = arbitrary_json_value_depth(u, depth - 1)?;
                map.insert(key, val);
            }
            Ok(serde_json::Value::Object(map))
        }
    }
}
