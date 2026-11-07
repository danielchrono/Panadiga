import { useCallback } from 'react';
import { useTranslation as useI18nextTranslation } from 'react-i18next';
import i18n from './i18n';
import { Locale, TranslationKey, translations } from './translations';

export type { Locale, TranslationKey };
export { translations };

export const setGlobalLocale = (newLocale: Locale): void => {
  i18n.changeLanguage(newLocale);
};

export const getGlobalLocale = (): Locale => {
  return (i18n.language as Locale) || 'en-US';
};

export const translate = (key: TranslationKey | string, fallback?: string): string => {
  const res = i18n.t(key);
  if (res === key && fallback) {
    return fallback;
  }
  return res;
};

/**
 * Functional Programming Hook: Provides declarative, strongly-typed i18n translations
 * backed by i18next with automatic global synchronization across all mounted components.
 */
export function useTranslation() {
  const { t: i18nTranslate, i18n: instance } = useI18nextTranslation();

  const setLocale = useCallback((newLocale: Locale) => {
    setGlobalLocale(newLocale);
  }, []);

  const t = useCallback(
    (key: TranslationKey | string, fallbackOrOptions?: string | Record<string, unknown>): string => {
      if (typeof fallbackOrOptions === 'string') {
        const res = i18nTranslate(key);
        return res === key ? fallbackOrOptions : res;
      }
      if (fallbackOrOptions) {
        return i18nTranslate(key, fallbackOrOptions);
      }
      return i18nTranslate(key);
    },
    [i18nTranslate]
  );

  return {
    t,
    locale: (instance.language as Locale) || 'en-US',
    setLocale,
    i18n: instance,
  };
}
