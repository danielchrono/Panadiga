/**
 * @file constants/index.ts
 * @brief Single Source of Truth (SSOT) barrel for all hardware physical parameters,
 *        canonical SSOT builders, forensic hex bookmarks, and UI interaction timing constants.
 */

export * from './hardwareSpecs';
export {
  DIGA_HEX_BOOKMARKS,
  HEX_FORENSIC_BOOKMARKS,
  getHexForensicBookmarks,
} from './hexBookmarks';

/* =========================================================================
 * UI Interaction & Polling Timing Constants
 * ========================================================================= */

/** Periodic telemetry polling interval in milliseconds */
export const TELEMETRY_POLL_INTERVAL_MS = 2000;

/** Toast notification visible duration in milliseconds */
export const TOAST_DURATION_MS = 3000;

/** Clipboard copy feedback state duration in milliseconds */
export const CLIPBOARD_FEEDBACK_MS = 2000;
