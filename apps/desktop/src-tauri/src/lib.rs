//! Panasonic DMR-EH55 DIGA Engineering Suite - Cross-Platform Desktop & Android Backend Library

pub mod commands;
pub mod pico;

pub use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    run_desktop_app();
}

pub fn run_desktop_app() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_system_overview,
            clean_nvram,
            analyze_flash_dump,
            analyze_live_nvram_bytes,
            clean_nvram_command,
            inspect_hdd,
            inspect_hdd_command,
            decode_meihdfs_allocations,
            decode_meihdfs_command,
            extract_partitions,
            extract_hdd_partitions_command,
            analyze_frm,
            analyze_frm_command,
            dump_frm_toc,
            dump_frm_toc_command,
            get_codebook,
            get_frm_codebook_command,
            list_frm_containers_command,
            pack_frm_command,
            unpack_extrhead_command,
            search_manuals,
            search_manuals_command,
            list_service_manuals,
            read_hex_dump,
            get_diga_model_catalog,
            list_pico_serial_ports,
            pico_get_status,
            pico_set_breakin_mode,
            pico_inject_ram_loader,
            pico_first_contact_backup,
            pico_send_serial_command,
            pico_connect_and_test,
            pico_execute_cold_break_in,
            pico_execute_in_circuit_erase,
            pico_switch_to_gdb_mode,
            start_gdb_bridge_command,
            stop_gdb_bridge_command,
            get_gdb_bridge_status_command,
            get_am34_registers_command,
            get_gdb_telemetry_command,
            launch_external_tool_command,
            query_rag_engine,
            get_hardware_ssot_spec,
            pico_execute_diagnostic,
            pico_start_sc0_bridge,
            pico_send_bridge_input,
            pico_stop_sc0_bridge,
            pico_get_bridge_status,
            pico_capture_nvram_live,
            set_pico_telemetry_stream,
            pico_detect_bootsel_drives,
            pico_trigger_bootsel_reboot,
            pico_flash_firmware,
            list_physical_drives,
            format_hdd_meihdfs,
            write_hdd_image,
            restore_hdd_mirrors,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
