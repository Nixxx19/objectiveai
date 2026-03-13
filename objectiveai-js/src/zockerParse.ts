const NUMBER_MIN = 0;
const NUMBER_MAX = 999;

function fixForSerde<T>(value: T): T {
  if (typeof value === "number") {
    if (!Number.isFinite(value) || value < NUMBER_MIN || value > NUMBER_MAX) {
      return (Math.floor(Math.random() * (NUMBER_MAX - NUMBER_MIN + 1)) + NUMBER_MIN) as T;
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
