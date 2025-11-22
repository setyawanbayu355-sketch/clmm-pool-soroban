use soroban_sdk::{contracttype, Address, Env, Symbol};

use crate::pool::{get_pool_state, set_pool_state, PoolState};
use crate::tick::{find_next_initialized_tick, read_tick_info, write_tick_info};

#[contracttype]
#[derive(Clone, Debug)]
pub struct SwapResult {
    pub amount_in: i128,
    pub amount_out: i128,
    pub sqrt_price_x64: u128,
    pub current_tick: i32,
}
pub fn apply_simple_price_move(
    pool: &mut PoolState,
    amount_in: i128,
    zero_for_one: bool,
) {
    // Representasi 1.0 di Q64
    let one_x64: u128 = 1u128 << 64;

    // liquidity selalu > 0 di level call (sudah dicek di lib.rs), tapi kita tetap safe
    let liq_u: u128 = if pool.liquidity < 0 {
        (-pool.liquidity) as u128
    } else {
        pool.liquidity as u128
    };

    if liq_u == 0 {
        // Tidak ada liquidity, tidak usah gerakin harga
        return;
    }

    let abs_in: u128 = if amount_in < 0 {
        (-amount_in) as u128
    } else {
        amount_in as u128
    };

    // seberapa besar trade relatif ke liquidity (Q64.64)
    let trade_ratio_x64: u128 = abs_in
        .saturating_mul(one_x64)
        / liq_u;

    // Clamp maksimal 10% (0.1 * ONE_X64)
    let max_ratio_x64: u128 = one_x64 / 10;
    let r: u128 = if trade_ratio_x64 > max_ratio_x64 {
        max_ratio_x64
    } else {
        trade_ratio_x64
    };

    // delta_sqrt ~ perubahan sqrt_price_x64
    // kalau trade_ratio = 10%, delta_sqrt ~ 5% dari harga sekarang
    let mut delta_sqrt: u128 = pool
        .sqrt_price_x64
        .saturating_mul(r)
        / (20 * one_x64); // 20 → max ~5%

    if delta_sqrt == 0 {
        delta_sqrt = 1; // minimal 1 biar keliatan gerak
    }

    // Arah:
    // zero_for_one = user jual token0 (XLM) → beli token1 (USDC) → harga token1 naik (sqrt_price naik)
    if zero_for_one {
        pool.sqrt_price_x64 = pool.sqrt_price_x64.saturating_add(delta_sqrt);
    } else {
        pool.sqrt_price_x64 = pool.sqrt_price_x64.saturating_sub(delta_sqrt);
    }

    // Phase ini belum ubah current_tick sama sekali
}
// ENTRY INTERNAL – BUKAN #[contractimpl]
pub fn swap(
    env: Env,
    caller: Address,
    amount_specified: i128,
    zero_for_one: bool,
    sqrt_price_limit_x64: u128,
) -> SwapResult {
    // 1. Load PoolState
    let mut pool = get_pool_state(&env);

    if amount_specified <= 0 {
        panic!("amount must be > 0");
    }

    // 2. Vars lokal
    let mut amount_remaining = amount_specified;
    let mut amount_calculated: i128 = 0;

    let mut sqrt_price: u128 = pool.sqrt_price_x64;
    let mut liquidity: i128 = pool.liquidity;
    let mut current_tick: i32 = pool.current_tick;

    // 3. Dummy loop swap
    loop {
        let next_initialized_tick = find_next_initialized_tick(
            &env,
            current_tick,
            pool.tick_spacing,
            zero_for_one,
        );

        let (next_sqrt_price, step_in, step_out, reached_boundary) =
            compute_swap_step_dummy(
                sqrt_price,
                liquidity,
                next_initialized_tick,
                zero_for_one,
            );

        amount_remaining -= step_in;
        amount_calculated += step_out;

        sqrt_price = next_sqrt_price;

        if reached_boundary {
            cross_tick(&env, next_initialized_tick, &mut pool, zero_for_one);

            // sinkronkan local state dengan pool global
            current_tick = pool.current_tick;
            liquidity = pool.liquidity;
        }

        if amount_remaining <= 0 {
            break;
        }

        if zero_for_one && sqrt_price <= sqrt_price_limit_x64 {
            break;
        }
        if !zero_for_one && sqrt_price >= sqrt_price_limit_x64 {
            break;
        }
    }

    pool.sqrt_price_x64 = sqrt_price;
    pool.current_tick = current_tick;
    pool.liquidity = liquidity;

    set_pool_state(&env, &pool);

    env.events().publish(
        (Symbol::new(&env, "swap"), caller.clone()),
        (amount_specified, amount_calculated, current_tick, sqrt_price),
    );

    SwapResult {
        amount_in: amount_specified - amount_remaining,
        amount_out: amount_calculated,
        sqrt_price_x64: sqrt_price,
        current_tick,
    }
}

// =============================================================
//  DUMMY IMPLEMENTATION (kompile aman v0)
// =============================================================

fn compute_swap_step_dummy(
    sqrt_price: u128,
    _liquidity: i128,
    _next_tick: i32,
    _zero_for_one: bool,
) -> (u128, i128, i128, bool) {
    let next_price = sqrt_price;
    let amount_in: i128 = 1;
    let amount_out: i128 = 1;
    let hit_boundary = true;

    (next_price, amount_in, amount_out, hit_boundary)
}

fn cross_tick(env: &Env, tick: i32, pool: &mut PoolState, zero_for_one: bool) {
    let info = read_tick_info(env, tick);

    if zero_for_one {
        pool.liquidity -= info.liquidity_net;
    } else {
        pool.liquidity += info.liquidity_net;
    }

    pool.current_tick = tick;
    write_tick_info(env, tick, &info);
}



