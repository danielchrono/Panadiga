import { NvramCleanReport, FlashSectorInfo } from '@/models';

export const mockNvramCleanReport: NvramCleanReport = {
  input_file: 'firmware_and_images/spansion_s29jl064h_dump.bin',
  output_file: 'firmware_and_images/spansion_s29jl064h_dump_CLEAN.bin',
  backup_file: 'firmware_and_images/nvram_corrupt_backup.bin',
  original_md5: '7b9e4a91c849102c7e09ab12f458bc21',
  original_sha256: 'e8d62c140409a25b29339e088d1d8ef3f24b22c83ef87340ad0098dfec11aa52',
  clean_md5: '4a11f99c2d1b76402ea891001f357aa9',
  clean_sha256: '0a394c8e7710bcf1489e3a6a92ecda46e01768494c8fa72722ee20c5c363dc78',
  total_flash_bytes: 8388608,
  nvram_offset_start: 0x007f0000,
  nvram_offset_end: 0x00800000,
  nvram_non_ff_bytes: 18432,
  nvram_non_zero_bytes: 65536,
  bootloader_immutable: true,
  microsectors: [
    {
      sector_name: 'SA134',
      offset: 0x007f0000,
      active_bytes: 2048,
      total_bytes: 8192,
      header_hex: 'AA554D45494E5652',
    },
    {
      sector_name: 'SA135',
      offset: 0x007f2000,
      active_bytes: 1536,
      total_bytes: 8192,
      header_hex: '45454D554C010020',
    },
    {
      sector_name: 'SA136',
      offset: 0x007f4000,
      active_bytes: 3072,
      total_bytes: 8192,
      header_hex: '53454C4643484B31',
    },
    {
      sector_name: 'SA137',
      offset: 0x007f6000,
      active_bytes: 4096,
      total_bytes: 8192,
      header_hex: '5539395F45525230',
    },
    {
      sector_name: 'SA138',
      offset: 0x007f8000,
      active_bytes: 2560,
      total_bytes: 8192,
      header_hex: '4844445F53455249',
    },
    {
      sector_name: 'SA139',
      offset: 0x007fa000,
      active_bytes: 1024,
      total_bytes: 8192,
      header_hex: '524547494F4E5F55',
    },
    {
      sector_name: 'SA140',
      offset: 0x007fc000,
      active_bytes: 2048,
      total_bytes: 8192,
      header_hex: '43484B53554D5F45',
    },
    {
      sector_name: 'SA141',
      offset: 0x007fe000,
      active_bytes: 2048,
      total_bytes: 8192,
      header_hex: '544553545F544F4B',
    },
  ],
};

// Interactive Flash Map entries representing SA0..SA141
export const mockFlashSectorMap: FlashSectorInfo[] = Array.from({ length: 142 }, (_, i) => {
  const isMicroSector = i >= 134;
  const isBootloader = i < 16;
  const isOsFirmware = i >= 16 && i < 134;

  const startOffset = isMicroSector
    ? 0x007f0000 + (i - 134) * 8192
    : i * 65536;
  const sizeBytes = isMicroSector ? 8192 : 65536;
  const endOffset = startOffset + sizeBytes - 1;

  const sectorType = isMicroSector
    ? 'nvram_microsector'
    : isBootloader
    ? 'bootloader'
    : isOsFirmware
    ? 'os_firmware'
    : 'normal_sector';

  return {
    index: i,
    name: `SA${i}`,
    start_offset: startOffset,
    end_offset: endOffset,
    size_bytes: sizeBytes,
    sector_type: sectorType,
    is_corrupted_area: isMicroSector,
    status: isMicroSector ? 'corrupted' : isBootloader ? 'immutable' : 'clean',
    active_bytes: isMicroSector ? (i === 137 ? 4096 : 2048) : sizeBytes,
    notes: isMicroSector
      ? `Top Parameter Bank: SA${i} (EEMUL / U99 Root Cause)`
      : isBootloader
      ? `Immutable Primary AM34 Bootloader Bank (0x${startOffset.toString(16).toUpperCase()})`
      : `Main OS & Codec Executable Segment`,
  };
});
