use {
    beethoven::{try_from_withdraw_context, WithdrawContext, WithdrawData},
    solana_account_view::{AccountView, RuntimeAccount, NOT_BORROWED},
    solana_address::Address,
    solana_program_error::ProgramError,
};

fn make_account(address: Address, is_signer: bool) -> (Vec<u64>, AccountView) {
    let mut backing =
        vec![0u64; core::mem::size_of::<RuntimeAccount>() / core::mem::size_of::<u64>() + 1];
    let raw = backing.as_mut_ptr() as *mut RuntimeAccount;

    unsafe {
        (*raw).borrow_state = NOT_BORROWED;
        (*raw).is_signer = is_signer as u8;
        (*raw).is_writable = 1;
        (*raw).executable = 0;
        (*raw).resize_delta = 0;
        (*raw).address = address;
        (*raw).owner = Address::new_from_array([9u8; 32]);
        (*raw).lamports = 0;
        (*raw).data_len = 8;
    }

    let view = unsafe { AccountView::new_unchecked(raw) };
    (backing, view)
}

fn build_marginfi_accounts(
    include_remaining_tail: bool,
) -> (Vec<Vec<u64>>, Vec<AccountView>, [Address; 10]) {
    let addresses = [
        beethoven::marginfi_withdraw::MARGINFI_PROGRAM_ID,
        Address::new_from_array([1; 32]),
        Address::new_from_array([2; 32]),
        Address::new_from_array([3; 32]),
        Address::new_from_array([4; 32]),
        Address::new_from_array([5; 32]),
        Address::new_from_array([6; 32]),
        Address::new_from_array([7; 32]),
        Address::new_from_array([8; 32]),
        Address::new_from_array([10; 32]),
    ];

    let total = if include_remaining_tail { 10 } else { 9 };
    let mut storage = Vec::with_capacity(total);
    let mut views = Vec::with_capacity(total);

    for (index, address) in addresses.iter().copied().take(total).enumerate() {
        let (backing, view) = make_account(address, index == 3);
        storage.push(backing);
        views.push(view);
    }

    (storage, views, addresses)
}

#[test]
fn marginfi_withdraw_accounts_try_from_parses_remaining_tail() {
    let (_storage, accounts, addresses) = build_marginfi_accounts(true);

    let ctx = beethoven::marginfi_withdraw::MarginfiWithdrawAccounts::try_from(accounts.as_slice())
        .unwrap();

    assert_eq!(ctx.marginfi_program.address(), &addresses[0]);
    assert_eq!(ctx.group.address(), &addresses[1]);
    assert_eq!(ctx.marginfi_account.address(), &addresses[2]);
    assert_eq!(ctx.authority.address(), &addresses[3]);
    assert_eq!(ctx.bank.address(), &addresses[4]);
    assert_eq!(ctx.destination_token_account.address(), &addresses[5]);
    assert_eq!(ctx.bank_liquidity_vault_authority.address(), &addresses[6]);
    assert_eq!(ctx.liquidity_vault.address(), &addresses[7]);
    assert_eq!(ctx.token_program.address(), &addresses[8]);
    assert_eq!(ctx.remaining_accounts.len(), 1);
    assert_eq!(ctx.remaining_accounts[0].address(), &addresses[9]);
}

#[test]
fn marginfi_withdraw_accounts_try_from_requires_fixed_accounts() {
    let (_storage, accounts, _) = build_marginfi_accounts(false);
    let err = match beethoven::marginfi_withdraw::MarginfiWithdrawAccounts::try_from(&accounts[..8])
    {
        Ok(_) => panic!("expected NotEnoughAccountKeys"),
        Err(err) => err,
    };
    assert_eq!(err, ProgramError::NotEnoughAccountKeys);
}

#[test]
fn try_from_withdraw_context_selects_marginfi() {
    let (_storage, accounts, _) = build_marginfi_accounts(true);

    let ctx = try_from_withdraw_context(accounts.as_slice()).unwrap();
    assert!(matches!(ctx, WithdrawContext::Marginfi(_)));
}

#[test]
fn marginfi_context_try_from_withdraw_data_parses_option_bool() {
    let (_storage, accounts, _) = build_marginfi_accounts(true);
    let ctx = try_from_withdraw_context(accounts.as_slice()).unwrap();

    let (none, rest) = ctx.try_from_withdraw_data(&[0, 0]).unwrap();
    assert!(rest.is_empty());
    let WithdrawData::Marginfi(data) = none;
    assert_eq!(data.withdraw_all, None);

    let (some_false, rest) = ctx.try_from_withdraw_data(&[1, 0]).unwrap();
    assert!(rest.is_empty());
    let WithdrawData::Marginfi(data) = some_false;
    assert_eq!(data.withdraw_all, Some(false));

    let (some_true, rest) = ctx.try_from_withdraw_data(&[1, 1]).unwrap();
    assert!(rest.is_empty());
    let WithdrawData::Marginfi(data) = some_true;
    assert_eq!(data.withdraw_all, Some(true));
}

#[test]
fn marginfi_context_try_from_withdraw_data_rejects_invalid_option_bool() {
    let (_storage, accounts, _) = build_marginfi_accounts(true);
    let ctx = try_from_withdraw_context(accounts.as_slice()).unwrap();

    let err = match ctx.try_from_withdraw_data(&[1, 9]) {
        Ok(_) => panic!("expected InvalidInstructionData"),
        Err(err) => err,
    };
    assert_eq!(err, ProgramError::InvalidInstructionData);
}
