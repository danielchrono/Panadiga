/**
 * @file logger.ts
 * @brief Dedicated Structured Logging Subsystem for Panadiga Frontend.
 *
 * Eliminates raw console.log, console.warn, and console.error calls,
 * providing structured severity levels, timestamps, subsystem tagging,
 * and extensible sinks for test and production diagnostics.
 */

export type LogSeverity = 'debug' | 'info' | 'warn' | 'error';

export interface LogMessage {
  timestamp: string;
  severity: LogSeverity;
  subsystem: string;
  message: string;
  details?: unknown;
}

export type LogSink = (entry: LogMessage) => void;

export interface StructuredLogger {
  readonly setMinLevel: (level: LogSeverity) => void;
  readonly addSink: (sink: LogSink) => () => void;
  readonly debug: (subsystem: string, message: string, details?: unknown) => void;
  readonly info: (subsystem: string, message: string, details?: unknown) => void;
  readonly warn: (subsystem: string, message: string, details?: unknown) => void;
  readonly error: (subsystem: string, message: string, details?: unknown) => void;
}

const SEVERITY_WEIGHT: Readonly<Record<LogSeverity, number>> = Object.freeze({
  debug: 0,
  info: 1,
  warn: 2,
  error: 3,
});

/**
 * Pure closure factory creating an isolated structured logger without classes or shared globals.
 */
export function createLogger(initialLevel: LogSeverity = 'info'): StructuredLogger {
  const sinks = new Set<LogSink>();
  let minLevel: LogSeverity = initialLevel;

  if (typeof window !== 'undefined') {
    sinks.add(entry => {
      const prefix = `[${entry.timestamp}] [${entry.severity.toUpperCase()}] [${entry.subsystem}]`;
      if (entry.severity === 'error') {
        // eslint-disable-next-line no-console
        console.error(prefix, entry.message, entry.details ?? '');
      } else if (entry.severity === 'warn') {
        // eslint-disable-next-line no-console
        console.warn(prefix, entry.message, entry.details ?? '');
      } else if (entry.severity === 'debug') {
        // eslint-disable-next-line no-console
        console.debug(prefix, entry.message, entry.details ?? '');
      } else {
        // eslint-disable-next-line no-console
        console.info(prefix, entry.message, entry.details ?? '');
      }
    });
  }

  const dispatch = (
    severity: LogSeverity,
    subsystem: string,
    message: string,
    details?: unknown
  ): void => {
    if (SEVERITY_WEIGHT[severity] < SEVERITY_WEIGHT[minLevel]) {
      return;
    }

    const entry: LogMessage = {
      timestamp: new Date().toTimeString().split(' ')[0],
      severity,
      subsystem,
      message,
      details,
    };

    sinks.forEach(sink => {
      try {
        sink(entry);
      } catch {
        // Logging failures must never interrupt application execution
      }
    });
  };

  return Object.freeze({
    setMinLevel: (level: LogSeverity) => {
      minLevel = level;
    },
    addSink: (sink: LogSink) => {
      sinks.add(sink);
      return () => {
        sinks.delete(sink);
      };
    },
    debug: (subsystem: string, message: string, details?: unknown) =>
      dispatch('debug', subsystem, message, details),
    info: (subsystem: string, message: string, details?: unknown) =>
      dispatch('info', subsystem, message, details),
    warn: (subsystem: string, message: string, details?: unknown) =>
      dispatch('warn', subsystem, message, details),
    error: (subsystem: string, message: string, details?: unknown) =>
      dispatch('error', subsystem, message, details),
  });
}

export const logger = createLogger();
