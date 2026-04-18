use {
    beethoven_client::deposit::marginfi::{
        build_accounts, build_extra_data, MarginfiDepositInput, MARGINFI_PROGRAM_ID,
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
    let user = Address::new_from_array([1; 32]);
    let group = Address::new_from_array([2; 32]);
    let marginfi_account = Address::new_from_array([3; 32]);
    let bank = Address::new_from_array([4; 32]);
    let signer_token_account = Address::new_from_array([5; 32]);
    let liquidity_vault = Address::new_from_array([6; 32]);
    let token_program = Address::new_from_array([7; 32]);
    let extra_mint = Address::new_from_array([8; 32]);

    let input = MarginfiDepositInput {
        user,
        group,
        marginfi_account,
        bank,
        signer_token_account,
        liquidity_vault,
        token_program,
        remaining_accounts: vec![AccountMeta::new_readonly(extra_mint, false)],
    };

    let accounts = build_accounts(&input);

    assert_eq!(accounts[0].pubkey, MARGINFI_PROGRAM_ID);
    assert_eq!(accounts[1].pubkey, group);
    assert_eq!(accounts[2].pubkey, marginfi_account);
    assert_eq!(accounts[3].pubkey, user);
    assert!(accounts[3].is_signer);
    assert_eq!(accounts[4].pubkey, bank);
    assert_eq!(accounts[5].pubkey, signer_token_account);
    assert_eq!(accounts[6].pubkey, liquidity_vault);
    assert_eq!(accounts[7].pubkey, token_program);
    assert_eq!(accounts[8].pubkey, extra_mint);
}
