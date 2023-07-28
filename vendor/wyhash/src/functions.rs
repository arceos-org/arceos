const P0: u64 = 0xa076_1d64_78bd_642f;
const P1: u64 = 0xe703_7ed1_a0b4_28db;
const P2: u64 = 0x8ebc_6af0_9c88_c6e3;
const P3: u64 = 0x5899_65cc_7537_4cc3;
const P4: u64 = 0x1d8e_4e27_c47d_124f;
const P5: u64 = 0xeb44_acca_b455_d165;

#[inline]
fn wymum(a: u64, b: u64) -> u64 {
    let r = u128::from(a) * u128::from(b);
    ((r >> 64) ^ r) as u64
}

#[inline]
pub fn read64(data: &[u8]) -> u64 {
    u64::from(data[7]) << 56
        | u64::from(data[6]) << 48
        | u64::from(data[5]) << 40
        | u64::from(data[4]) << 32
        | u64::from(data[3]) << 24
        | u64::from(data[2]) << 16
        | u64::from(data[1]) << 8
        | u64::from(data[0])
}

#[inline]
fn read32(data: &[u8]) -> u64 {
    u64::from(data[3]) << 24
        | u64::from(data[2]) << 16
        | u64::from(data[1]) << 8
        | u64::from(data[0])
}

#[inline]
fn read64_swapped(data: &[u8]) -> u64 {
    (read32(data) << 32) | read32(&data[4..])
}

#[inline]
fn read_rest(data: &[u8]) -> u64 {
    // This may be mathematically acceptable but the hashes would change as the byte sorting changes.
    // let mut result = 0;
    // for i in 0..data.len() {
    //     result |= u64::from(data[i]) << ((data.len() - i - 1) * 8);
    // }
    // result

    match data.len() {
        1 => u64::from(data[0]),
        2 => u64::from(data[1]) << 8 | u64::from(data[0]),
        3 => u64::from(data[1]) << 16 | u64::from(data[0]) << 8 | u64::from(data[2]),
        4 => read32(data),
        5 => read32(data) << 8 | u64::from(data[4]),
        6 => read32(data) << 16 | u64::from(data[5]) << 8 | u64::from(data[4]),
        7 => {
            read32(data) << 24
                | u64::from(data[5]) << 16
                | u64::from(data[4]) << 8
                | u64::from(data[6])
        }
        8 => read64_swapped(data),
        _ => panic!(),
    }
}

/// Generate a hash for the input data and seed
pub fn wyhash(bytes: &[u8], seed: u64) -> u64 {
    let seed = wyhash_core(bytes, seed);
    wyhash_finish(bytes.len() as u64, seed)
}

#[inline]
pub(crate) fn mix_with_p0(seed: u64) -> u64 {
    seed ^ P0
}

#[inline]
pub(crate) fn wyhash_core(bytes: &[u8], mut seed: u64) -> u64 {
    for chunk in bytes.chunks_exact(32) {
        seed = wymum(
            seed ^ P0,
            wymum(read64(chunk) ^ P1, read64(&chunk[8..]) ^ P2)
                ^ wymum(read64(&chunk[16..]) ^ P3, read64(&chunk[24..]) ^ P4),
        );
    }
    seed = mix_with_p0(seed);

    let rest = bytes.len() & 31;
    if rest != 0 {
        let start = bytes.len() & !31;
        match ((bytes.len() - 1) & 31) / 8 {
            0 => seed = wymum(seed, read_rest(&bytes[start..]) ^ P1),
            1 => {
                seed = wymum(
                    read64_swapped(&bytes[start..]) ^ seed,
                    read_rest(&bytes[start + 8..]) ^ P2,
                )
            }
            2 => {
                seed = wymum(
                    read64_swapped(&bytes[start..]) ^ seed,
                    read64_swapped(&bytes[start + 8..]) ^ P2,
                ) ^ wymum(seed, read_rest(&bytes[start + 16..]) ^ P3)
            }

            3 => {
                seed = wymum(
                    read64_swapped(&bytes[start..]) ^ seed,
                    read64_swapped(&bytes[start + 8..]) ^ P2,
                ) ^ wymum(
                    read64_swapped(&bytes[start + 16..]) ^ seed,
                    read_rest(&bytes[start + 24..]) ^ P4,
                )
            }
            _ => unreachable!(),
        }
    }
    seed
}

#[inline]
pub(crate) fn wyhash_finish(length: u64, seed: u64) -> u64 {
    wymum(seed, length ^ P5)
}

/// Pseudo-Random Number Generator (PRNG)
///
/// Note that the input seed is updated
pub fn wyrng(seed: &mut u64) -> u64 {
    *seed = seed.wrapping_add(P0);
    wymum(*seed ^ P1, *seed)
}

#[cfg(test)]
mod tests {
    use super::{read64, read_rest};

    #[test]
    fn read_rest_byte_sorting() {
        assert_eq!(0x01, read_rest(&[1]));
        assert_eq!(0x0201, read_rest(&[1, 2]));
        assert_eq!(0x02_0103, read_rest(&[1, 2, 3]));
        assert_eq!(0x0403_0201, read_rest(&[1, 2, 3, 4]));
        assert_eq!(0x04_0302_0105, read_rest(&[1, 2, 3, 4, 5]));
        assert_eq!(0x0403_0201_0605, read_rest(&[1, 2, 3, 4, 5, 6]));
        assert_eq!(0x04_0302_0106_0507, read_rest(&[1, 2, 3, 4, 5, 6, 7]));
        assert_eq!(0x0403_0201_0807_0605, read_rest(&[1, 2, 3, 4, 5, 6, 7, 8]));
    }

    #[test]
    fn read64_byte_sorting() {
        assert_eq!(0x0807_0605_0403_0201, read64(&[1, 2, 3, 4, 5, 6, 7, 8]));
    }
}
