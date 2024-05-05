use self::bet::BetService;

pub mod bet;

#[derive(Clone)]
pub struct Services {
    pub bet: BetService,
}

impl Services {
    pub fn new() -> Self {
        Self {
            bet: BetService::new(),
        }
    }
}
