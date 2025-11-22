use soroban_sdk::{Env, contracttype};

pub type Tick = i32;

#[contracttype]
#[derive(Clone, Debug)]
pub struct TickInfo {
    pub liquidity_gross: i128,
    pub liquidity_net: i128,
}

// --- helper baca tulis tick ke storage ---

pub fn read_tick_info(env: &Env, tick: Tick) -> TickInfo {
    env.storage()
        .instance()
        .get::<Tick, TickInfo>(&tick)
        .unwrap_or(TickInfo {
            liquidity_gross: 0,
            liquidity_net: 0,
        })
}

pub fn write_tick_info(env: &Env, tick: Tick, info: &TickInfo) {
    env.storage().instance().set(&tick, info);
}

// --- dummy find_next_initialized_tick buat v0 ---

pub fn find_next_initialized_tick(
    _env: &Env,
    current_tick: Tick,
    tick_spacing: i32,
    zero_for_one: bool,
) -> Tick {
    if zero_for_one {
        current_tick - tick_spacing
    } else {
        current_tick + tick_spacing
    }
}


// ------------------------------------------------------------
// update_tick (untuk add/remove liquidity)
// ------------------------------------------------------------
//
// Digunakan nanti kalau mau wiring langsung add_liquidity/remove_liquidity
// ke TickInfo via helper ini.
//
pub fn update_tick(
    env: &Env,
    tick: Tick,
    delta_liquidity: i128,
    upper: bool,
) {
    let mut info = read_tick_info(env, tick);

    // Update gross
    info.liquidity_gross += delta_liquidity;

    // Net liquidity:
    //  - tick_lower → upper=false → +L
    //  - tick_upper → upper=true  → -L
    if upper {
        info.liquidity_net -= delta_liquidity;
    } else {
        info.liquidity_net += delta_liquidity;
    }

    write_tick_info(env, tick, &info);
}



