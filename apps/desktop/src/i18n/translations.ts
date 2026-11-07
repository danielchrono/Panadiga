/**
 * @file translations.ts
 * @brief Strictly-typed internationalization (i18n) dictionary for Panasonic DIGA Suite.
 *
 * Supports pt-BR (Primary) and en-US (Secondary) locales.
 * Enforces compile-time key validation.
 */

export type Locale = "pt-BR" | "en-US";

export const translations = {
  "pt-BR": {
    // App & Layout
    "app.title": "Panasonic DMR-EH55 DIGA Engineering Suite",
    "app.subtitle": "Bancada de Engenharia Reversa, Criptoanálise & Reparo de Hardware",
    "app.demoMode": "Modo Simulação",
    "app.liveMode": "Conexão Real",
    "app.loading": "Carregando subsistema de bancada...",

    // Navigation Views
    "nav.dashboard": "Visão Geral",
    "nav.dashboard.sub": "Status do Gravador & Binários",
    "nav.nvram": "Flash & NVRAM",
    "nav.nvram.sub": "Spansion SA134..SA141 U99",
    "nav.hdd": "Armazenamento HD",
    "nav.hdd.sub": "Partições MEIHDFS & MBR",
    "nav.picobench": "Bancada Pico",
    "nav.picobench.sub": "DWire32A & Servidor GDB",
    "nav.frmcrypto": "Cripto PANA_DVD",
    "nav.frmcrypto.sub": "Dicionário ECB & Entropia",
    "nav.manuals": "Manuais Técnicos",
    "nav.manuals.sub": "15 PDFs & Esquemáticos",
    "nav.hexinspector": "Inspetor Hex",
    "nav.hexinspector.sub": "Alinhamento LBA & Offsets",

    // Pico Bench
    "pico.title": "Bancada Raspberry Pi Pico & DWire32A OCD Master",
    "pico.port": "Porta Serial COM",
    "pico.connect": "Conectar",
    "pico.disconnect": "Desconectar",
    "pico.connected": "CONECTADO",
    "pico.disconnected": "DESCONECTADO",
    "pico.targetRail": "Tensão da Placa (VDD)",
    "pico.targetState": "Estado do Core AM34",
    "pico.chipId": "Identificador do Chip",
    "pico.inCircuitErase": "Injetar Loader & Limpar NVRAM",
    "pico.erasing": "Executando Apagamento In-Circuit...",
    "pico.gdbServer": "Servidor GDB Remote RSP (Porta TCP 3333)",
    "pico.gdbStart": "Iniciar Servidor GDB",
    "pico.gdbStop": "Parar Servidor GDB",
    "pico.gdbListening": "OUVINDO (3333)",
    "pico.gdbOffline": "OFFLINE",
    "pico.gdbClientActive": "CLIENTE ATIVO",
    "pico.extTools": "Ecossistema & Ferramentas Externas:",
    "pico.launchGdb": "GDB Multiarch",
    "pico.launchOpenocd": "OpenOCD AM34",
    "pico.launchGhidra": "Scripts Ghidra",
    "pico.launchHxd": "HxD Editor",
    "pico.launchTerminal": "PowerShell",

    // NVRAM View
    "nvram.title": "Flash Spansion S29JL064H & Cirurgia NVRAM U99",
    "nvram.cleanButton": "Executar Cirurgia no Dump",
    "nvram.cleaning": "Higienizando Micro-Setores...",
    "nvram.statusClean": "NVRAM HIGIENIZADA (U99 CORRIGIDO)",
    "nvram.statusCorrupted": "NVRAM CORROMPIDA (ERRO U99 PRESENTE)",
    "nvram.microsectorsTitle": "Mapa dos 8 Micro-Setores de Boot (SA134 a SA141)",
    "nvram.auditButton": "Ver Auditoria & Hashes SHA-256",

    // Common Actions
    "common.close": "Fechar",
    "common.copy": "Copiar",
    "common.copied": "Copiado!",
    "common.execute": "Executar",
    "common.searching": "Buscando...",
    "common.cancel": "Cancelar",
    "common.ok": "OK",
    "common.error": "Erro",
    "common.success": "Sucesso",
  },
  "en-US": {
    // App & Layout
    "app.title": "Panasonic DMR-EH55 DIGA Engineering Suite",
    "app.subtitle": "Reverse Engineering Bench, Cryptanalysis & Hardware Repair",
    "app.demoMode": "Demo Mode",
    "app.liveMode": "Live Hardware",
    "app.loading": "Loading engineering subsystem...",

    // Navigation Views
    "nav.dashboard": "Overview",
    "nav.dashboard.sub": "Recorder Status & Binaries",
    "nav.nvram": "Flash & NVRAM",
    "nav.nvram.sub": "Spansion SA134..SA141 U99",
    "nav.hdd": "HDD Storage",
    "nav.hdd.sub": "MEIHDFS Partitions & MBR",
    "nav.picobench": "Pico Bench",
    "nav.picobench.sub": "DWire32A & GDB Server",
    "nav.frmcrypto": "PANA_DVD Crypto",
    "nav.frmcrypto.sub": "ECB Codebook & Entropy",
    "nav.manuals": "Service Manuals",
    "nav.manuals.sub": "15 PDFs & Schematics",
    "nav.hexinspector": "Hex Inspector",
    "nav.hexinspector.sub": "LBA Alignment & Offsets",

    // Pico Bench
    "pico.title": "Raspberry Pi Pico Bench & DWire32A OCD Master",
    "pico.port": "Serial COM Port",
    "pico.connect": "Connect",
    "pico.disconnect": "Disconnect",
    "pico.connected": "CONNECTED",
    "pico.disconnected": "DISCONNECTED",
    "pico.targetRail": "Target Board Rail (VDD)",
    "pico.targetState": "AM34 Core State",
    "pico.chipId": "Chip Identifier",
    "pico.inCircuitErase": "Inject Loader & Erase NVRAM",
    "pico.erasing": "Executing In-Circuit Erase...",
    "pico.gdbServer": "GDB Remote RSP Server (TCP Port 3333)",
    "pico.gdbStart": "Start GDB Server",
    "pico.gdbStop": "Stop GDB Server",
    "pico.gdbListening": "LISTENING (3333)",
    "pico.gdbOffline": "OFFLINE",
    "pico.gdbClientActive": "CLIENT ACTIVE",
    "pico.extTools": "Ecosystem & External Tools:",
    "pico.launchGdb": "GDB Multiarch",
    "pico.launchOpenocd": "OpenOCD AM34",
    "pico.launchGhidra": "Ghidra Scripts",
    "pico.launchHxd": "HxD Editor",
    "pico.launchTerminal": "PowerShell",

    // NVRAM View
    "nvram.title": "Spansion S29JL064H Flash & NVRAM U99 Surgery",
    "nvram.cleanButton": "Perform Dump Surgery",
    "nvram.cleaning": "Sanitizing Micro-Sectors...",
    "nvram.statusClean": "NVRAM SANITIZED (U99 RESOLVED)",
    "nvram.statusCorrupted": "NVRAM CORRUPTED (U99 ERROR ACTIVE)",
    "nvram.microsectorsTitle": "Boot Micro-Sectors Map (SA134 to SA141)",
    "nvram.auditButton": "View Audit & SHA-256 Hashes",

    // Common Actions
    "common.close": "Close",
    "common.copy": "Copy",
    "common.copied": "Copied!",
    "common.execute": "Execute",
    "common.searching": "Searching...",
    "common.cancel": "Cancel",
    "common.ok": "OK",
    "common.error": "Error",
    "common.success": "Success",
  },
} as const;

export type TranslationKey = keyof typeof translations["pt-BR"];
