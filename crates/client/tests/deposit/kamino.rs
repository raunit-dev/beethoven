use {
    beethoven_client::deposit::kamino::{
        build_accounts, build_extra_data, KaminoDepositInput, FARMS_PROGRAM_ID,
        KAMINO_LEND_PROGRAM_ID,
    },
    solana_address::Address,
    solana_instruction::AccountMeta,
};

#[test]
fn kamino_build_extra_data_keeps_explicit_counts() {
    assert_eq!(build_extra_data(2, 3, 1, 1), vec![2, 3, 1, 1]);
}

#[test]
fn kamino_build_accounts_keeps_program_first_and_preserves_tail() {
    let input = KaminoDepositInput {
        user: Address::new_from_array([1; 32]),
        obligation: Address::new_from_array([2; 32]),
        lending_market: Address::new_from_array([3; 32]),
        lending_market_authority: Address::new_from_array([4; 32]),
        reserve: Address::new_from_array([5; 32]),
        reserve_liquidity_mint: Address::new_from_array([6; 32]),
        reserve_liquidity_supply: Address::new_from_array([7; 32]),
        reserve_collateral_mint: Address::new_from_array([8; 32]),
        reserve_destination_deposit_collateral: Address::new_from_array([9; 32]),
        user_source_liquidity: Address::new_from_array([10; 32]),
        placeholder_user_destination_collateral: KAMINO_LEND_PROGRAM_ID,
        collateral_token_program: beethoven_client::TOKEN_PROGRAM_ID,
        liquidity_token_program: beethoven_client::TOKEN_2022_PROGRAM_ID,
        instruction_sysvar_account: beethoven_client::SYSVAR_INSTRUCTIONS_ID,
        obligation_farm_user_state: Address::new_from_array([11; 32]),
        reserve_farm_state: Address::new_from_array([12; 32]),
        farms_program: FARMS_PROGRAM_ID,
        reserve_pyth_oracle: Address::new_from_array([13; 32]),
        reserve_switchboard_price_oracle: Address::new_from_array([14; 32]),
        reserve_switchboard_twap_oracle: Address::new_from_array([15; 32]),
        reserve_scope_prices: Address::new_from_array([16; 32]),
        remaining_accounts: vec![AccountMeta::new(Address::new_from_array([17; 32]), false)],
    };

    let accounts = build_accounts(&input);

    assert_eq!(accounts.len(), 23);
    assert_eq!(accounts[0].pubkey, KAMINO_LEND_PROGRAM_ID);
    assert_eq!(accounts[1].pubkey, input.user);
    assert!(accounts[1].is_signer);
    assert_eq!(accounts[2].pubkey, input.obligation);
    assert_eq!(accounts[21].pubkey, input.reserve_scope_prices);
    assert_eq!(accounts[22].pubkey, Address::new_from_array([17; 32]));
}
