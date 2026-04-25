use super::path::*;

#[test]
fn roundtrip_u128() {
    let path: Vec<u8> = vec![0, 1, 254, 100];
    let encoded = path_to_u128(&path).unwrap();
    let decoded: Vec<u8> = u128_to_path(encoded).unwrap();
    assert_eq!(path, decoded);
}

#[test]
fn roundtrip_b62() {
    let path: Vec<u16> = vec![3, 200, 0, 42];
    let b62 = path_to_b62(&path).unwrap();
    let decoded: Vec<u16> = b62_to_path(&b62).unwrap();
    assert_eq!(path, decoded);
}

#[test]
fn empty_path() {
    let path: Vec<u8> = vec![];
    let encoded = path_to_u128(&path).unwrap();
    assert_eq!(encoded, 0);
    let decoded: Vec<u8> = u128_to_path(encoded).unwrap();
    assert_eq!(path, decoded);
}

#[test]
fn single_element() {
    let path: Vec<u8> = vec![0];
    let encoded = path_to_u128(&path).unwrap();
    assert_eq!(encoded, 1);
    let decoded: Vec<u8> = u128_to_path(encoded).unwrap();
    assert_eq!(path, decoded);
}

#[test]
fn different_lengths_differ() {
    let a = path_to_u128(&[0u8]).unwrap();
    let b = path_to_u128(&[0u8, 0]).unwrap();
    assert_ne!(a, b);
}

#[test]
fn max_length() {
    let path: Vec<u8> = vec![254; 16];
    let encoded = path_to_u128(&path).unwrap();
    let decoded: Vec<u8> = u128_to_path(encoded).unwrap();
    assert_eq!(path, decoded);
}

#[test]
fn too_long() {
    let path: Vec<u8> = vec![0; 17];
    assert!(path_to_u128(&path).is_err());
}

#[test]
fn value_too_large() {
    let path: Vec<u16> = vec![255];
    assert!(path_to_u128(&path).is_err());
}

#[test]
fn b62_is_fixed_width() {
    // Both a tiny and a moderate path encode to the same width,
    // proving callers can length-check the suffix.
    let small = path_to_b62(&[0u64]).unwrap();
    let bigger = path_to_b62(&[3u64, 200, 0, 42]).unwrap();
    assert_eq!(small.len(), PATH_B62_LEN);
    assert_eq!(bigger.len(), PATH_B62_LEN);
}

#[test]
fn b62_decode_rejects_short_input() {
    // Used to misparse user-chosen short suffixes (`-v`, `-final`,
    // ...) as real paths and replace them with computed paths.
    for s in ["v", "final", "x", "abc"] {
        assert!(b62_to_path::<u64>(s).is_err(), "{s:?} must not parse");
    }
}

#[test]
fn b62_decode_rejects_too_long_input() {
    // Anything longer than PATH_B62_LEN is also not a valid path
    // encoding — the encoder always pads to exactly PATH_B62_LEN.
    let s = "a".repeat(PATH_B62_LEN + 1);
    assert!(b62_to_path::<u64>(&s).is_err());
}

#[test]
fn b62_decode_accepts_fixed_width_zero_padded() {
    // Real path encodings round-trip through the strict decoder.
    for path in [vec![0u64], vec![3, 200, 0, 42], vec![254; 16]] {
        let b62 = path_to_b62(&path).unwrap();
        assert_eq!(b62.len(), PATH_B62_LEN);
        let back: Vec<u64> = b62_to_path(&b62).unwrap();
        assert_eq!(back, path);
    }
}
