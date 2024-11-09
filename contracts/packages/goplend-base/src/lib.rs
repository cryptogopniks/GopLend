pub mod oracle {
    pub mod msg;
    pub mod state;
    pub mod types;
}

pub mod lending_platform {
    pub mod msg;
    pub mod state;
    pub mod types;
}

pub mod minter {
    pub mod msg;
    pub mod state;
    pub mod types;
}

pub mod market_maker {
    pub mod types;

    pub mod hub {
        pub mod msg;
        pub mod state;
    }

    pub mod outpost {
        pub mod msg;
        pub mod state;
    }
}

pub mod adapters {
    pub mod marketplace {

        pub mod stargaze {
            pub mod msg;
            pub mod state;
            pub mod types;
        }
    }

    pub mod scheduler {
        pub mod common {
            pub mod msg;
            pub mod state;
            pub mod types;
        }
    }
}

pub mod assets;
pub mod constants;
pub mod converters;
pub mod error;
pub mod math;
pub mod utils;
