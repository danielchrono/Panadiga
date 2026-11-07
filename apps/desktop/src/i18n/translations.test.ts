import { describe, it, expect, beforeEach } from 'vitest';
import { translations, TranslationKey } from './translations';
import { useTranslation, setGlobalLocale, getGlobalLocale } from './useTranslation';
import { renderHook, act } from '@testing-library/react';

describe('i18n strictly-typed translations', () => {
  beforeEach(() => {
    setGlobalLocale('en-US');
  });

  it('defaults to en-US as the primary locale', () => {
    expect(getGlobalLocale()).toBe('en-US');
    const { result } = renderHook(() => useTranslation());
    expect(result.current.locale).toBe('en-US');
  });

  it('guarantees key parity between en-US and pt-BR', () => {
    const enKeys = Object.keys(translations['en-US']) as TranslationKey[];
    const ptKeys = Object.keys(translations['pt-BR']) as TranslationKey[];

    expect(enKeys.sort()).toEqual(ptKeys.sort());
  });

  it('provides all critical Zero-Error RAG translation keys', () => {
    const { result } = renderHook(() => useTranslation());

    expect(result.current.t('rag.title')).toBe('Zero-Error AI Engineering Assistant');
    expect(result.current.t('rag.badge')).toBe('GROUNDED CRAG & SELF-RAG');
    expect(result.current.t('rag.safetyPassed')).toContain('HARDWARE INVARIANTS VERIFIED');
    expect(result.current.t('rag.safetyBlocked')).toContain('BLOCKED BY HARDWARE SAFETY SHIELD');
    expect(result.current.t('rag.groundedBadge')).toBe('100% GROUNDED IN CORPUS');
  });

  it('translates correctly when switching locale to pt-BR', () => {
    const { result } = renderHook(() => useTranslation());

    act(() => {
      result.current.setLocale('pt-BR');
    });

    expect(result.current.locale).toBe('pt-BR');
    expect(result.current.t('rag.title')).toBe('Assistente Técnico RAG Zero-Erro');
    expect(result.current.t('manuals.header.badge')).toBe('INDEXAÇÃO LOCAL RÁPIDA');
  });
});
