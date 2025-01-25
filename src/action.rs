pub enum Action {
    MakeFood,
    GiveMoney(GiveInfo),
    GiveFood(GiveInfo),
    Converse(String, usize),
    Broadcast(String, usize),
}

pub struct GiveInfo {
    amount: u32,
    agent_trading_with: usize,
}
