//! Matsushita MN103S (`BOOT.BIN` `FUN_4400a666`) 64-bit Feistel Block Cipher & Strided Adler-32
//!
//! Pure-Rust implementation reconstructed directly from `BOOT.BIN` (`FUN_44011d42` key schedule,
//! `FUN_44011ec0` block encrypt, `FUN_44012029` block decrypt, `FUN_44012191` F-function, and
//! `FUN_4401e5a3` strided Adler-32 checksum).

/// Reconstructs the 7-byte Feistel master key from split `.data` fragments (`0x50480278` / `0x5048027F`).
pub fn reconstruct_master_key_7b(p1: &[u8; 7], p2: &[u8; 8]) -> [u8; 7] {
    let mut master = [0u8; 7];
    for (idx, dst) in master.iter_mut().enumerate() {
        *dst = p2[idx] ^ p1[idx];
    }
    master[1] ^= p2[7];
    master[4] ^= p2[7];
    master
}

/// Evaluates the 32-bit Matsushita Feistel F-function (`FUN_44012191`).
#[allow(clippy::too_many_arguments)]
pub fn feistel_f_round(
    b0: u8,
    b1: u8,
    b2: u8,
    b3: u8,
    k0: u8,
    k1: u8,
    k2: u8,
    k3: u8,
    sbox: &[u8; 256],
) -> [u8; 4] {
    let v3 = k1 ^ b1;
    let v4 = k0 ^ b0;
    let x3 = k3 ^ b3;
    let x2 = k2 ^ b2;

    let v1 = x3 >> 2;
    let v5 = x3.rotate_right(2);
    let v2 = x2 >> 5;
    let v6 = x2.rotate_right(5);

    let o3 = (v6 >> 1) | (v1 << 7);
    let o2 = (v3 & 0x7F) | (v2 << 7);
    let o1 = (v4 >> 1) | (v3 & 0x80);
    let o0 = (v5 >> 1) | (v4 << 7);

    let s = sbox[(v4 ^ v3.rotate_left(1) ^ v6 ^ v5) as usize];
    [
        o0.wrapping_add(s.rotate_left(3)),
        o1.wrapping_add(s.rotate_left(2)),
        o2.wrapping_add(s.rotate_left(1)),
        o3.wrapping_add(s),
    ]
}

/// Expands the 7-byte master key and 4-byte IV into the 32-byte round-key table (`FUN_44011d42`).
pub fn derive_round_key_32b(key_7b: &[u8; 7], iv: &[u8; 4], sbox: &[u8; 256]) -> [u8; 32] {
    let mut rk = [0u8; 32];
    rk[..7].copy_from_slice(key_7b);
    let mut state = *iv;

    for round in 0..16usize {
        let off = (round & 7) * 4;
        let next = feistel_f_round(
            rk[off],
            rk[off + 1],
            rk[off + 2],
            rk[off + 3],
            state[0],
            state[1],
            state[2],
            state[3],
            sbox,
        );
        rk[off..off + 4].copy_from_slice(&next);
        state = next;
    }

    rk
}

fn run_feistel_rounds(
    block: &[u8; 8],
    round_key: &[u8; 32],
    sbox: &[u8; 256],
    decrypt: bool,
) -> [u8; 8] {
    let mut l = [block[0], block[1], block[2], block[3]];
    let mut r = [block[4], block[5], block[6], block[7]];

    for step in 0..16usize {
        let round = if decrypt { 15 - step } else { step };
        let k_off = (round & 7) * 4;
        let f_out = feistel_f_round(
            l[0],
            l[1],
            l[2],
            l[3],
            round_key[k_off],
            round_key[k_off + 1],
            round_key[k_off + 2],
            round_key[k_off + 3],
            sbox,
        );
        let prev_l = l;
        for idx in 0..4 {
            l[idx] = f_out[idx] ^ r[idx];
        }
        r = prev_l;
    }

    [r[0], r[1], r[2], r[3], l[0], l[1], l[2], l[3]]
}

/// Decrypts a single 64-bit ECB block (`FUN_44012029`).
pub fn feistel_decrypt_block(block: &[u8; 8], round_key: &[u8; 32], sbox: &[u8; 256]) -> [u8; 8] {
    run_feistel_rounds(block, round_key, sbox, true)
}

/// Encrypts a single 64-bit ECB block (`FUN_44011ec0`).
pub fn feistel_encrypt_block(block: &[u8; 8], round_key: &[u8; 32], sbox: &[u8; 256]) -> [u8; 8] {
    run_feistel_rounds(block, round_key, sbox, false)
}

fn transform_buffer_in_place(
    data: &mut [u8],
    round_key: &[u8; 32],
    sbox: &[u8; 256],
    decrypt: bool,
) {
    for chunk in data.chunks_exact_mut(8) {
        let mut blk = [0u8; 8];
        blk.copy_from_slice(chunk);
        let out = run_feistel_rounds(&blk, round_key, sbox, decrypt);
        chunk.copy_from_slice(&out);
    }
}

/// Decrypts an aligned buffer in-place in 8-byte blocks (`FUN_4400a666`).
pub fn feistel_decrypt_buffer(data: &mut [u8], round_key: &[u8; 32], sbox: &[u8; 256]) {
    transform_buffer_in_place(data, round_key, sbox, true);
}

/// Encrypts an aligned buffer in-place in 8-byte blocks.
pub fn feistel_encrypt_buffer(data: &mut [u8], round_key: &[u8; 32], sbox: &[u8; 256]) {
    transform_buffer_in_place(data, round_key, sbox, false);
}

/// Computes Panasonic's strided Adler-32 checksum (`FUN_4401e5a3`, `s1 | (s2 << 16)`).
pub fn adler32_panasonic_strided(data: &[u8], step: usize) -> u32 {
    const MOD_ADLER: u32 = 65_521;
    let stride = step.max(1);
    let mut s1: u32 = 1;
    let mut s2: u32 = 0;

    for &byte in data.iter().step_by(stride) {
        s1 = (s1 + u32::from(byte)) % MOD_ADLER;
        s2 = (s2 + s1) % MOD_ADLER;
    }

    s1 | (s2 << 16)
}
