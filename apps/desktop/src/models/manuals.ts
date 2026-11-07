/**
 * Panasonic Service Manual & Schematics Search Models
 * Matches src/manuals.rs & src/constants.rs
 */

export interface SearchMatchSnippet {
  term: string;
  page: number;
  context: string;
}

export interface PdfSearchResult {
  file_name: string;
  file_path: string;
  total_pages: number;
  matches: Record<string, number[]>;
}

export interface ManualSearchResult {
  file_name: string;
  file_path: string;
  total_pages: number;
  title: string;
  category: 'service_manual' | 'schematic' | 'operating_instructions' | 'bulletin';
  total_match_count: number;
  matches: Record<string, number[]>;
  snippets: SearchMatchSnippet[];
}

export interface ManualDocumentMetadata {
  id: string;
  file_name: string;
  file_path: string;
  title: string;
  description: string;
  category: 'service_manual' | 'schematic' | 'operating_instructions' | 'bulletin';
  size_bytes: number;
  page_count: number;
  key_ics: string[];
}
