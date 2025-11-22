#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Env};

/// State utama pool (versi bayi)
#[derive(Clone)]
#[contracttype]
pub struct PoolState {
    /// Harga dalam bentuk sqrt(P) fixed-point (sementara diisi manual aja dulu)
    pub sqrt_price_x64: i128,
    /// Tick aktif sekarang
    pub current_tick: i32,
    /// Total liquidity global (sementara cuma angka doang)
    pub liquidity: i128,
}

/// Key untuk storage
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    PoolState,
}

/// Helper: simpan state pool ke storage
fn write_pool_state(env: &Env, state: &PoolState) {
    env.storage().persistent().set(&DataKey::PoolState, state);
}

/// Helper: baca state pool dari storage
fn read_pool_state(env: &Env) -> PoolState {
    env.storage()
        .persistent()
        .get::<_, PoolState>(&DataKey::PoolState)
        // Untuk v0 kita anggap sudah di-initialize, kalau belum ya panic aja
        .expect("pool not initialized")
}

#[contract]
pub struct ClmmPool;

#[contractimpl]
impl ClmmPool {
    /// Inisialisasi pool pertama kali
    pub fn initialize(env: Env, sqrt_price_x64: i128, current_tick: i32) {
        let state = PoolState {
            sqrt_price_x64,
            current_tick,
            liquidity: 0,
        };
        write_pool_state(&env, &state);
    }

    /// Baca state pool buat UI / debugging
    pub fn get_pool_state(env: Env) -> PoolState {
        read_pool_state(&env)
    }

    /// Tambah liquidity (VERSI BAYI — belum ada tick range, belum token transfer)
    pub fn add_liquidity(env: Env, amount: i128) {
        let mut ps = read_pool_state(&env);
        ps.liquidity += amount;
        write_pool_state(&env, &ps);
    }
}

