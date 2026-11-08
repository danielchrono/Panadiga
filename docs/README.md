# Panadiga Documentation Gateway / Portal do Acervo Técnico de Documentação

Welcome to the **Panadiga** technical documentation portal. The documentation is organized by language locale into dedicated directories:

Bem-vindo ao portal mestre do acervo de documentação técnica do ecossistema **Panadiga**. O acervo está categorizado e estruturado por idioma em diretórios dedicados:

---

## 🇺🇸 Canonical Technical English (`docs/en-US/`)

High-standard engineering documentation, technical blueprints, and specifications in standard English:

| Document | Technical Domain | Description |
| :--- | :--- | :--- |
| [**SYSTEM_ARCHITECTURE_AND_ENGINEERING_BLUEPRINT.md**](en-US/SYSTEM_ARCHITECTURE_AND_ENGINEERING_BLUEPRINT.md) | Systems Architecture | Master specification: modular decomposition, Pure FP domain, strict MVVM, GDB ring buffer, and silicon invariants. |
| [**ANDROID_APK_AND_BLUETOOTH_ARCHITECTURE.md**](en-US/ANDROID_APK_AND_BLUETOOTH_ARCHITECTURE.md) | Mobile / Wireless | Android APK (`aarch64-linux-android`), Native Rust HAL (BLE NUS / USB-C OTG), and P7501 power rules. |
| [**TESTING_ARCHITECTURE_AND_COMPLIANCE.md**](en-US/TESTING_ARCHITECTURE_AND_COMPLIANCE.md) | Quality & Verification | 5-layer test pyramid (Pure FP, GDB RSP, MVVM ADTs, CLI SSOT, E2E), traceability matrix, and CI/CD verification status. |
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
| [**ARQUITETURA_E_BLUEPRINT_DE_ENGENHARIA.md**](pt-BR/ARQUITETURA_E_BLUEPRINT_DE_ENGENHARIA.md) | Arquitetura de Sistemas | Especificação mestre: decomposição modular, Pure FP domain, MVVM rigoroso, anel GDB e invariantes de silício. |
| [**ARQUITETURA_MOBILE_ANDROID_E_BLUETOOTH.md**](pt-BR/ARQUITETURA_MOBILE_ANDROID_E_BLUETOOTH.md) | Mobile / Sem Fio | APK Android (`aarch64-linux-android`), HAL nativa em Rust (BLE NUS / USB-C OTG) e isolamento da porta P7501. |
| [**ARQUITETURA_DE_TESTES_E_CONFORMIDADE.md**](pt-BR/ARQUITETURA_DE_TESTES_E_CONFORMIDADE.md) | Qualidade & Testes | Pirâmide de testes em 5 camadas (Pure FP, GDB RSP, MVVM ADTs, CLI SSOT, E2E), matriz de rastreabilidade e status CI/CD. |
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
