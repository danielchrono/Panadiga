# Panasonic DMR-EH50 Firmware Assets

- **Model:** Panasonic DMR-EH50 (Gen 3 Early DIGA)
- **SoC Architecture:** Matsushita UniPhier MN2DS0016 (AM34 RISC)
- **Optical Container:** `PANA_DVD.FRM` (Extracted from official `UPDATE_08_07_2006_DMR_EH50.zip`, 7.22 MB)
- **Cryptographic Characteristics:**
  - 64-bit ECB Matsushita cipher
  - Erased NOR Flash block: `79BB2C325AD4FD34` (4,107 occurrences)
  - Null padding block: `B3A25A0B9D864F08` (178 occurrences)
  - Global Header: `492F0560CEDE508CE3C08AFD49B64A9A` (16 bytes)
- **Usage:** Analyze with `panadiga frm analyze firmware_and_images/dmr-eh50/PANA_DVD.FRM`.
