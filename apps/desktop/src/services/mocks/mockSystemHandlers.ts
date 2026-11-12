/**
 * Mock IPC handlers for system overview, manuals lookup, hex viewer, external tools, and RAG queries.
 */

import {
  generateMockHexDump,
  mockManualSearchResults,
  mockSystemOverview,
} from '../mockData';

export function handleMockSystemCommand(
  cmd: string,
  args?: Record<string, unknown>
): unknown | undefined {
  switch (cmd) {
    case 'get_system_overview':
      return mockSystemOverview;

    case 'search_manuals': {
      const query = (args?.query as string)?.toUpperCase();
      if (!query) {
        return mockManualSearchResults;
      }
      return mockManualSearchResults.filter(item => (
        item.title.toUpperCase().includes(query) ||
        item.file_name.toUpperCase().includes(query) ||
        Object.keys(item.matches).some(k => k.toUpperCase().includes(query)) ||
        item.snippets.some(s => s.context.toUpperCase().includes(query))
      ));
    }

    case 'read_hex_dump': {
      const offset = (args?.offset as number) ?? 0;
      const count = (args?.count as number) ?? 32;
      return generateMockHexDump(offset, count);
    }

    case 'launch_external_tool_command': {
      const tool = (args?.tool as string) || 'desconhecido';
      return `[Modo Demonstração] Ferramenta externa '${tool}' executada com sucesso.`;
    }

    case 'query_rag_engine': {
      const q = (args?.query as string) || '';
      return {
        query: q,
        answer:
          'Error U99 indicates NVRAM corruption on the Spansion S29JL064H NOR Flash (micro-sectors SA134..SA141). Erase sectors SA134..SA141 (0x007F0000..0x007FFFFF) to restore virgin 0xFF state while preserving bootloader IPL at SA0..SA133.',
        confidence: 0.98,
        groundedness_score: 1.0,
        is_abstained: false,
        abstention_reason: null,
        citations: [
          {
            doc: 'manuals_and_schematics/dmr-eh55/service/panasonic_dmr_eh55p9_eh55pc9_service_manual_north_america.pdf',
            section: 'Self-Diagnostic and Error Codes',
            lines: 'P.21-22',
            excerpt: 'U99: System microprocessor communication fault or NVRAM checksum mismatch.',
          },
        ],
        safety: {
          is_safe: true,
          violations: [],
          alerts: [],
        },
        critic: {
          is_faithful: true,
          ungrounded_entities: [],
        },
        retrieved_chunks_count: 3,
        execution_time_ms: 12,
      };
    }

    default:
      return undefined;
  }
}
