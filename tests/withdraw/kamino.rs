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

fn build_kamino_accounts(
    refresh_reserve_group_count: usize,
    obligation_tail_count: usize,
) -> (Vec<Vec<u64>>, Vec<AccountView>, Vec<Address>) {
    let total_accounts = 22
        + refresh_reserve_group_count
            * beethoven::kamino_withdraw::KaminoWithdrawData::REFRESH_RESERVE_GROUP_ACCOUNTS_LEN;
    let total_accounts = total_accounts + obligation_tail_count;
    let mut addresses = Vec::with_capacity(total_accounts);
    addresses.push(beethoven::kamino_withdraw::KAMINO_LEND_PROGRAM_ID);
    for i in 1..total_accounts {
        addresses.push(Address::new_from_array([i as u8; 32]));
    }

    let mut storage = Vec::with_capacity(total_accounts);
    let mut views = Vec::with_capacity(total_accounts);

    for (index, address) in addresses.iter().copied().enumerate() {
        let (backing, view) = make_account(address, index == 1);
        storage.push(backing);
        views.push(view);
    }

    (storage, views, addresses)
}

#[test]
fn kamino_withdraw_accounts_try_from_keeps_refresh_tail() {
    let (_storage, accounts, addresses) = build_kamino_accounts(2, 4);

    let ctx =
        beethoven::kamino_withdraw::KaminoWithdrawAccounts::try_from(accounts.as_slice()).unwrap();

    assert_eq!(ctx.kamino_lending_program.address(), &addresses[0]);
    assert_eq!(ctx.owner.address(), &addresses[1]);
    assert_eq!(ctx.obligation.address(), &addresses[2]);
    assert_eq!(ctx.reserve_scope_prices.address(), &addresses[21]);
    assert_eq!(ctx.remaining_accounts.len(), 14);
    assert_eq!(ctx.remaining_accounts[0].address(), &addresses[22]);
    assert_eq!(ctx.remaining_accounts[13].address(), &addresses[35]);
}

#[test]
fn kamino_withdraw_accounts_try_from_requires_fixed_accounts() {
    let (_storage, accounts, _) = build_kamino_accounts(0, 0);

    let err = match beethoven::kamino_withdraw::KaminoWithdrawAccounts::try_from(&accounts[..21]) {
        Ok(_) => panic!("expected NotEnoughAccountKeys"),
        Err(err) => err,
    };
    assert_eq!(err, ProgramError::NotEnoughAccountKeys);
}

#[test]
fn try_from_withdraw_context_selects_kamino() {
    let (_storage, accounts, _) = build_kamino_accounts(1, 0);

    let ctx = try_from_withdraw_context(accounts.as_slice()).unwrap();
    assert!(matches!(ctx, WithdrawContext::Kamino(_)));
}

#[test]
fn kamino_context_try_from_withdraw_data_parses_counts_and_rest() {
    let (_storage, accounts, _) = build_kamino_accounts(1, 0);
    let ctx = try_from_withdraw_context(accounts.as_slice()).unwrap();

    let (withdraw_data, rest) = ctx.try_from_withdraw_data(&[2, 3, 1, 1, 99, 100]).unwrap();
    assert_eq!(rest, &[99, 100]);

    let WithdrawData::Kamino(data) = withdraw_data;
    assert_eq!(data.refresh_reserve_group_count, 2);
    assert_eq!(data.deposit_reserve_count, 3);
    assert_eq!(data.borrow_reserve_count, 1);
    assert_eq!(data.borrow_referrer_token_state_count, 1);
}

#[test]
fn kamino_context_try_from_withdraw_data_rejects_invalid_count() {
    let (_storage, accounts, _) = build_kamino_accounts(0, 0);
    let ctx = try_from_withdraw_context(accounts.as_slice()).unwrap();

    let err = match ctx.try_from_withdraw_data(&[0, 0, 1, 2]) {
        Ok(_) => panic!("expected InvalidInstructionData"),
        Err(err) => err,
    };
    assert_eq!(err, ProgramError::InvalidInstructionData);
}
