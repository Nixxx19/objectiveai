import { describe, it, expect } from "vitest";
import { scoreColor, SCORE_COLORS } from "../types";

describe("scoreColor", () => {
  it("returns high for scores >= 0.5", () => {
    expect(scoreColor(0.5)).toBe(SCORE_COLORS.high);
    expect(scoreColor(0.8)).toBe(SCORE_COLORS.high);
    expect(scoreColor(1.0)).toBe(SCORE_COLORS.high);
  });

  it("returns midHigh for scores in [0.3, 0.5)", () => {
    expect(scoreColor(0.3)).toBe(SCORE_COLORS.midHigh);
    expect(scoreColor(0.4)).toBe(SCORE_COLORS.midHigh);
    expect(scoreColor(0.499)).toBe(SCORE_COLORS.midHigh);
  });

  it("returns midLow for scores in [0.15, 0.3)", () => {
    expect(scoreColor(0.15)).toBe(SCORE_COLORS.midLow);
    expect(scoreColor(0.2)).toBe(SCORE_COLORS.midLow);
    expect(scoreColor(0.299)).toBe(SCORE_COLORS.midLow);
  });

  it("returns low for scores < 0.15", () => {
    expect(scoreColor(0.0)).toBe(SCORE_COLORS.low);
    expect(scoreColor(0.1)).toBe(SCORE_COLORS.low);
    expect(scoreColor(0.149)).toBe(SCORE_COLORS.low);
  });

  it("boundary: exactly 0.5 is high", () => {
    expect(scoreColor(0.5)).toBe(SCORE_COLORS.high);
  });

  it("boundary: exactly 0.3 is midHigh", () => {
    expect(scoreColor(0.3)).toBe(SCORE_COLORS.midHigh);
  });

  it("boundary: exactly 0.15 is midLow", () => {
    expect(scoreColor(0.15)).toBe(SCORE_COLORS.midLow);
  });
});
