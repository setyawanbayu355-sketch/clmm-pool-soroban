use soroban_sdk::{contracttype, Address, Env};

#[contracttype]
#[derive(Clone, Debug)]
pub struct Position {
    pub owner: Address,
    pub liquidity: i128,
}

pub fn read_position(env: &Env, owner: &Address) -> Position {
    env.storage().instance().get(owner).unwrap_or(Position {
        owner: owner.clone(),
        liquidity: 0,
    })
}

pub fn write_position(env: &Env, pos: &Position) {
    env.storage().instance().set(&pos.owner, pos);
}

