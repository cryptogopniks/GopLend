use cosmwasm_std::{testing::MockStorage, Addr, Binary, Decimal, Empty, StdResult};
use cw_gopniks::assets::Token;
use cw_multi_test::{
    App, AppResponse, BankKeeper, DistributionKeeper, FailingModule, GovFailingModule,
    IbcFailingModule, MockApiBech32, StakeKeeper, WasmKeeper,
};

use anyhow::Error;
use strum_macros::{Display, EnumIter, IntoStaticStr};

use goplend_base::{constants::P12, converters::str_to_dec};

pub const DEFAULT_FUNDS_AMOUNT: u128 = P12; // give each user 1 asset (1 CRD, 1 INJ, etc.)
pub const INCREASED_FUNDS_AMOUNT: u128 = 100 * P12; // give admin such amount of assets to ensure providing 1e6 of assets to each pair

pub const DEFAULT_DECIMALS: u8 = 6;
pub const INCREASED_DECIMALS: u8 = 18;

pub type CustomApp = App<
    BankKeeper,
    MockApiBech32,
    MockStorage,
    FailingModule<Empty, Empty, Empty>,
    WasmKeeper<Empty, Empty>,
    StakeKeeper,
    DistributionKeeper,
    IbcFailingModule,
    GovFailingModule,
>;

#[derive(Debug, Clone, Copy, Display, IntoStaticStr, EnumIter)]
pub enum ProjectAccount {
    #[strum(serialize = "wasm1335hded4gyzpt00fpz75mms4m7ck02wgw07yhw9grahj4dzg4yvqvrz0p4")]
    Admin,
    #[strum(serialize = "wasm190vqdjtlpcq27xslcveglfmr4ynfwg7gmw86cnun4acakxrdd6gqy7k8ya")]
    Alice,
    #[strum(serialize = "wasm1sxmr0k8u6trd5c6eu6trzyapzux7090ykujmsng7pdx0m8k93n5s6sey0n")]
    Bob,
    #[strum(serialize = "wasm1jmvkxtekx4jvcvpj2g2qnnez4pf0yqewasyea4vk0sxsqr8vvpaq7wvjwx")]
    John,
    #[strum(serialize = "wasm19sjfgua22qwy8u25e3dayvevf3dt22qum37z9wsyjujgstk6yhmsvkkpgy")]
    Kate,
    #[strum(serialize = "wasm1hyfcr98la8nu3wmd08g76439j4farrvukc9kdca6t23wtvrcq4dq3cnj04")]
    Ruby,
    #[strum(serialize = "wasm1fsgzj6t7udv8zhf6zj32mkqhcjcpv52yph5qsdcl0qt94jgdckqszmp4sw")]
    Owner,
    #[strum(serialize = "wasm17u86jdhzml8558g3xu8pfvzzvygrypxckx89kryj53pqnhdhxxzq4nfaaw")]
    Scheduler,
}

impl ProjectAccount {
    pub fn get_initial_funds_amount(&self) -> u128 {
        match self {
            ProjectAccount::Admin => INCREASED_FUNDS_AMOUNT,
            ProjectAccount::Alice => DEFAULT_FUNDS_AMOUNT,
            ProjectAccount::Bob => DEFAULT_FUNDS_AMOUNT,
            ProjectAccount::John => DEFAULT_FUNDS_AMOUNT,
            ProjectAccount::Kate => DEFAULT_FUNDS_AMOUNT,
            ProjectAccount::Ruby => DEFAULT_FUNDS_AMOUNT,
            ProjectAccount::Owner => DEFAULT_FUNDS_AMOUNT,
            ProjectAccount::Scheduler => DEFAULT_FUNDS_AMOUNT,
        }
    }
}

#[derive(Debug, Clone, Copy, Display, IntoStaticStr, EnumIter)]
pub enum ProjectCoin {
    #[strum(serialize = "factory/wasm1s/ustars")]
    Stars,
    #[strum(serialize = "factory/wasm1s/uusdc")]
    Usdc,
    #[strum(serialize = "factory/wasm1s/ukuji")]
    Kuji,
    #[strum(serialize = "factory/wasm1s/uusk")]
    Usk,
}

#[derive(Debug, Clone, Copy, Display, IntoStaticStr, EnumIter)]
pub enum ProjectToken {
    #[strum(serialize = "wasm1mzdhwvvh22wrt07w59wxyd58822qavwkx5lcej7aqfkpqqlhaqfsqq5gpq")]
    Atom,
    #[strum(serialize = "wasm14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s0phg4d")]
    Luna,
    #[strum(serialize = "wasm1suhgf5svhu4usrurvxzlgn54ksxmn8gljarjtxqnapv8kjnp4nrss5maay")]
    Inj,
}

#[derive(Debug, Clone, Copy, Display, IntoStaticStr, EnumIter)]
pub enum ProjectNft {
    #[strum(serialize = "wasm1xr3rq8yvd7qplsw5yx90ftsr2zdhg4e9z60h5duusgxpv72hud3s0nakef")]
    Gopniks,
    #[strum(serialize = "wasm1aakfpghcanxtc45gpqlx8j3rq0zcpyf49qmhm9mdjrfx036h4z5se0hfnq")]
    Pinjeons,
}

pub trait GetPrice {
    fn get_price(&self) -> Decimal;
}

impl GetPrice for ProjectAsset {
    fn get_price(&self) -> Decimal {
        match self {
            ProjectAsset::Coin(project_coin) => project_coin.get_price(),
            ProjectAsset::Token(project_token) => project_token.get_price(),
        }
    }
}

impl GetPrice for ProjectCoin {
    fn get_price(&self) -> Decimal {
        match self {
            ProjectCoin::Stars => str_to_dec("0.01"),
            ProjectCoin::Usdc => str_to_dec("1"),
            ProjectCoin::Kuji => str_to_dec("1.5"),
            ProjectCoin::Usk => str_to_dec("1"),
        }
    }
}

impl GetPrice for ProjectToken {
    fn get_price(&self) -> Decimal {
        match self {
            ProjectToken::Atom => str_to_dec("10"),
            ProjectToken::Luna => str_to_dec("0.5"),
            ProjectToken::Inj => str_to_dec("5"),
        }
    }
}

pub trait GetDecimals {
    fn get_decimals(&self) -> u8;
}

impl GetDecimals for ProjectAsset {
    fn get_decimals(&self) -> u8 {
        match self {
            ProjectAsset::Coin(project_coin) => project_coin.get_decimals(),
            ProjectAsset::Token(project_token) => project_token.get_decimals(),
        }
    }
}

impl GetDecimals for ProjectCoin {
    fn get_decimals(&self) -> u8 {
        match self {
            ProjectCoin::Stars => DEFAULT_DECIMALS,
            ProjectCoin::Usdc => DEFAULT_DECIMALS,
            ProjectCoin::Kuji => DEFAULT_DECIMALS,
            ProjectCoin::Usk => DEFAULT_DECIMALS,
        }
    }
}

impl GetDecimals for ProjectToken {
    fn get_decimals(&self) -> u8 {
        match self {
            ProjectToken::Atom => DEFAULT_DECIMALS,
            ProjectToken::Luna => DEFAULT_DECIMALS,
            ProjectToken::Inj => INCREASED_DECIMALS,
        }
    }
}

impl From<ProjectAccount> for Addr {
    fn from(project_account: ProjectAccount) -> Self {
        Self::unchecked(project_account.to_string())
    }
}

impl From<ProjectToken> for Addr {
    fn from(project_token: ProjectToken) -> Self {
        Addr::unchecked(project_token.to_string())
    }
}

impl From<ProjectNft> for Addr {
    fn from(project_nft: ProjectNft) -> Self {
        Addr::unchecked(project_nft.to_string())
    }
}

impl From<ProjectCoin> for Token {
    fn from(project_coin: ProjectCoin) -> Self {
        Self::new_native(&project_coin.to_string())
    }
}

impl From<ProjectToken> for Token {
    fn from(project_token: ProjectToken) -> Self {
        Self::new_cw20(&project_token.into())
    }
}

#[derive(Debug, Clone, Copy, Display)]
pub enum ProjectAsset {
    Coin(ProjectCoin),
    Token(ProjectToken),
}

impl From<ProjectCoin> for ProjectAsset {
    fn from(project_coin: ProjectCoin) -> Self {
        Self::Coin(project_coin)
    }
}

impl From<ProjectToken> for ProjectAsset {
    fn from(project_token: ProjectToken) -> Self {
        Self::Token(project_token)
    }
}

#[derive(Debug, Clone, Copy, EnumIter)]
pub enum ProjectPair {
    AtomLuna,
    StarsInj,
    StarsLuna,
    StarsNoria,
}

impl ProjectPair {
    pub fn split_pair(&self) -> (ProjectAsset, ProjectAsset) {
        match self {
            ProjectPair::AtomLuna => (ProjectToken::Atom.into(), ProjectToken::Luna.into()),
            ProjectPair::StarsInj => (ProjectCoin::Kuji.into(), ProjectToken::Inj.into()),
            ProjectPair::StarsLuna => (ProjectCoin::Kuji.into(), ProjectToken::Luna.into()),
            ProjectPair::StarsNoria => (ProjectCoin::Kuji.into(), ProjectCoin::Usk.into()),
        }
    }
}

#[derive(Debug)]
pub enum WrappedResponse {
    Execute(Result<AppResponse, Error>),
    Query(StdResult<Binary>),
}

impl From<Result<AppResponse, Error>> for WrappedResponse {
    fn from(exec_res: Result<AppResponse, Error>) -> Self {
        Self::Execute(exec_res)
    }
}

impl From<StdResult<Binary>> for WrappedResponse {
    fn from(query_res: StdResult<Binary>) -> Self {
        Self::Query(query_res)
    }
}
