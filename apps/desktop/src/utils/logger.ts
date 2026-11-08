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

class PanadigaLogger {
  private sinks: Set<LogSink> = new Set();
  private minLevel: LogSeverity = 'info';

  private severityWeight: Record<LogSeverity, number> = {
    debug: 0,
    info: 1,
    warn: 2,
    error: 3,
  };

  constructor() {
    // Default console sink routing
    if (typeof window !== 'undefined') {
      this.sinks.add(entry => {
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
  }

  public setMinLevel(level: LogSeverity): void {
    this.minLevel = level;
  }

  public addSink(sink: LogSink): () => void {
    this.sinks.add(sink);
    return () => {
      this.sinks.delete(sink);
    };
  }

  private dispatch(severity: LogSeverity, subsystem: string, message: string, details?: unknown): void {
    if (this.severityWeight[severity] < this.severityWeight[this.minLevel]) {
      return;
    }

    const entry: LogMessage = {
      timestamp: new Date().toTimeString().split(' ')[0],
      severity,
      subsystem,
      message,
      details,
    };

    this.sinks.forEach(sink => {
      try {
        sink(entry);
      } catch {
        // Logging failures must never interrupt application execution
      }
    });
  }

  public debug(subsystem: string, message: string, details?: unknown): void {
    this.dispatch('debug', subsystem, message, details);
  }

  public info(subsystem: string, message: string, details?: unknown): void {
    this.dispatch('info', subsystem, message, details);
  }

  public warn(subsystem: string, message: string, details?: unknown): void {
    this.dispatch('warn', subsystem, message, details);
  }

  public error(subsystem: string, message: string, details?: unknown): void {
    this.dispatch('error', subsystem, message, details);
  }
}

export const logger = new PanadigaLogger();
