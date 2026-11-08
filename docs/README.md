# Panadiga Documentation Gateway / Portal de Documentação

Welcome to the **Panadiga** technical documentation portal. The documentation is organized by language locale into dedicated directories:

Bem-vindo ao portal de documentação técnica do ecossistema **Panadiga**. A documentação está estruturada por idioma em diretórios dedicados:

---

## 🇺🇸 Canonical Technical English (`docs/en-US/`)

High-standard engineering documentation, technical blueprints, and specifications in standard English:

| Document | Technical Domain | Description |
| :--- | :--- | :--- |
| [**BLUEPRINT_BENCH_WIRING_AND_SCHEMATIC.md**](en-US/BLUEPRINT_BENCH_WIRING_AND_SCHEMATIC.md) | Hardware / Electronics | Wiring harness, P7501 connector pinout, RP2040 hookup, prototype vs production resistors. |
| [**STORAGE_ARCHITECTURE_MEIHDFS_PARTITIONS.md**](en-US/STORAGE_ARCHITECTURE_MEIHDFS_PARTITIONS.md) | Storage / Forensics | Panasonic MEIHFS, LBA 0 "SH" MBR, 28 MB mirror, 8 MB journal, and firmware partitions. |
| [**DOSSIER_CRYPTOGRAPHY_AND_FIRMWARE_PANA_DVD.md**](en-US/DOSSIER_CRYPTOGRAPHY_AND_FIRMWARE_PANA_DVD.md) | Firmware / Cryptanalysis | Optical container PANA_DVD.FRM, CPRM/C2 ciphers, 64-bit ECB codebooks, and entropy analysis. |
| [**GHIDRA_GDB_INTEGRATION_STRATEGY.md**](en-US/GHIDRA_GDB_INTEGRATION_STRATEGY.md) | Reverse Engineering | AM34/MN103S processor architecture, Ghidra SLED module, and GDB Remote Serial Protocol (:3333). |
| [**BENCH_PROBE_ASSEMBLY_AND_PHOTOS_GUIDE.md**](en-US/BENCH_PROBE_ASSEMBLY_AND_PHOTOS_GUIDE.md) | Laboratory / Assembly | Step-by-step soldering, JST ZR 1.5mm female socket mating, Pin 4 GND continuity verification. |
| [**TECHNICAL_INVENTORY_AND_OPERATIONS_MANUAL.md**](en-US/TECHNICAL_INVENTORY_AND_OPERATIONS_MANUAL.md) | Operations / Inventory | File inventory, cryptographic SHA-256 manifests, and step-by-step U99 unbrick runbook. |
| [**FINAL_AUDIT_AND_MODULARIZATION_REPORT.md**](en-US/FINAL_AUDIT_AND_MODULARIZATION_REPORT.md) | Software Architecture | Modularization audit, cross-stack integration (Rust/C/Python/TypeScript), 9 quality gates status. |
| [**LOGGING_AND_TELEMETRY_SUBSYSTEM.md**](en-US/LOGGING_AND_TELEMETRY_SUBSYSTEM.md) | Systems / Telemetry | Structured JSON logging, thread-safe sinks, and real-time serial terminal integration. |
| [**THEORY_AND_PRACTICE_U99_UNBRICK_MATSUSHITA_CRYPTO.md**](en-US/THEORY_AND_PRACTICE_U99_UNBRICK_MATSUSHITA_CRYPTO.md) | Silicon / Physics | Semiconductor physics of NOR Flash, EEMUL circular wear-leveling, and 64 KB NVRAM sanitization. |
| [**ZERO_ERROR_RAG_ENGINE_AND_PHYSICAL_SAFETY.md**](en-US/ZERO_ERROR_RAG_ENGINE_AND_PHYSICAL_SAFETY.md) | AI Safety / Invariants | 6-layer defense system (QueryGuard, CRAG, GroundedSynthesis, FactCritic, InvariantShield). |

---

## 🇧🇷 Dossiês Técnicos de Bancada em Português (`docs/pt-BR/`)

Documentação exaustiva de bancada, esquemáticos e manuais de engenharia reversa em Português:

| Dossiê | Domínio Técnico | Descrição |
| :--- | :--- | :--- |
| [**BLUEPRINT_ESQUEMA_ELETRICO_BANCADA.md**](pt-BR/BLUEPRINT_ESQUEMA_ELETRICO_BANCADA.md) | Hardware / Eletrônica | Esquema de bancada, conector P7501, RP2040, disclaimer de protótipo vs produto final. |
| [**ARQUITETURA_STORAGE_MEIHDFS_PARTICOES.md**](pt-BR/ARQUITETURA_STORAGE_MEIHDFS_PARTICOES.md) | Armazenamento / MEIHDFS | MBR 'SH' em LBA 0, espelho a 28 MB, journal transacional e partições de firmware. |
| [**DOSSIE_CRIPTOGRAFIA_E_FIRMWARE_PANA_DVD.md**](pt-BR/DOSSIE_CRIPTOGRAFIA_E_FIRMWARE_PANA_DVD.md) | Firmware / Criptoanálise | Contêiner óptico PANA_DVD.FRM, cifras CPRM/C2, codebook ECB e análise de entropia. |
| [**ESTRATEGIA_INTEGRACAO_GHIDRA_GDB.md**](pt-BR/ESTRATEGIA_INTEGRACAO_GHIDRA_GDB.md) | Engenharia Reversa | Processador Matsushita AM34/MN103S, scripts Ghidra e servidor GDB RSP (:3333). |
| [**GUIA_MONTAGEM_E_FOTOS_PROBE_BANCADA.md**](pt-BR/GUIA_MONTAGEM_E_FOTOS_PROBE_BANCADA.md) | Bancada / Montagem | Procedimento ilustrado de montagem, chicote JST ZR 1.5mm e teste de terra (GND Pino 4). |
| [**INVENTARIO_TECNICO_E_GUIA_DE_OPERACAO.md**](pt-BR/INVENTARIO_TECNICO_E_GUIA_DE_OPERACAO.md) | Operações / Inventário | Inventário de arquivos, hashes SHA-256 e roteiro passo a passo de desbrick do erro U99. |
| [**RELATORIO_AUDITORIA_E_MODULARIZACAO_FINAL.md**](pt-BR/RELATORIO_AUDITORIA_E_MODULARIZACAO_FINAL.md) | Arquitetura de Software | Relatório de modularização, integração cross-stack e homologação dos 9 Quality Gates. |
| [**SUBSISTEMA_LOGGING_E_TELEMETRIA.md**](pt-BR/SUBSISTEMA_LOGGING_E_TELEMETRIA.md) | Sistemas / Telemetria | Logging unificado, sinks thread-safe e telemetria serial em tempo real na interface gráfica. |
| [**TEORIA_E_PRATICA_UNBRICK_U99_E_CRIPTOGRAFIA_MATSUSHITA.md**](pt-BR/TEORIA_E_PRATICA_UNBRICK_U99_E_CRIPTOGRAFIA_MATSUSHITA.md) | Física / Semicondutores | Física da Flash NOR Spansion, motor EEMUL circular e sanitização dos 64 KB de NVRAM para 0xFF. |
| [**ZERO_ERROR_RAG_ENGINE_E_SEGURANCA_FISICA.md**](pt-BR/ZERO_ERROR_RAG_ENGINE_E_SEGURANCA_FISICA.md) | Segurança de IA / RAG | Defesa em 6 camadas determinísticas (QueryGuard, CRAG, FactCritic, InvariantShield). |
