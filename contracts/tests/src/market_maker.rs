use cw_multi_test::Executor;
use speculoos::assert_that;

use cosmwasm_std::StdResult;

use goplend_base::oracle::msg::MigrateMsg;

use crate::helpers::{
    market_maker::MarketMakerExtension,
    suite::{
        core::Project,
        types::{ProjectAccount, ProjectNft},
    },
};

#[test]
fn migrate_default() {
    let mut project = Project::new();

    project
        .app
        .migrate_contract(
            ProjectAccount::Admin.into(),
            project.get_market_maker_address(),
            &MigrateMsg {
                version: "2.1.1".to_string(),
            },
            project.get_market_maker_code_id(),
        )
        .unwrap();
}

#[test]
fn query_collection_owner_list_pagination() -> StdResult<()> {
    let mut p = Project::new();

    let collection_list = &[ProjectNft::Pinjeons, ProjectNft::Gopniks];

    for collection in collection_list {
        p.market_maker_try_set_collection(
            ProjectAccount::Admin,
            *collection,
            ProjectAccount::Owner,
        )?;
    }

    let collection_list_response: Vec<String> = p
        .market_maker_query_collection_owner_list(9, &None)?
        .iter()
        .map(|x| x.collection_address.to_string())
        .collect();
    assert_that(&collection_list_response).is_equal_to(
        collection_list
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>(),
    );

    let collection_list_response: Vec<String> = p
        .market_maker_query_collection_owner_list(9, &Some(*collection_list.last().unwrap()))?
        .iter()
        .map(|x| x.collection_address.to_string())
        .collect();
    assert_that(&collection_list_response).is_equal_to(vec![]);

    Ok(())
}
