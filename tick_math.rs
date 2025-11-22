// src/tick_math.rs
//
// Tick <-> sqrt_price_x64 mapping (v0)
// ====================================
//
// Definisi gaya Uniswap V3:
//
//   price        = 1.0001^tick        (token1 per 1 token0)
//   sqrt_price   = 1.0001^(tick / 2)
//   sqrt_price_x64 = floor( sqrt_price * 2^64 )   (Q64.64)
//
// Implementasi v0:
//
//  - tick_to_sqrt_price_x64(tick):
//       * hitung faktor = (sqrt(1.0001))^|tick| di Q64.64
//       * kalau tick > 0 → sqrt = 1 * faktor
//       * kalau tick < 0 → sqrt = 1 / faktor
//
//  - sqrt_price_x64_to_tick(sqrt):
//       * approx: dari sqrt=1, kali / bagi sqrt(1.0001)
//         sampai mendekati target (loop sampai max_abs_tick)
//
// NB: Ini matematis bener, tapi O(|tick|).
//     Cocok buat testnet / tick range kecil. Nanti kalau
//     kamu butuh tick besar, kita upgrade ke versi cepat
//     (exponentiation by squaring & tabel konstanta).
//

#![allow(dead_code)]

use core::cmp;

/// 2^64 dalam u128 (basis Q64.64)
pub const ONE_X64: u128 = 1u128 << 64;

/// sqrt(1.0001) dalam Q64.64
/// sqrt(1.0001) ≈ 1.0000499987500624
/// SQRT_1_0001_X64 = floor( sqrt(1.0001) * 2^64 )
pub const SQRT_1_0001_X64: u128 = 18_447_666_387_855_958_016u128;

// ------------------------------------------------------------
// Helper Q64.64
// ------------------------------------------------------------

#[inline]
pub fn mul_q64(a: u128, b: u128) -> u128 {
    // (a * b) / 2^64
    a.saturating_mul(b) / ONE_X64
}

#[inline]
pub fn div_q64(a: u128, b: u128) -> u128 {
    // (a * 2^64) / b
    a.saturating_mul(ONE_X64) / b
}

// ------------------------------------------------------------
// tick -> sqrt_price_x64
// ------------------------------------------------------------

/// tick -> sqrt_price_x64 (Q64.64)
///
/// price      = 1.0001^tick
/// sqrt_price = 1.0001^(tick/2)
/// sqrt_x64   = floor( sqrt_price * 2^64 )
///
/// v0: O(|tick|) loop, cukup untuk tick di sekitar 0.
pub fn tick_to_sqrt_price_x64(tick: i32) -> u128 {
    if tick == 0 {
        return ONE_X64;
    }

    let abs_tick = if tick < 0 { -tick } else { tick } as u32;

    // faktor = (sqrt(1.0001))^abs_tick
    let mut factor: u128 = ONE_X64;
    for _ in 0..abs_tick {
        factor = mul_q64(factor, SQRT_1_0001_X64);
    }

    if tick > 0 {
        // tick positif → price > 1
        factor
    } else {
        // tick negatif → price < 1 → ambil kebalikan
        div_q64(ONE_X64, factor)
    }
}

// ------------------------------------------------------------
// sqrt_price_x64 -> tick (approx / debug)
// ------------------------------------------------------------

/// sqrt_price_x64 -> tick (approx)
///
/// - Kalau sqrt > 1 → tick positif
/// - Kalau sqrt < 1 → tick negatif
///
/// max_abs_tick: batas iterasi,
///   misal 10_000 artinya kita asumsikan tick di range [-10k, +10k].
pub fn sqrt_price_x64_to_tick(sqrt_price_x64: u128, max_abs_tick: i32) -> i32 {
    if sqrt_price_x64 == ONE_X64 {
        return 0;
    }

    let mut current: u128 = ONE_X64;
    let mut tick: i32 = 0;

    if sqrt_price_x64 > ONE_X64 {
        // price > 1 → tick positif
        while tick < max_abs_tick {
            current = mul_q64(current, SQRT_1_0001_X64);
            tick += 1;

            if current >= sqrt_price_x64 {
                return tick;
            }
        }
        max_abs_tick
    } else {
        // price < 1 → tick negatif
        while tick > -max_abs_tick {
            current = div_q64(current, SQRT_1_0001_X64);
            tick -= 1;

            if current <= sqrt_price_x64 {
                return tick;
            }
        }
        -max_abs_tick
    }
}

// ------------------------------------------------------------
// Helper: sqrt_price_x64 -> price_x64 (optional)
// ------------------------------------------------------------

/// Price (Q64.64) dari sqrt_price_x64:
/// price = sqrt_price^2
pub fn sqrt_price_x64_to_price_x64(sqrt_price_x64: u128) -> u128 {
    mul_q64(sqrt_price_x64, sqrt_price_x64)
}

