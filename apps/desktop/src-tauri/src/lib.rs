//! Panasonic DMR-EH55 DIGA Engineering Suite - Desktop Backend Entrypoint

pub mod commands;
pub mod pico_serial;

use commands::*;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            clean_nvram_command,
            inspect_hdd_command,
            decode_meihdfs_command,
            extract_hdd_partitions_command,
            analyze_frm_command,
            dump_frm_toc_command,
            get_frm_codebook_command,
            search_manuals_command,
            list_pico_serial_ports,
            pico_connect_and_test,
            pico_execute_cold_break_in,
            pico_execute_in_circuit_erase,
            pico_switch_to_gdb_mode,
            start_gdb_bridge_command,
            stop_gdb_bridge_command,
            get_gdb_bridge_status_command,
            get_am34_registers_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
