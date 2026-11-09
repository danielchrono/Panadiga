import { describe, it, expect, beforeEach } from 'vitest';
import fs from 'fs';
import path from 'path';
import { translations, TranslationKey } from './translations';
import { useTranslation, setGlobalLocale, getGlobalLocale } from './useTranslation';
import { useManualsViewModel } from '../viewmodels/useManualsViewModel';
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

  it('reactively updates ViewModel error message on locale switch (OPUS-09)', async () => {
    const { result } = renderHook(() => useManualsViewModel());

    // Trigger error by searching empty query
    act(() => {
      result.current.setQuery('');
    });

    await act(async () => {
      await result.current.executeSearch();
    });

    expect(result.current.errorMessage).toBe('Please enter a search keyword or part code.');

    // Switch locale globally to pt-BR
    act(() => {
      setGlobalLocale('pt-BR');
    });

    expect(result.current.errorMessage).toBe('Por favor, digite uma palavra-chave ou código de componente.');
  });

  it('verifies all Views integrate useTranslation strictly (OPUS-09 Static Scanner)', () => {
    const viewsDir = path.resolve(import.meta.dirname, '../views');
    const files = fs.readdirSync(viewsDir).filter((f: string) => f.endsWith('.tsx'));

    expect(files.length).toBeGreaterThanOrEqual(6);

    for (const file of files) {
      const filePath = path.join(viewsDir, file);
      const content = fs.readFileSync(filePath, 'utf-8');

      // Every view must import and invoke useTranslation
      expect(content).toContain('useTranslation');
      expect(content).toMatch(/const\s+\{\s*t(?:\s*,\s*locale)?\s*\}\s*=\s*useTranslation\(\)/);
    }
  });
});
