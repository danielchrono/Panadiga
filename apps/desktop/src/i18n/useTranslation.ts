import { useState, useCallback, useEffect } from 'react';
import { translations, Locale, TranslationKey } from './translations';

export type { Locale, TranslationKey };

let currentGlobalLocale: Locale = 'en-US';
const listeners = new Set<(locale: Locale) => void>();

export const setGlobalLocale = (newLocale: Locale): void => {
  currentGlobalLocale = newLocale;
  listeners.forEach(fn => fn(newLocale));
};

export const getGlobalLocale = (): Locale => currentGlobalLocale;

export const translate = (key: TranslationKey, fallback?: string): string => {
  const dict = translations[currentGlobalLocale] || translations['en-US'];
  return dict[key] ?? fallback ?? key;
};

/**
 * Functional Programming Hook: Provides declarative, strongly-typed i18n translations
 * with automatic global synchronization across all mounted components.
 */
export function useTranslation() {
  const [locale, setLocaleState] = useState<Locale>(currentGlobalLocale);

  useEffect(() => {
    listeners.add(setLocaleState);
    return () => {
      listeners.delete(setLocaleState);
    };
  }, []);

  const setLocale = useCallback((newLocale: Locale) => {
    setGlobalLocale(newLocale);
  }, []);

  const t = useCallback(
    (key: TranslationKey, fallback?: string): string => {
      const dict = translations[locale] || translations['en-US'];
      return dict[key] ?? fallback ?? key;
    },
    [locale]
  );

  return {
    t,
    locale,
    setLocale,
  };
}
