#[test]
fn test_gdb_rsp_checksum_calculation() {
    fn calc_gdb_checksum(payload: &str) -> String {
        let sum: u32 = payload.bytes().map(|b| b as u32).sum();
        format!("{:02x}", (sum % 256) as u8)
    }

    assert_eq!(calc_gdb_checksum("qSupported"), "37");
    assert_eq!(calc_gdb_checksum("g"), "67");
    assert_eq!(calc_gdb_checksum("?"), "3f");
    assert_eq!(calc_gdb_checksum("c"), "63");
    assert_eq!(calc_gdb_checksum("s"), "73");
    assert_eq!(calc_gdb_checksum("OK"), "9a");
}
