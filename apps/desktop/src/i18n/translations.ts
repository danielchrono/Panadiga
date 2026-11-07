/**
 * @file translations.ts
 * @brief Internationalization resources loaded directly from canonical JSON tables.
 *
 * Backed by locales/en-US.json and locales/pt-BR.json.
 */

import enUSJson from '../locales/en-US.json';
import ptBRJson from '../locales/pt-BR.json';

export type Locale = 'en-US' | 'pt-BR';

function flattenObject(obj: Record<string, unknown>, prefix = ''): Record<string, string> {
  const result: Record<string, string> = {};
  for (const [key, value] of Object.entries(obj)) {
    const fullKey = prefix ? `${prefix}.${key}` : key;
    if (typeof value === 'object' && value !== null && !Array.isArray(value)) {
      Object.assign(result, flattenObject(value as Record<string, unknown>, fullKey));
    } else {
      result[fullKey] = String(value);
    }
  }
  return result;
}

export const translations = {
  'en-US': flattenObject(enUSJson),
  'pt-BR': flattenObject(ptBRJson),
} as const;

export type TranslationKey = keyof typeof translations['en-US'];
