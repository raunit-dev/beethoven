use {
    beethoven_client::withdraw::marginfi::{
        build_accounts, build_extra_data, MarginfiWithdrawInput, MARGINFI_PROGRAM_ID,
    },
    solana_address::Address,
    solana_instruction::AccountMeta,
};

#[test]
fn marginfi_build_extra_data_matches_option_bool_encoding() {
    assert_eq!(build_extra_data(None), vec![0, 0]);
    assert_eq!(build_extra_data(Some(false)), vec![1, 0]);
    assert_eq!(build_extra_data(Some(true)), vec![1, 1]);
}

#[test]
fn marginfi_build_accounts_keeps_program_first_and_preserves_tail() {
    let input = MarginfiWithdrawInput {
        user: Address::new_from_array([1; 32]),
        group: Address::new_from_array([2; 32]),
        marginfi_account: Address::new_from_array([3; 32]),
        bank: Address::new_from_array([4; 32]),
        destination_token_account: Address::new_from_array([5; 32]),
        bank_liquidity_vault_authority: Address::new_from_array([6; 32]),
        liquidity_vault: Address::new_from_array([7; 32]),
        token_program: Address::new_from_array([8; 32]),
        remaining_accounts: vec![AccountMeta::new_readonly(
            Address::new_from_array([9; 32]),
            false,
        )],
    };

    let accounts = build_accounts(&input);

    assert_eq!(accounts[0].pubkey, MARGINFI_PROGRAM_ID);
    assert_eq!(accounts[1].pubkey, input.group);
    assert_eq!(accounts[2].pubkey, input.marginfi_account);
    assert_eq!(accounts[3].pubkey, input.user);
    assert!(accounts[3].is_signer);
    assert_eq!(accounts[6].pubkey, input.bank_liquidity_vault_authority);
    assert_eq!(accounts[8].pubkey, input.token_program);
    assert_eq!(accounts[9].pubkey, Address::new_from_array([9; 32]));
}
