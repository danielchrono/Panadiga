/**
 * Functional Programming (FP) Pure Formatting Utilities
 * All functions are pure, deterministic, and free of side effects.
 */

import {
  PICO_MAX_VDD_CEILING_VOLTS,
  PICO_MIN_VDD_THRESHOLD_VOLTS,
  PICO_SAFE_OPERATING_MIN_VOLTS,
} from '@/constants';

/**
 * Pure function: formats a 32-bit unsigned number into 8-digit uppercase hex representation with 0x prefix.
 *
 * @param val Number to format
 * @param defaultVal Fallback string when undefined or null
 * @returns Formatted hex string e.g. "0x80000000"
 */
export const formatHex32 = (val: number | undefined | null, defaultVal = '0x00000000'): string => {
  if (val === undefined || val === null || isNaN(val)) {
    return defaultVal;
  }
  return `0x${(val >>> 0).toString(16).padStart(8, '0').toUpperCase()}`;
};

/**
 * Pure function: formats a float voltage into a 2-decimal string.
 *
 * @param volts Measured voltage
 * @returns Formatted string e.g. "3.32"
 */
export const formatVoltage = (volts: number | undefined | null): string => {
  if (volts === undefined || volts === null || isNaN(volts)) {
    return '0.00';
  }
  return volts.toFixed(2);
};

/**
 * Pure predicate: determines whether target reference voltage indicates a powered board.
 *
 * @param volts Measured voltage
 * @param threshold Minimum voltage threshold (defaults to 2.5V)
 * @returns boolean
 */
export const isTargetPowered = (
  volts: number | undefined | null,
  threshold = PICO_MIN_VDD_THRESHOLD_VOLTS
): boolean => {
  if (volts === undefined || volts === null || isNaN(volts)) {
    return false;
  }
  return volts >= threshold;
};

/**
 * Pure predicate: determines whether measured VDD rail lies within nominal 3.3V LVTTL tolerance.
 */
export const isTargetVoltageNominal = (
  volts: number | undefined | null,
  minVolts = PICO_SAFE_OPERATING_MIN_VOLTS,
  maxVolts = PICO_MAX_VDD_CEILING_VOLTS
): boolean => {
  if (volts === undefined || volts === null || isNaN(volts)) {
    return false;
  }
  return volts >= minVolts && volts <= maxVolts;
};

/**
 * Pure function: formats millisecond duration into human-readable latency.
 */
export const formatElapsedMs = (ms: number | undefined | null): string => {
  if (!ms || ms < 0) return '0ms';
  return `${ms.toLocaleString('en-US')}ms`;
};
