#[cfg(test)]
pub mod adapter_scheduler_common;
#[cfg(test)]
pub mod lending_platform;
#[cfg(test)]
pub mod math;
#[cfg(test)]
pub mod oracle;

pub mod helpers {
    pub mod lending_platform;
    pub mod minter;
    pub mod oracle;

    pub mod suite {
        pub mod codes;
        pub mod core;
        pub mod types;
    }

    pub mod adapters {
        pub mod scheduler {
            pub mod common;
            pub mod stargaze;
        }

        pub mod marketplace {
            pub mod kujira;
        }
    }
}
