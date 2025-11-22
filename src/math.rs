#![allow(dead_code)]

use soroban_sdk::Env;

// =============================================================
// MATH CLMM (v0) – Tick <-> sqrt_price_x64
// =============================================================
//
// Definisi (ikut gaya Uniswap V3):
//   price        = 1.0001^tick        (token1 per 1 token0)
//   sqrt_price   = 1.0001^(tick / 2)
//   sqrt_price_x64 = floor( sqrt_price * 2^64 )   (Q64.64)
//
// Di v0 ini kita:
//  - Implement tick_to_sqrt_price_x64 (beneran matematis)
//  - Implement sqrt_price_to_tick (approx, buat debug / view)
//
// Fungsi lain (liquidity / swap) masih dummy dulu biar kontrak
// tetap compile dan jalan.
//
// =============================================================

// 2^64 dalam u128 (basis Q64.64)
const ONE_X64: u128 = 1u128 << 64;

// sqrt(1.0001) dalam Q64.64
// sqrt(1.0001) ≈ 1.0000499987500624
// SQRT_1_0001_X64 = floor( sqrt(1.0001) * 2^64 )
const SQRT_1_0001_X64: u128 = 18_447_666_387_855_958_016u128;

// -------------------------------------------------------------
// Helper Q64.64
// -------------------------------------------------------------

#[inline]
fn mul_q64(a: u128, b: u128) -> u128 {
    // (a * b) / 2^64
    //
    // NOTE: untuk v0 kita pakai saturating_mul biar ga panic kalau overflow.
    // Selama tick di sekitar 0 (range kecil) ini aman.
    a.saturating_mul(b) / ONE_X64
}

#[inline]
fn div_q64(a: u128, b: u128) -> u128 {
    // (a * 2^64) / b
    a.saturating_mul(ONE_X64) / b
}

// -------------------------------------------------------------
// Convert tick → sqrt_price_x64 (Q64.64)
// -------------------------------------------------------------
//
// Definisi:
//   price      = 1.0001^tick
//   sqrt_price = 1.0001^(tick/2)
//   sqrt_x64   = floor( sqrt_price * 2^64 )
//
// Implementasi v0:
//   - Mulai dari sqrt_price = 1.0 (ONE_X64)
//   - Hitung faktor = (sqrt(1.0001))^|tick|
//   - Kalau tick > 0 → sqrt = 1.0 * faktor
//   - Kalau tick < 0 → sqrt = 1.0 / faktor
//
// Kompleksitas: O(|tick|) → cukup buat tick dekat 0 / testnet.
//
pub fn tick_to_sqrt_price_x64(_env: &Env, tick: i32) -> u128 {
    if tick == 0 {
        return ONE_X64;
    }

    let abs_tick = if tick < 0 { -tick } else { tick } as u32;

    // hitung faktor = (sqrt(1.0001))^abs_tick
    let mut factor: u128 = ONE_X64;
    for _ in 0..abs_tick {
        factor = mul_q64(factor, SQRT_1_0001_X64);
    }

    if tick > 0 {
        // price > 1
        factor
    } else {
        // price < 1  → ambil kebalikan
        div_q64(ONE_X64, factor)
    }
}

// -------------------------------------------------------------
// Convert sqrt_price_x64 → tick (versi naive / debug)
// -------------------------------------------------------------
//
// Cara kerja (approx):
//  - Kalau sqrt_price_x64 > 2^64 (price > 1):
//      mulai dari sqrt = 1.0, kalikan sqrt(1.0001) sampai >= target.
//  - Kalau sqrt_price_x64 < 2^64 (price < 1):
//      mulai dari sqrt = 1.0, bagi sqrt(1.0001) sampai <= target.
//
// max_abs_tick = batas iterasi biar ga infinite loop.
//
// ⚠️ Jangan pakai ini buat tick ratusan ribu, ini cuma buat:
//  - view / debug
//  - sync current_tick sesekali kalau perlu.
//
pub fn sqrt_price_to_tick(_env: &Env, sqrt_price_x64: u128) -> i32 {
    if sqrt_price_x64 == ONE_X64 {
        return 0;
    }

    let max_abs_tick: i32 = 10_000; // batas aman v0
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

// =============================================================
// SISA FUNGSI: masih dummy dulu, supaya kontrak tetap compile
// =============================================================

// Hitung amount0 & amount1 untuk liquidity di range
pub fn get_amounts_for_liquidity(
    _env: &Env,
    liquidity: i128,
    _sqrt_price_lower: u128,
    _sqrt_price_upper: u128,
    _current_sqrt_price: u128,
) -> (i128, i128) {
    let _ = liquidity;
    // TODO: nanti diisi rumus Uniswap V3:
    // amount0 = L * (sqrtU - sqrtL) / (sqrtU * sqrtL)
    // amount1 = L * (sqrtU - sqrtL)
    (0, 0)
}

// Hitung liquidity berdasarkan amount0
pub fn get_liquidity_for_amount0(
    _env: &Env,
    amount0: i128,
    _sqrt_price_lower: u128,
    _sqrt_price_upper: u128,
) -> i128 {
    let _ = amount0;
    // TODO: L = amount0 * (sqrtU * sqrtL) / (sqrtU - sqrtL)
    0
}

// Hitung liquidity berdasarkan amount1
pub fn get_liquidity_for_amount1(
    _env: &Env,
    amount1: i128,
    _sqrt_price_lower: u128,
    _sqrt_price_upper: u128,
) -> i128 {
    let _ = amount1;
    // TODO: L = amount1 / (sqrtU - sqrtL)
    0
}

// Core SWAP math (dummy)
//
// Nanti fungsi ini akan:
// - menentukan next sqrt price
// - menentukan amount_in/out
// - menentukan apakah boundary tercapai
//
pub fn compute_swap_step(
    _env: &Env,
    sqrt_price_current: u128,
    liquidity: i128,
    sqrt_price_target: u128,
    _zero_for_one: bool,
) -> (u128, i128, i128, bool) {
    let _ = (liquidity, sqrt_price_target);

    // dummy: harga tidak bergerak, amount = 1
    let next_price = sqrt_price_current;
    let amount_in = 1;
    let amount_out = 1;
    let reached_boundary = true;

    (next_price, amount_in, amount_out, reached_boundary)
}

// Math utility: safe add/sub (v0 simple)
pub fn add_delta(a: i128, b: i128) -> i128 {
    a + b
}

pub fn sub_delta(a: i128, b: i128) -> i128 {
    a - b
}



