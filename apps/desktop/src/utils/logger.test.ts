import { describe, it, expect, vi, beforeEach } from 'vitest';
import { logger, LogMessage } from './logger';

describe('PanadigaLogger Subsystem', () => {
  beforeEach(() => {
    logger.setMinLevel('debug');
  });

  it('dispatches structured log messages to custom sinks', () => {
    const messages: LogMessage[] = [];
    const unsubscribe = logger.addSink(entry => messages.push(entry));

    logger.info('TEST_SUBSYSTEM', 'Test info message', { code: 42 });

    expect(messages).toHaveLength(1);
    expect(messages[0].severity).toBe('info');
    expect(messages[0].subsystem).toBe('TEST_SUBSYSTEM');
    expect(messages[0].message).toBe('Test info message');
    expect(messages[0].details).toEqual({ code: 42 });
    expect(typeof messages[0].timestamp).toBe('string');

    unsubscribe();
  });

  it('filters log messages below minLevel threshold', () => {
    const messages: LogMessage[] = [];
    const unsubscribe = logger.addSink(entry => messages.push(entry));

    logger.setMinLevel('warn');

    logger.debug('TEST', 'Debug message');
    logger.info('TEST', 'Info message');
    logger.warn('TEST', 'Warning message');
    logger.error('TEST', 'Error message');

    expect(messages).toHaveLength(2);
    expect(messages.map(m => m.severity)).toEqual(['warn', 'error']);

    unsubscribe();
  });

  it('supports unregistering sinks cleanly via cleanup callback', () => {
    const sink = vi.fn();
    const unsubscribe = logger.addSink(sink);

    logger.info('TEST', 'First message');
    expect(sink).toHaveBeenCalledTimes(1);

    unsubscribe();
    logger.info('TEST', 'Second message');
    expect(sink).toHaveBeenCalledTimes(1);
  });

  it('handles faulty sinks without interrupting logger dispatching', () => {
    const faultySink = vi.fn().mockImplementation(() => {
      throw new Error('Sink runtime failure');
    });
    const goodSink = vi.fn();

    const unsub1 = logger.addSink(faultySink);
    const unsub2 = logger.addSink(goodSink);

    expect(() => {
      logger.error('TEST', 'Resilience test');
    }).not.toThrow();

    expect(faultySink).toHaveBeenCalledTimes(1);
    expect(goodSink).toHaveBeenCalledTimes(1);

    unsub1();
    unsub2();
  });
});
