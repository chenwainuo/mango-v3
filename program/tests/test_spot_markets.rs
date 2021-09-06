mod program_test;
use program_test::*;
use program_test::cookies::*;
use program_test::scenarios::*;
use program_test::assertions::*;
use solana_program_test::*;
use mango::state::ZERO_I80F48;
use fixed::types::I80F48;
use std::collections::HashMap;

#[tokio::test]
async fn test_list_spot_market_on_serum() {
    // === Arrange ===
    let config = MangoProgramTestConfig::default();
    let mut test = MangoProgramTest::start_new(&config).await;
    // Supress some of the logs
    solana_logger::setup_with_default(
        "solana_rbpf::vm=info,\
             solana_runtime::message_processor=debug,\
             solana_runtime::system_instruction_processor=info,\
             solana_program_test=info",
    );
    // Disable all logs except error
    // solana_logger::setup_with("error");

    let mint_index: usize = 0;
    // === Act ===
    let spot_market_cookie = test.list_spot_market(mint_index).await;
    // === Assert ===
    println!("Serum Market PK: {}", spot_market_cookie.market.to_string());
    // Todo: Figure out how to assert this
}

#[tokio::test]
async fn test_init_spot_markets() {

    // === Arrange ===
    let config = MangoProgramTestConfig::default();
    let mut test = MangoProgramTest::start_new(&config).await;
    // Supress some of the logs
    solana_logger::setup_with_default(
        "solana_rbpf::vm=info,\
             solana_runtime::message_processor=debug,\
             solana_runtime::system_instruction_processor=info,\
             solana_program_test=info",
    );
    // Disable all logs except error
    // solana_logger::setup_with("error");
    let mut mango_group_cookie = MangoGroupCookie::default(&mut test).await;

    // === Act ===
    test.add_oracles_to_mango_group(&mango_group_cookie.address).await;
    mango_group_cookie.add_spot_markets(&mut test, config.num_mints - 1).await;

    // === Assert ===
    // TODO: Figure out how to assert

}

#[tokio::test]
async fn test_place_spot_order() {
    // === Arrange ===
    let config = MangoProgramTestConfig { compute_limit: 200_000, num_users: 2, num_mints: 2 };
    let mut test = MangoProgramTest::start_new(&config).await;
    // Supress some of the logs
    solana_logger::setup_with_default(
        "solana_rbpf::vm=info,\
             solana_runtime::message_processor=debug,\
             solana_runtime::system_instruction_processor=info,\
             solana_program_test=info",
    );
    // Disable all logs except error
    // solana_logger::setup_with("error");

    let mut mango_group_cookie = MangoGroupCookie::default(&mut test).await;
    mango_group_cookie.full_setup(&mut test, config.num_users, config.num_mints - 1).await;

    // General parameters
    let user_index: usize = 0;
    let mint_index: usize = 0;
    let base_price: f64 = 10_000.0;
    let base_size: f64 = 1.0;
    let mint = test.with_mint(mint_index);
    let quote_mint = test.quote_mint;

    // Set oracles
    mango_group_cookie.set_oracle(&mut test, mint_index, base_price).await;

    // Deposit amounts
    let user_deposits = vec![
        (user_index, test.quote_index, base_price),
    ];

    // Spot Orders
    let user_spot_orders = vec![
        (user_index, mint_index, serum_dex::matching::Side::Bid, base_size, base_price),
    ];

    // === Act ===
    // Step 1: Make deposits
    deposit_scenario(&mut test, &mut mango_group_cookie, &user_deposits).await;

    // Step 2: Place spot orders
    place_spot_order_scenario(&mut test, &mut mango_group_cookie, &user_spot_orders).await;

    // === Assert ===
    mango_group_cookie.run_keeper(&mut test).await;

    let expected_values_vec: Vec<(usize, usize, HashMap<&str, I80F48>)> = vec![
        (
            0 as usize, // Mint index
            0 as usize, // User index
            [
                ("quote_free", test.to_native(&quote_mint, base_price * base_size)),
                ("quote_locked", ZERO_I80F48),
                ("base_free", test.to_native(&mint, 1.0)),
                ("base_locked", ZERO_I80F48),
            ].iter().cloned().collect(),
        )
    ];

    for expected_values in expected_values_vec {
        assert_user_spot_orders(&mut test, &mango_group_cookie, expected_values).await;
    }

}

#[tokio::test]
async fn test_match_spot_order() {
    // === Arrange ===
    let config = MangoProgramTestConfig { compute_limit: 200_000, num_users: 2, num_mints: 2 };
    let mut test = MangoProgramTest::start_new(&config).await;
    // Supress some of the logs
    solana_logger::setup_with_default(
        "solana_rbpf::vm=info,\
             solana_runtime::message_processor=debug,\
             solana_runtime::system_instruction_processor=info,\
             solana_program_test=info",
    );
    // Disable all logs except error
    // solana_logger::setup_with("error");

    let mut mango_group_cookie = MangoGroupCookie::default(&mut test).await;
    mango_group_cookie.full_setup(&mut test, config.num_users, config.num_mints - 1).await;

    // General parameters
    let bidder_user_index: usize = 0;
    let asker_user_index: usize = 1;
    let mint_index: usize = 0;
    let base_price: f64 = 10_000.0;
    let base_size: f64 = 1.0;
    let mint = test.with_mint(mint_index);
    let quote_mint = test.quote_mint;

    // Set oracles
    mango_group_cookie.set_oracle(&mut test, mint_index, base_price).await;

    // Deposit amounts
    let user_deposits = vec![
        (bidder_user_index, test.quote_index, base_price),
        (asker_user_index, mint_index, 1.0),
    ];

    // Matched Spot Orders
    let matched_spot_orders = vec![
        vec![
            (bidder_user_index, mint_index, serum_dex::matching::Side::Bid, base_size, base_price),
            (asker_user_index, mint_index, serum_dex::matching::Side::Ask, base_size, base_price),
        ],
    ];

    // === Act ===
    // Step 1: Make deposits
    deposit_scenario(&mut test, &mut mango_group_cookie, &user_deposits).await;

    // Step 2: Place and match spot order
    match_spot_order_scenario(&mut test, &mut mango_group_cookie, &matched_spot_orders).await;

    // === Assert ===
    mango_group_cookie.run_keeper(&mut test).await;

    let expected_values_vec: Vec<(usize, usize, HashMap<&str, I80F48>)> = vec![
        (
            0 as usize, // Mint index
            0 as usize, // User index
            [
                ("quote_free", test.to_native(&quote_mint, 3.0)), // serum_dex fee
                ("quote_locked", ZERO_I80F48),
                ("base_free", test.to_native(&mint, 1.0)),
                ("base_locked", ZERO_I80F48),
            ].iter().cloned().collect(),
        ),
        (
            0 as usize, // Mint index
            1 as usize, // User index
            [
                ("quote_free", ZERO_I80F48),
                ("quote_locked", ZERO_I80F48),
                ("base_free", ZERO_I80F48),
                ("base_locked", ZERO_I80F48),
            ].iter().cloned().collect(),
        ),
    ];

    for expected_values in expected_values_vec {
        assert_user_spot_orders(&mut test, &mango_group_cookie, expected_values).await;
    }

}

#[tokio::test]
async fn test_match_and_settle_spot_order() {
    // === Arrange ===
    let config = MangoProgramTestConfig { compute_limit: 200_000, num_users: 2, num_mints: 2 };
    let mut test = MangoProgramTest::start_new(&config).await;
    // Supress some of the logs
    solana_logger::setup_with_default(
        "solana_rbpf::vm=info,\
             solana_runtime::message_processor=debug,\
             solana_runtime::system_instruction_processor=info,\
             solana_program_test=info",
    );
    // Disable all logs except error
    // solana_logger::setup_with("error");

    let mut mango_group_cookie = MangoGroupCookie::default(&mut test).await;
    mango_group_cookie.full_setup(&mut test, config.num_users, config.num_mints - 1).await;

    // General parameters
    let bidder_user_index: usize = 0;
    let asker_user_index: usize = 1;
    let mint_index: usize = 0;
    let base_price: f64 = 10_000.0;
    let base_size: f64 = 1.0;
    let mint = test.with_mint(mint_index);
    let quote_mint = test.quote_mint;

    // Set oracles
    mango_group_cookie.set_oracle(&mut test, mint_index, base_price).await;

    // Deposit amounts
    let user_deposits = vec![
        (bidder_user_index, test.quote_index, base_price),
        (asker_user_index, mint_index, 1.0),
    ];

    // Matched Spot Orders
    let matched_spot_orders = vec![
        vec![
            (bidder_user_index, mint_index, serum_dex::matching::Side::Bid, base_size, base_price),
            (asker_user_index, mint_index, serum_dex::matching::Side::Ask, base_size, base_price),
        ],
    ];

    // === Act ===
    // Step 1: Make deposits
    deposit_scenario(&mut test, &mut mango_group_cookie, &user_deposits).await;

    // Step 2: Place and match spot order
    match_spot_order_scenario(&mut test, &mut mango_group_cookie, &matched_spot_orders).await;

    // Step 3: Settle all spot
    for matched_spot_order in matched_spot_orders {
        mango_group_cookie.settle_spot_funds(&mut test, &matched_spot_order).await;
    }

    // === Assert ===
    mango_group_cookie.run_keeper(&mut test).await;

    let bidder_base_deposit =
        &mango_group_cookie.mango_accounts[bidder_user_index].mango_account
        .get_native_deposit(&mango_group_cookie.mango_cache.root_bank_cache[mint_index], mint_index).unwrap();
    let asker_base_deposit =
        &mango_group_cookie.mango_accounts[asker_user_index].mango_account
        .get_native_deposit(&mango_group_cookie.mango_cache.root_bank_cache[mint_index], mint_index).unwrap();

    // let bidder_quote_deposit =
    //     &mango_group_cookie.mango_accounts[bidder_user_index].mango_account
    //     .get_native_deposit(&mango_group_cookie.mango_cache.root_bank_cache[QUOTE_INDEX], QUOTE_INDEX).unwrap();
    // let asker_quote_deposit =
    //     &mango_group_cookie.mango_accounts[asker_user_index].mango_account
    //     .get_native_deposit(&mango_group_cookie.mango_cache.root_bank_cache[QUOTE_INDEX], QUOTE_INDEX).unwrap();

    assert_eq!(bidder_base_deposit.to_string(), I80F48::from_num(1000000).to_string());
    assert_eq!(asker_base_deposit.to_string(), I80F48::from_num(0).to_string());


}
