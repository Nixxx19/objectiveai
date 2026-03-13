const NUMBER_MIN = 0;
const NUMBER_MAX = 999;

/**
 * Recursively sanitizes zocker-generated values for serde compatibility:
 * - Converts all Maps to plain objects
 * - Deletes keys with undefined values from objects
 * - Converts undefined to null inside arrays (matching JSON/serde behavior)
 * - Re-randomizes numbers outside [NUMBER_MIN, NUMBER_MAX] to avoid f64
 *   precision drift when Rust uses rust_decimal::Decimal for accumulation
 * - When deleteNull is true, also deletes keys with null values from objects.
 *   Use this for the initial accumulator to match serde's skip_serializing_if
 *   behavior (the server never sends null for those fields).
 *
 * Mutates arrays and objects in-place to minimize allocations.
 *
 * @param value - The value to sanitize
 * @param deleteNull - If true, strip null values from objects (default: false)
 * @returns The sanitized value
 */
export function sanitizeZocker<T>(value: T, deleteNull = false): T {
  if (typeof value === "number") {
    if (!Number.isFinite(value) || value < NUMBER_MIN || value > NUMBER_MAX) {
      return (Math.floor(Math.random() * (NUMBER_MAX - NUMBER_MIN + 1)) + NUMBER_MIN) as T;
    }
    return value;
  } else if (value instanceof Map) {
    const result: Record<string, unknown> = {};
    for (const [k, v] of value) {
      if (v === undefined || (deleteNull && v === null)) continue;
      result[String(k)] = sanitizeZocker(v, deleteNull);
    }
    return result as T;
  } else if (Array.isArray(value)) {
    for (let i = 0; i < value.length; i++) {
      value[i] = value[i] === undefined ? null : sanitizeZocker(value[i], deleteNull);
    }
    return value;
  } else if (value !== null && typeof value === "object") {
    const obj = value as Record<string, unknown>;
    for (const k in obj) {
      if (obj[k] === undefined || (deleteNull && obj[k] === null)) {
        delete obj[k];
      } else {
        obj[k] = sanitizeZocker(obj[k], deleteNull);
      }
    }
    return value;
  } else {
    return value;
  }
}
