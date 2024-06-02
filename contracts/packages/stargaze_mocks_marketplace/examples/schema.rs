use cosmwasm_schema::write_api;

use sg_marketplace::msg::{ExecuteMsg, QueryMsg};
use stargaze_mocks_marketplace::msg::InstantiateMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,

    }
}
