import { describe, it, expect } from "vitest";
import {
  formatHex32,
  formatVoltage,
  isTargetPowered,
  formatElapsedMs,
} from "./formatters";

describe("formatters pure utility functions", () => {
  describe("formatHex32", () => {
    it("formats 0 as 0x00000000", () => {
      expect(formatHex32(0)).toBe("0x00000000");
    });

    it("formats standard 32-bit integer", () => {
      expect(formatHex32(0x80000000)).toBe("0x80000000");
      expect(formatHex32(0x1000ffff)).toBe("0x1000FFFF");
    });

    it("handles negative numbers via unsigned right shift (val >>> 0)", () => {
      expect(formatHex32(-1)).toBe("0xFFFFFFFF");
    });

    it("falls back to defaultVal on null or undefined", () => {
      expect(formatHex32(null)).toBe("0x00000000");
      expect(formatHex32(undefined)).toBe("0x00000000");
      expect(formatHex32(undefined, "N/A")).toBe("N/A");
      expect(formatHex32(NaN, "INVALID")).toBe("INVALID");
    });
  });

  describe("formatVoltage", () => {
    it("formats floating point voltage with 2 decimals", () => {
      expect(formatVoltage(3.324)).toBe("3.32");
      expect(formatVoltage(5)).toBe("5.00");
      expect(formatVoltage(0)).toBe("0.00");
    });

    it("handles null and undefined safely", () => {
      expect(formatVoltage(null)).toBe("0.00");
      expect(formatVoltage(undefined)).toBe("0.00");
      expect(formatVoltage(NaN)).toBe("0.00");
    });
  });

  describe("isTargetPowered", () => {
    it("returns true when voltage exceeds or equals threshold", () => {
      expect(isTargetPowered(3.3, 2.5)).toBe(true);
      expect(isTargetPowered(2.5, 2.5)).toBe(true);
    });

    it("returns false when voltage is below threshold or null", () => {
      expect(isTargetPowered(1.8, 2.5)).toBe(false);
      expect(isTargetPowered(0.2, 2.5)).toBe(false);
      expect(isTargetPowered(null)).toBe(false);
      expect(isTargetPowered(undefined)).toBe(false);
    });
  });

  describe("formatElapsedMs", () => {
    it("formats milliseconds correctly", () => {
      expect(formatElapsedMs(120)).toBe("120ms");
      expect(formatElapsedMs(1500)).toBe("1,500ms");
      expect(formatElapsedMs(0)).toBe("0ms");
      expect(formatElapsedMs(null)).toBe("0ms");
    });
  });
});
