#!/usr/bin/env python3
"""
===============================================================================
Panasonic DMR-EH55 HDD Image Partition & MEIHDFS Storage Parser
===============================================================================
Engenharia Reversa de Armazenamento e Filesystem Proprietário Matsushita / Panasonic
Autor: Antigravity Filesystem & Storage Engineering Team
Uso:
    python scripts/parse_panasonic_hdd_partitions.py [OPCOES]

Exemplos:
    python scripts/parse_panasonic_hdd_partitions.py
    python scripts/parse_panasonic_hdd_partitions.py --extract
    python scripts/parse_panasonic_hdd_partitions.py --extract --outdir extracted_partitions
    python scripts/parse_panasonic_hdd_partitions.py --json report_hdd.json
===============================================================================
"""

import os
import sys
import argparse
import hashlib
import json
import math
from collections import Counter

# Constantes de Estrutura do Gravador Panasonic DMR-EH55
DEFAULT_IMAGE_PATH = os.path.join(
    "firmware_and_images", "panasonic DMR EH-55 Firmware hdd_100.bin"
)
SECTOR_SIZE = 512
PANASONIC_PARTITION_SIGNATURE = bytes.fromhex("51 87 13 8D 5B 0A FB 22")
PANASONIC_MBR_SIGNATURE = bytes.fromhex("00 00 00 00 01 00 53 48 62 8c 6e f5 1b 2c 36 66")


def calculate_entropy(data: bytes) -> float:
    """Calcula a Entropia de Shannon (0.0 a 8.0 bits/byte)."""
    if not data:
        return 0.0
    total = len(data)
    counts = Counter(data)
    return -sum((c / total) * math.log2(c / total) for c in counts.values())


def get_hashes(data: bytes):
    """Calcula MD5 e SHA-256 para um bloco de dados."""
    md5 = hashlib.md5(data).hexdigest()
    sha256 = hashlib.sha256(data).hexdigest()
    return md5, sha256


class PanasonicHDDParser:
    def __init__(self, image_path: str, verbose: bool = False):
        self.image_path = image_path
        self.verbose = verbose
        self.file_size = 0
        self.total_sectors = 0
        self.report = {}

    def load_and_verify(self):
        """Verifica existência e métricas globais da imagem de 100MB."""
        if not os.path.exists(self.image_path):
            raise FileNotFoundError(f"Imagem do HDD nao encontrada: {self.image_path}")

        self.file_size = os.path.getsize(self.image_path)
        self.total_sectors = self.file_size // SECTOR_SIZE

        with open(self.image_path, "rb") as f:
            full_data = f.read()

        full_md5, full_sha256 = get_hashes(full_data)

        self.report["image_info"] = {
            "path": os.path.abspath(self.image_path),
            "size_bytes": self.file_size,
            "size_mb": round(self.file_size / (1024 * 1024), 2),
            "total_sectors": self.total_sectors,
            "md5": full_md5,
            "sha256": full_sha256,
        }

        print("=" * 80)
        print(" PANASONIC DMR-EH55 HDD REVERSE ENGINEERING & PARTITION PARSER")
        print("=" * 80)
        print(f" Arquivo da Imagem : {self.image_path}")
        print(f" Tamanho Total     : {self.file_size:,} bytes ({self.file_size / (1024*1024):.2f} MB)")
        print(f" Total de Setores  : {self.total_sectors:,} LBAs (512 bytes/setor)")
        print(f" MD5 Global        : {full_md5}")
        print(f" SHA-256 Global    : {full_sha256}")
        print("-" * 80)

    def parse_mbr_tables(self):
        """Analisa o MBR mestre (LBA 0) e a tabela espelho (LBA 57344 / 28MB)."""
        with open(self.image_path, "rb") as f:
            # 1. LBA 0 (Master MBR)
            f.seek(0)
            lba0 = f.read(SECTOR_SIZE)

            # 2. LBA 57344 (Mirror MBR / 28 MB)
            mirror_offset = 57344 * SECTOR_SIZE
            f.seek(mirror_offset)
            lba_mirror = f.read(SECTOR_SIZE)

        # Decodificação de campos de LBA 0
        has_std_mbr_sig = (lba0[510:512] == b"\x55\xaa")
        sig_match = (lba0[:16] == PANASONIC_MBR_SIGNATURE)
        is_mirror_identical = (lba0 == lba_mirror)

        version = int.from_bytes(lba0[4:6], byteorder="little")
        magic_ascii = lba0[6:8].decode("latin1", errors="replace")
        volume_guid = lba0[8:16].hex().upper()

        # Checagem de zeros no restante
        zeros_after_header = all(b == 0 for b in lba0[16:])

        self.report["mbr_tables"] = {
            "lba0": {
                "offset": 0,
                "hex_sig": lba0[:16].hex(" "),
                "format_version": version,
                "magic_ascii": magic_ascii,
                "disk_volume_guid": volume_guid,
                "standard_mbr_0x55aa": has_std_mbr_sig,
                "clean_zeros_16_511": zeros_after_header,
                "md5": get_hashes(lba0)[0],
                "sha256": get_hashes(lba0)[1],
            },
            "lba57344_mirror": {
                "offset": mirror_offset,
                "offset_mb": mirror_offset / (1024 * 1024),
                "is_identical_to_lba0": is_mirror_identical,
                "md5": get_hashes(lba_mirror)[0],
                "sha256": get_hashes(lba_mirror)[1],
            },
        }

        print("\n[+] ESTRUTURA DO MASTER BOOT RECORD & TABELA ESPELHO (LBA 0 & LBA 57344)")
        print(f"  * LBA 0 (Offset 0x00000000):")
        print(f"    - Assinatura Bruta  : {lba0[:16].hex(' ')}")
        print(f"    - Versao de Formato : Rev {version}.0 (Bytes 0x04-0x05: 0x{version:04X})")
        print(f"    - Magic Tag         : '{magic_ascii}' (Bytes 0x06-0x07: 0x{lba0[6:8].hex().upper()} -> System Storage Header)")
        print(f"    - Disk Volume GUID  : {volume_guid} (Bytes 0x08-0x0F: ID exclusivo da formatacao)")
        print(f"    - MBR Padrao (0x55AA): {'SIM' if has_std_mbr_sig else 'NAO (Assinatura Proprietaria Panasonic MEIHDFS RAW)'}")
        print(f"    - Setor 0 (16..511) : {'100% Zeros limpos' if zeros_after_header else 'Possui bytes adicionais'}")

        print(f"  * LBA 57.344 (Offset 0x{mirror_offset:08X} = 28.00 MB):")
        print(f"    - Status do Espelho : {'IDENTICO BIT-A-BIT AO LBA 0 (Mirror Valido)' if is_mirror_identical else 'DIVERGENCIA DETECTADA'}")
        print(f"    - SHA-256 LBA 0     : {self.report['mbr_tables']['lba0']['sha256']}")
        print(f"    - SHA-256 LBA 57344 : {self.report['mbr_tables']['lba57344_mirror']['sha256']}")

    def parse_metadata_and_allocation(self):
        """Analisa os setores de metadados 24..31, LBA 16384, journal 0x00820000 e LBA 49152."""
        with open(self.image_path, "rb") as f:
            # Setores 24 a 31 (0x00003000 a 0x00004000)
            f.seek(24 * SECTOR_SIZE)
            sec_24_31 = f.read(8 * SECTOR_SIZE)

            # Mirror dos Setores 24 a 31 em 57368 a 57375
            f.seek(57368 * SECTOR_SIZE)
            mirror_24_31 = f.read(8 * SECTOR_SIZE)

            # LBA 16384 (0x00800000)
            f.seek(16384 * SECTOR_SIZE)
            lba_16384 = f.read(SECTOR_SIZE)

            # Região 0x00820000 a 0x00960000 (10 blocos de 128KB)
            f.seek(0x00820000)
            journal_data = f.read(0x00960000 - 0x00820000)

            # LBA 49152 (0x01800000 / 24 MB)
            f.seek(49152 * SECTOR_SIZE)
            lba_49152 = f.read(SECTOR_SIZE)

        is_metadata_mirror_identical = (sec_24_31 == mirror_24_31)
        j_block_count = int.from_bytes(lba_16384[0:4], byteorder="little")
        j_max_index = int.from_bytes(lba_16384[4:8], byteorder="little")

        # Inspeciona os 10 blocos de 128KB
        block_info = []
        for i in range(10):
            blk_offset = i * (256 * SECTOR_SIZE)
            blk_data = journal_data[blk_offset : blk_offset + (256 * SECTOR_SIZE)]
            head = blk_data[:16].hex(" ")
            # Últimos 16 bytes não-zero do bloco
            nz_indices = [idx for idx, b in enumerate(blk_data) if b != 0]
            last_nz = nz_indices[-1] if nz_indices else 0
            seq_trailer = blk_data[last_nz - 7 : last_nz + 1].hex(" ")
            block_info.append({
                "block_idx": i,
                "start_offset": 0x00820000 + blk_offset,
                "start_lba": 16640 + i * 256,
                "header_hex": head,
                "trailer_hex": seq_trailer,
            })

        self.report["allocation_and_journal"] = {
            "sectors_24_31": {
                "offset": 0x00003000,
                "size_bytes": len(sec_24_31),
                "is_mirror_identical": is_metadata_mirror_identical,
                "sha256": get_hashes(sec_24_31)[1],
            },
            "journal_catalog_lba16384": {
                "offset": 0x00800000,
                "total_blocks": j_block_count,
                "max_block_index": j_max_index,
                "header_hex": lba_16384[:16].hex(" "),
            },
            "journal_blocks": block_info,
            "boundary_marker_lba49152": {
                "offset": 0x01800000,
                "header_hex": lba_49152[:24].hex(" "),
            },
        }

        print("\n[+] METADADOS DE ALOCACAO & JOURNAL TRANSACIONAL")
        print(f"  * Setores 24 a 31 (0x00003000 a 0x00003FFF, 8 setores = 4.096 bytes):")
        print(f"    - Espelho em LBA 57368 (0x01C03000): {'100% IDENTICO (Integridade Confirmada)' if is_metadata_mirror_identical else 'Divergente'}")
        print(f"    - SHA-256 Descritores              : {self.report['allocation_and_journal']['sectors_24_31']['sha256']}")
        print(f"  * LBA 16.384 (Offset 0x00800000 = 8.00 MB):")
        print(f"    - Catalogo de Journal : {j_block_count} blocos de alocacao (Max Index: {j_max_index})")
        print(f"  * Regiao de Journal (0x00820000 a 0x00960000 = 1.310.720 bytes = 1.25 MB):")
        print(f"    - 10 Blocos alinhados a 256 setores (128 KB cada), iniciados com '5D FF FF FF...'")
        for blk in block_info:
            print(f"      [Bloco {blk['block_idx']}] LBA {blk['start_lba']:5d} (0x{blk['start_offset']:08X}) -> Trailer de Sequencia: {blk['trailer_hex']}")
        print(f"  * LBA 49.152 (Offset 0x01800000 = 24.00 MB):")
        print(f"    - Marcador de Transicao / Estado : {lba_49152[:24].hex(' ')}")

    def parse_firmware_partitions(self):
        """Descobre, valida e calcula métricas das partições de firmware."""
        with open(self.image_path, "rb") as f:
            data = f.read()

        # Localiza todas as ocorrências da assinatura de partição 51 87 13 8D 5B 0A FB 22
        positions = []
        pos = 0
        while True:
            idx = data.find(PANASONIC_PARTITION_SIGNATURE, pos)
            if idx == -1:
                break
            positions.append(idx)
            pos = idx + 1

        self.partitions = []

        # Partição 1: Offset 0x02000000 (32 MB, LBA 65536)
        if 0x02000000 in positions:
            p1_offset = 0x02000000
            p1_exact_size = 0x5E3250  # 6,173,264 bytes
            p1_aligned_size = 0x5E3400  # 6,173,696 bytes (12,058 setores)

            p1_payload = data[p1_offset : p1_offset + p1_exact_size]
            p1_aligned = data[p1_offset : p1_offset + p1_aligned_size]

            p1_md5, p1_sha256 = get_hashes(p1_payload)
            p1_s_md5, p1_s_sha256 = get_hashes(p1_aligned)
            p1_ent = calculate_entropy(p1_payload)

            part1 = {
                "partition_id": 1,
                "name": "Firmware Principal & Codecs (AM34 / DIGA OS)",
                "offset": p1_offset,
                "offset_hex": f"0x{p1_offset:08X}",
                "lba_start": p1_offset // SECTOR_SIZE,
                "exact_payload_size": p1_exact_size,
                "sector_aligned_size": p1_aligned_size,
                "sectors_count": p1_aligned_size // SECTOR_SIZE,
                "signature_hex": PANASONIC_PARTITION_SIGNATURE.hex(" "),
                "entropy": round(p1_ent, 4),
                "exact_payload_hashes": {"md5": p1_md5, "sha256": p1_sha256},
                "sector_aligned_hashes": {"md5": p1_s_md5, "sha256": p1_s_sha256},
                "header_hex_64": p1_payload[:64].hex(" "),
            }
            self.partitions.append(part1)

        # Partição 2: Offset 0x04000000 (64 MB, LBA 131072)
        if 0x04000000 in positions:
            p2_offset = 0x04000000
            p2_size = 0xB0000  # 720,896 bytes (1,408 setores)
            p2_pre_pad_size = 0xAFFD0  # 720,848 bytes úteis antes de 48 bytes 0xFF

            p2_payload = data[p2_offset : p2_offset + p2_size]
            p2_md5, p2_sha256 = get_hashes(p2_payload)
            p2_ent = calculate_entropy(p2_payload)

            part2 = {
                "partition_id": 2,
                "name": "Sub-sistema / Microcodigo & Fontes OSD",
                "offset": p2_offset,
                "offset_hex": f"0x{p2_offset:08X}",
                "lba_start": p2_offset // SECTOR_SIZE,
                "exact_payload_size": p2_size,
                "pre_padding_size": p2_pre_pad_size,
                "sectors_count": p2_size // SECTOR_SIZE,
                "signature_hex": PANASONIC_PARTITION_SIGNATURE.hex(" "),
                "entropy": round(p2_ent, 4),
                "exact_payload_hashes": {"md5": p2_md5, "sha256": p2_sha256},
                "header_hex_64": p2_payload[:64].hex(" "),
            }
            self.partitions.append(part2)

        self.report["partitions"] = self.partitions

        print("\n[+] PARTICÕES DE FIRMWARE PROPRIETÁRIAS (Assinatura: 51 87 13 8D 5B 0A FB 22)")
        for p in self.partitions:
            print(f"\n  ------------------------------------------------------------------------")
            print(f"  PARTICAO {p['partition_id']}: {p['name']}")
            print(f"  ------------------------------------------------------------------------")
            print(f"  Offset Inicial    : {p['offset_hex']} (LBA {p['lba_start']:,})")
            print(f"  Tamanho Util      : {p['exact_payload_size']:,} bytes ({p['exact_payload_size']/(1024*1024):.3f} MB / 0x{p['exact_payload_size']:X})")
            print(f"  Setores Ocupados  : {p['sectors_count']:,} LBAs ({p['sectors_count']*SECTOR_SIZE:,} bytes)")
            print(f"  Assinatura Magic  : {p['signature_hex']}")
            print(f"  Entropia Shannon  : {p['entropy']:.4f} bits/byte (Alta densidade / Criptografia / Compressao)")
            print(f"  MD5 Payload Util  : {p['exact_payload_hashes']['md5']}")
            print(f"  SHA-256 Util      : {p['exact_payload_hashes']['sha256']}")
            if "sector_aligned_hashes" in p:
                print(f"  SHA-256 Alinhado  : {p['sector_aligned_hashes']['sha256']}")

    def extract_partitions(self, output_dir: str):
        """Exporta partições e metadados para arquivos .bin independentes."""
        os.makedirs(output_dir, exist_ok=True)
        manifest_entries = []

        print(f"\n[+] EXTRAINDO PARTICÕES E METADADOS PARA: '{output_dir}'")

        with open(self.image_path, "rb") as f:
            # 1. Master MBR (LBA 0)
            f.seek(0)
            mbr_data = f.read(SECTOR_SIZE)
            mbr_file = os.path.join(output_dir, "hdd_mbr_master_lba0.bin")
            with open(mbr_file, "wb") as out:
                out.write(mbr_data)
            manifest_entries.append((mbr_file, get_hashes(mbr_data)[1]))
            print(f"  [+] Extraido MBR Master : {mbr_file} (512 bytes)")

            # 2. Mirror MBR (LBA 57344)
            f.seek(57344 * SECTOR_SIZE)
            mirror_data = f.read(SECTOR_SIZE)
            mirror_file = os.path.join(output_dir, "hdd_mbr_mirror_lba57344.bin")
            with open(mirror_file, "wb") as out:
                out.write(mirror_data)
            manifest_entries.append((mirror_file, get_hashes(mirror_data)[1]))
            print(f"  [+] Extraido MBR Mirror : {mirror_file} (512 bytes)")

            # 3. Allocation Descriptors (0x00003000)
            f.seek(0x00003000)
            alloc_data = f.read(4096)
            alloc_file = os.path.join(output_dir, "hdd_allocation_descriptors_0x00003000.bin")
            with open(alloc_file, "wb") as out:
                out.write(alloc_data)
            manifest_entries.append((alloc_file, get_hashes(alloc_data)[1]))
            print(f"  [+] Extraido Descritores: {alloc_file} (4,096 bytes)")

            # 4. Journal Area (0x00820000 - 0x00960000)
            f.seek(0x00820000)
            j_data = f.read(0x00960000 - 0x00820000)
            j_file = os.path.join(output_dir, "hdd_journal_blocks_0x00820000.bin")
            with open(j_file, "wb") as out:
                out.write(j_data)
            manifest_entries.append((j_file, get_hashes(j_data)[1]))
            print(f"  [+] Extraido Journal    : {j_file} (1,310,720 bytes)")

            # 5. Firmware Partition 1 (Offset 0x02000000 - Exact Payload)
            f.seek(0x02000000)
            p1_data = f.read(0x5E3250)
            p1_file = os.path.join(output_dir, "hdd_partition_firmware_0x02000000.bin")
            with open(p1_file, "wb") as out:
                out.write(p1_data)
            manifest_entries.append((p1_file, get_hashes(p1_data)[1]))
            print(f"  [+] Extraida Particao 1 : {p1_file} (6,173,264 bytes / 0x5E3250)")

            # 6. Firmware Partition 1 (Sector Padded - 0x5E3400)
            f.seek(0x02000000)
            p1_s_data = f.read(0x5E3400)
            p1_s_file = os.path.join(output_dir, "hdd_partition_firmware_0x02000000_sector_aligned.bin")
            with open(p1_s_file, "wb") as out:
                out.write(p1_s_data)
            manifest_entries.append((p1_s_file, get_hashes(p1_s_data)[1]))
            print(f"  [+] Extraida P1 Alinhada: {p1_s_file} (6,173,696 bytes / 0x5E3400)")

            # 7. Firmware Partition 2 (Offset 0x04000000 - 0xB0000)
            f.seek(0x04000000)
            p2_data = f.read(0xB0000)
            p2_file = os.path.join(output_dir, "hdd_partition_firmware_0x04000000.bin")
            with open(p2_file, "wb") as out:
                out.write(p2_data)
            manifest_entries.append((p2_file, get_hashes(p2_data)[1]))
            print(f"  [+] Extraida Particao 2 : {p2_file} (720,896 bytes / 0xB0000)")

        # Salva manifesto SHA256SUMS.txt
        manifest_path = os.path.join(output_dir, "SHA256SUMS.txt")
        with open(manifest_path, "w", encoding="utf-8") as m:
            for filepath, sha in manifest_entries:
                basename = os.path.basename(filepath)
                m.write(f"{sha}  {basename}\n")
        print(f"  [+] Manifesto Gerado    : {manifest_path}")

    def export_json(self, json_path: str):
        """Exporta relatório completo em JSON."""
        with open(json_path, "w", encoding="utf-8") as j:
            json.dump(self.report, j, indent=2)
        print(f"\n[+] Relatorio JSON exportado com sucesso: '{json_path}'")


def main():
    parser = argparse.ArgumentParser(
        description="Panasonic DMR-EH55 HDD Image Partition & MEIHDFS Storage Parser",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "image",
        nargs="?",
        default=DEFAULT_IMAGE_PATH,
        help=f"Caminho para a imagem de disco (Padrao: {DEFAULT_IMAGE_PATH})",
    )
    parser.add_argument(
        "--extract",
        action="store_true",
        help="Extrai as particoes e metadados para arquivos .bin separados",
    )
    parser.add_argument(
        "--outdir",
        default="extracted_partitions",
        help="Diretorio de saida para extracao (Padrao: extracted_partitions)",
    )
    parser.add_argument(
        "--json",
        metavar="FILE",
        help="Exporta o relatorio tecnico estruturado para arquivo JSON",
    )
    parser.add_argument(
        "--verbose",
        "-v",
        action="store_true",
        help="Habilita exibicao de logs tecnicos detalhados",
    )

    args = parser.parse_args()

    try:
        hdd_parser = PanasonicHDDParser(args.image, verbose=args.verbose)
        hdd_parser.load_and_verify()
        hdd_parser.parse_mbr_tables()
        hdd_parser.parse_metadata_and_allocation()
        hdd_parser.parse_firmware_partitions()

        if args.extract:
            hdd_parser.extract_partitions(args.outdir)

        if args.json:
            hdd_parser.export_json(args.json)

        print("\n" + "=" * 80)
        print(" ANALISE DE ENGENHARIA REVERSA CONCLUIDA COM SUCESSO")
        print("=" * 80)

    except Exception as e:
        print(f"\n[ERRO CRITICO] Falha durante o processamento: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()
