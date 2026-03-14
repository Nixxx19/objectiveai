const NEGATIVE_NUMBER_MIN = -1000;
const POSITIVE_NUMBER_MAX = 1000;

function fixForSerde<T>(value: T): T {
  if (typeof value === "number") {
    if (!Number.isFinite(value)) {
      return 0 as T;
    }
    if (value < NEGATIVE_NUMBER_MIN) {
      return (Math.floor(Math.random() * Math.abs(NEGATIVE_NUMBER_MIN)) * -1) as T;
    }
    if (value > POSITIVE_NUMBER_MAX) {
      return Math.floor(Math.random() * POSITIVE_NUMBER_MAX) as T;
    }
    return value;
  } else if (value !== null && typeof value === "object") {
    const obj = value as Record<string, unknown>;
    for (const k in obj) {
      obj[k] = fixForSerde(obj[k]);
    }
    return value;
  } else {
    return value;
  }
}

export function zockerParse<T>(gen: { generate(): T }, normalize: (value: T) => string): T {
  let raw: T;
  for (let attempt = 0; ; attempt++) {
    try {
      raw = gen.generate();
      break;
    } catch (e: any) {
      if (attempt >= 99) throw e;
    }
  }
  const fixed = fixForSerde(raw);
  return JSON.parse(normalize(fixed));
}
