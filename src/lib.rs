#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Symbol};


mod tick;  // tick.rs (TickInfo + tick logic)
mod pool;  // pool.rs (PoolState + init_pool + helpers)
mod swap;  // swap.rs (swap loop)

pub use pool::*;
pub use tick::TickInfo;
pub use swap::SwapResult;

// =============================================================
//                    POOL CONFIG + POSITION
// =============================================================

#[derive(Clone)]
#[contracttype]
pub struct PoolConfig {
    pub admin: Address,
    pub token_a: Address,
    pub token_b: Address,
    pub fee_bps: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct Position {
    pub liquidity: i128,
    pub token_a_amount: i128,
    pub token_b_amount: i128,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    PoolState,
    PoolConfig,
    Initialized,
    Tick(i32),
    Position(Address, i32, i32),
}

// =============================================================
//                     STORAGE HELPERS
// =============================================================

fn pool_address(env: &Env) -> Address {
    env.current_contract_address()
}

fn write_pool_config(env: &Env, cfg: &PoolConfig) {
    env.storage().persistent().set(&DataKey::PoolConfig, cfg);
}

fn read_pool_config(env: &Env) -> PoolConfig {
    env.storage()
        .persistent()
        .get::<_, PoolConfig>(&DataKey::PoolConfig)
        .expect("pool config not initialized")
}

fn write_position(env: &Env, owner: &Address, lower: i32, upper: i32, pos: &Position) {
    env.storage()
        .persistent()
        .set(&DataKey::Position(owner.clone(), lower, upper), pos);
}

fn read_position(env: &Env, owner: &Address, lower: i32, upper: i32) -> Position {
    env.storage()
        .persistent()
        .get::<_, Position>(&DataKey::Position(owner.clone(), lower, upper))
        .unwrap_or(Position {
            liquidity: 0,
            token_a_amount: 0,
            token_b_amount: 0,
        })
}

// TickInfo helpers
fn read_tick_info_persistent(env: &Env, tick: i32) -> TickInfo {
    env.storage()
        .persistent()
        .get::<_, TickInfo>(&DataKey::Tick(tick))
        .unwrap_or(TickInfo {
            liquidity_gross: 0,
            liquidity_net: 0,
        })
}

fn write_tick_info_persistent(env: &Env, tick: i32, info: &TickInfo) {
    env.storage()
        .persistent()
        .set(&DataKey::Tick(tick), info);
}

// PoolState helpers
fn read_pool_state(env: &Env) -> PoolState {
    env.storage()
        .persistent()
        .get::<_, PoolState>(&DataKey::PoolState)
        .expect("pool not initialized")
}

fn write_pool_state(env: &Env, state: &PoolState) {
    env.storage()
        .persistent()
        .set(&DataKey::PoolState, state);
}

// =============================================================
//                      CONTRACT ENTRYPOINT
// =============================================================

#[contract]
pub struct ClmmPool;

#[contractimpl]
impl ClmmPool {
    // ------------------------------------
    // INITIALIZE
    // ------------------------------------

pub fn initialize(
    env: Env,
    admin: Address,
    token_a: Address,
    token_b: Address,
    fee_bps: u32,
    sqrt_price_x64: u128,
    current_tick: i32,
    tick_spacing: i32,
) {
    // 1️⃣ Cegah double-initialize
    if env.storage().persistent().has(&DataKey::Initialized) {
        panic!("pool already initialized");
    }

    // 2️⃣ Validasi basic (biar ga keisi data aneh)
    if token_a == token_b {
        panic!("token_a and token_b must be different");
    }

    if tick_spacing <= 0 {
        panic!("tick_spacing must be > 0");
    }

    if fee_bps == 0 {
        panic!("fee_bps must be > 0");
    }

    // 3️⃣ Tentukan harga awal
    //    Kalau caller kirim sqrt_price_x64 = 0, kita kasih default 1.0 (2^64)
    let initial_sqrt_price_x64: u128 = if sqrt_price_x64 == 0 {
        1u128 << 64 // Q64.64 untuk price = 1.0
    } else {
        sqrt_price_x64
    };

    // 4️⃣ Inisialisasi PoolState (di pool.rs)
    //
    //    init_pool bertugas:
    //    - set sqrt_price_x64
    //    - set current_tick
    //    - set tick_spacing
    //    - set token_a/token_b
    //    - set liquidity awal = 0
    init_pool(
        &env,
        initial_sqrt_price_x64,
        current_tick,
        tick_spacing,
        token_a.clone(),
        token_b.clone(),
    );

    // 5️⃣ Simpan PoolConfig (admin, fee, token)
    let cfg = PoolConfig {
        admin,
        token_a,
        token_b,
        fee_bps,
    };
    write_pool_config(&env, &cfg);

    // 6️⃣ Flag bahwa pool sudah di-init
    env.storage()
        .persistent()
        .set(&DataKey::Initialized, &true);
}

    // ------------------------------------
    // READERS
    // ------------------------------------

    pub fn get_pool_state(env: Env) -> PoolState {
        read_pool_state(&env)
    }

    pub fn get_pool_config(env: Env) -> PoolConfig {
        read_pool_config(&env)
    }

    pub fn get_tick_info(env: Env, tick: i32) -> TickInfo {
        read_tick_info_persistent(&env, tick)
    }

    pub fn get_position(env: Env, owner: Address, lower: i32, upper: i32) -> Position {
        read_position(&env, &owner, lower, upper)
    }

    // ------------------------------------
    // SWAP ENTRYPOINT
    // ------------------------------------

     // ------------------------------------
    // SWAP ENTRYPOINT (PHASE 2 - SIMPLE PRICE MOVE, NO TICK CROSS)
    // ------------------------------------

    // ------------------------------------
    // SWAP ENTRYPOINT (PHASE 2 - SIMPLE PRICE MOVE, NO TICK CROSS)
    // ------------------------------------

    pub fn swap(
        env: Env,
        caller: Address,
        amount_specified: i128,
        zero_for_one: bool,
        _sqrt_price_limit_x64: u128, // belum dipakai di Phase 2
    ) -> SwapResult {
        caller.require_auth();

        if amount_specified <= 0 {
            panic!("amount_specified must be > 0");
        }

        // Baca config & state pool
        let cfg = read_pool_config(&env);
        let mut pool = read_pool_state(&env);
        let pool_addr = pool_address(&env);

        if pool.liquidity <= 0 {
            panic!("no liquidity in pool");
        }

        // Tentukan token_in / token_out
        // zero_for_one = true  => token_a (XLM) -> token_b (USDC)
        // zero_for_one = false => token_b (USDC) -> token_a (XLM)
        let (token_in, token_out) = if zero_for_one {
            (cfg.token_a.clone(), cfg.token_b.clone())
        } else {
            (cfg.token_b.clone(), cfg.token_a.clone())
        };

        // Hitung fee & amount_out (versi fixed price, 1:1 - fee)
        let amount_in: i128 = amount_specified;
        let fee_bps_i128: i128 = cfg.fee_bps as i128;

        let fee: i128 = amount_in * fee_bps_i128 / 10_000;
        let amount_after_fee: i128 = amount_in - fee;

        if amount_after_fee <= 0 {
            panic!("amount_after_fee must be > 0");
        }

        // Untuk Phase 2: harga masih sederhana → amount_out = amount_after_fee (1:1)
        let amount_out: i128 = amount_after_fee;

        // Transfer token:
        // 1) user kirim token_in ke pool (full amount_in, fee ikut masuk pool)
        token::Client::new(&env, &token_in).transfer(&caller, &pool_addr, &amount_in);

        // 2) pool kirim token_out ke user (amount_out)
        token::Client::new(&env, &token_out).transfer(&pool_addr, &caller, &amount_out);

        // ----------------------------------------------------
        // PRICE MATH DIPINDAH KE swap.rs (apply_simple_price_move)
        // ----------------------------------------------------
        swap::apply_simple_price_move(&mut pool, amount_in, zero_for_one);

        // Simpan state baru
        write_pool_state(&env, &pool);

        // Event swap pakai nilai state terbaru
        env.events().publish(
            (Symbol::new(&env, "swap"), caller.clone()),
            (
                amount_in,
                amount_out,
                pool.current_tick,
                pool.sqrt_price_x64,
            ),
        );

        SwapResult {
            amount_in,
            amount_out,
            current_tick: pool.current_tick,
            sqrt_price_x64: pool.sqrt_price_x64,
        }
    }

    // ------------------------------------
    // ADD LIQUIDITY
    // ------------------------------------

    pub fn add_liquidity(
        env: Env,
        owner: Address,
        lower: i32,
        upper: i32,
        liquidity: i128,
        amt_a: i128,
        amt_b: i128,
    ) {
        owner.require_auth();
        if lower >= upper {
            panic!("tick_lower must < tick_upper");
        }
        if liquidity <= 0 {
            panic!("liquidity must > 0");
        }

        let cfg = read_pool_config(&env);
        let pool_addr = pool_address(&env);

        // transfer tokens
        token::Client::new(&env, &cfg.token_a).transfer(&owner, &pool_addr, &amt_a);
        token::Client::new(&env, &cfg.token_b).transfer(&owner, &pool_addr, &amt_b);

        // update global PoolState
        let mut ps = read_pool_state(&env);
        ps.liquidity += liquidity;
        write_pool_state(&env, &ps);

        // update tick_lower
        let mut lo = read_tick_info_persistent(&env, lower);
        lo.liquidity_gross += liquidity;
        lo.liquidity_net += liquidity;
        write_tick_info_persistent(&env, lower, &lo);

        // update tick_upper
        let mut up = read_tick_info_persistent(&env, upper);
        up.liquidity_gross += liquidity;
        up.liquidity_net -= liquidity;
        write_tick_info_persistent(&env, upper, &up);

        // update position
        let mut pos = read_position(&env, &owner, lower, upper);
        pos.liquidity += liquidity;
        pos.token_a_amount += amt_a;
        pos.token_b_amount += amt_b;
        write_position(&env, &owner, lower, upper, &pos);
    }

    // ------------------------------------
    // REMOVE LIQUIDITY
    // ------------------------------------

    pub fn remove_liquidity(
        env: Env,
        owner: Address,
        lower: i32,
        upper: i32,
        liquidity: i128,
    ) {
        if liquidity <= 0 {
            panic!("liquidity must > 0");
        }

        let cfg = read_pool_config(&env);
        let pool_addr = pool_address(&env);

        let mut pos = read_position(&env, &owner, lower, upper);
        if pos.liquidity < liquidity {
            panic!("not enough liquidity");
        }

        let out_a = pos.token_a_amount * liquidity / pos.liquidity;
        let out_b = pos.token_b_amount * liquidity / pos.liquidity;

        pos.liquidity -= liquidity;
        pos.token_a_amount -= out_a;
        pos.token_b_amount -= out_b;
        write_position(&env, &owner, lower, upper, &pos);

        // global liquidity
        let mut ps = read_pool_state(&env);
        ps.liquidity -= liquidity;
        write_pool_state(&env, &ps);

        // ticks
        let mut lo = read_tick_info_persistent(&env, lower);
        lo.liquidity_gross -= liquidity;
        lo.liquidity_net -= liquidity;
        write_tick_info_persistent(&env, lower, &lo);

        let mut up = read_tick_info_persistent(&env, upper);
        up.liquidity_gross -= liquidity;
        up.liquidity_net += liquidity;
        write_tick_info_persistent(&env, upper, &up);

        // transfer back
        token::Client::new(&env, &cfg.token_a).transfer(&pool_addr, &owner, &out_a);
        token::Client::new(&env, &cfg.token_b).transfer(&pool_addr, &owner, &out_b);
    }

}



