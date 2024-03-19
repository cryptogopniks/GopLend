use lending_platform::math::calc_liquidation_sets;

#[test]
fn default() {
    let collateral_list: Vec<u128> = vec![5, 16, 14, 2, 10, 13, 7, 19, 5];
    let target_cost: u128 = 11;

    let res = calc_liquidation_sets(&collateral_list, target_cost);

    println!("{:#?}", res);
}
