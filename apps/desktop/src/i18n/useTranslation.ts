import { useState, useCallback } from 'react';
import { translations, Locale, TranslationKey } from './translations';

let currentGlobalLocale: Locale = 'en-US';
const listeners = new Set<(locale: Locale) => void>();

export const setGlobalLocale = (newLocale: Locale): void => {
  currentGlobalLocale = newLocale;
  listeners.forEach(fn => fn(newLocale));
};

export const getGlobalLocale = (): Locale => currentGlobalLocale;

/**
 * Functional Programming Hook: Provides declarative, strongly-typed i18n translations.
 */
export function useTranslation() {
  const [locale, setLocaleState] = useState<Locale>(currentGlobalLocale);

  const setLocale = useCallback((newLocale: Locale) => {
    setGlobalLocale(newLocale);
    setLocaleState(newLocale);
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
