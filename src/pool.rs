use soroban_sdk::{Env, Symbol, contracttype, Address};

use crate::DataKey;

//
// PoolState = state utama CLMM
//
#[contracttype]
#[derive(Clone, Debug)]
pub struct PoolState {
    pub sqrt_price_x64: u128,
    pub current_tick: i32,
    pub liquidity: i128,
    pub tick_spacing: i32,
    pub token0: Address,
    pub token1: Address,
}

// ------------------------------------------------------------
// STORAGE: pakai persistent + DataKey::PoolState
// ------------------------------------------------------------
pub fn get_pool_state(env: &Env) -> PoolState {
    env.storage()
        .persistent()
        .get::<_, PoolState>(&DataKey::PoolState)
        .expect("pool not initialized")
}

pub fn set_pool_state(env: &Env, state: &PoolState) {
    env.storage()
        .persistent()
        .set::<_, PoolState>(&DataKey::PoolState, state);
}

// ------------------------------------------------------------
// INITIALIZE POOL (dipanggil sekali dari lib.rs::initialize)
// ------------------------------------------------------------
//
// NOTE:
// - auth admin sudah di-handle di lib.rs (owner.require_auth())
// - di sini fokus set state awal + event
//
pub fn init_pool(
    env: &Env,
    sqrt_price_x64: u128,
    initial_tick: i32,
    tick_spacing: i32,
    token0: Address,
    token1: Address,
) {
    if tick_spacing <= 0 {
        panic!("tick_spacing must be > 0");
    }

    let state = PoolState {
        sqrt_price_x64,
        current_tick: initial_tick,
        liquidity: 0,
        tick_spacing,
        token0,
        token1,
    };

    set_pool_state(env, &state);

    // EVENT
    env.events().publish(
        (Symbol::new(env, "init_pool"),),
        (sqrt_price_x64, initial_tick, tick_spacing),
    );
}

