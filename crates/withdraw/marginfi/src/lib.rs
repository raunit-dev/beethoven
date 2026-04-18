#![no_std]

use {
    beethoven_core::Withdraw,
    core::mem::MaybeUninit,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

pub const MARGINFI_PROGRAM_ID: Address = Address::new_from_array([
    5, 48, 122, 214, 69, 75, 188, 94, 30, 78, 146, 5, 146, 83, 161, 139, 184, 200, 134, 140, 88,
    166, 49, 46, 200, 106, 57, 230, 34, 78, 55, 59,
]);
pub const LENDING_ACCOUNT_WITHDRAW_DISCRIMINATOR: [u8; 8] = [36, 72, 74, 19, 210, 210, 192, 192];
pub const WITHDRAW_DATA_LEN: usize = 18;
const FIXED_ACCOUNTS_LEN: usize = 8;
// `solana-instruction-view` currently caps static CPI account arrays at 64,
// so this adapter can only forward up to 56 trailing remaining accounts.
//
// This is likely workable for normal native-marginfi withdraws: marginfi
// accounts can hold up to 16 balances, and withdraw/borrow risk checks
// require passing all balance banks plus their oracle accounts in
// `remaining_accounts`. In practice that is often on the order of
// 16 * (bank + 1-2 oracle accounts) = 32-48 trailing accounts.
//
// Sources:
// - https://github.com/mrgnlabs/marginfi-v2
// - https://docs.marginfi.com/mfi-v2
const MAX_REMAINING_ACCOUNTS: usize = 56;
const MAX_TOTAL_ACCOUNTS: usize = FIXED_ACCOUNTS_LEN + MAX_REMAINING_ACCOUNTS;

pub struct Marginfi;

pub struct MarginfiWithdrawData {
    pub withdraw_all: Option<bool>,
}

impl MarginfiWithdrawData {
    pub const DATA_LEN: usize = 2;
}

impl TryFrom<&[u8]> for MarginfiWithdrawData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < Self::DATA_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(Self {
            withdraw_all: match data[0] {
                0 => None,
                1 => Some(match data[1] {
                    0 => false,
                    1 => true,
                    _ => return Err(ProgramError::InvalidInstructionData),
                }),
                _ => return Err(ProgramError::InvalidInstructionData),
            },
        })
    }
}

pub struct MarginfiWithdrawAccounts<'info> {
    pub marginfi_program: &'info AccountView,
    pub group: &'info AccountView,
    pub marginfi_account: &'info AccountView,
    pub authority: &'info AccountView,
    pub bank: &'info AccountView,
    pub destination_token_account: &'info AccountView,
    pub bank_liquidity_vault_authority: &'info AccountView,
    pub liquidity_vault: &'info AccountView,
    pub token_program: &'info AccountView,
    pub remaining_accounts: &'info [AccountView],
}

impl<'info> TryFrom<&'info [AccountView]> for MarginfiWithdrawAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [marginfi_program, group, marginfi_account, authority, bank, destination_token_account, bank_liquidity_vault_authority, liquidity_vault, token_program, remaining_accounts @ ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(MarginfiWithdrawAccounts {
            marginfi_program,
            group,
            marginfi_account,
            authority,
            bank,
            destination_token_account,
            bank_liquidity_vault_authority,
            liquidity_vault,
            token_program,
            remaining_accounts,
        })
    }
}

#[inline(never)]
fn invoke_marginfi_withdraw<'info>(
    accounts: &[InstructionAccount<'_>],
    account_infos: &[&'info AccountView],
    instruction_data: &[u8],
    signer_seeds: &[Signer],
) -> ProgramResult {
    let withdraw_ix = InstructionView {
        program_id: &MARGINFI_PROGRAM_ID,
        accounts,
        data: instruction_data,
    };

    invoke_signed_with_bounds::<MAX_TOTAL_ACCOUNTS>(&withdraw_ix, account_infos, signer_seeds)
}

impl<'info> Withdraw<'info> for Marginfi {
    type Accounts = MarginfiWithdrawAccounts<'info>;
    type Data = MarginfiWithdrawData;

    #[inline(never)]
    fn withdraw_signed(
        ctx: &MarginfiWithdrawAccounts<'info>,
        amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        if ctx.remaining_accounts.len() > MAX_REMAINING_ACCOUNTS {
            return Err(ProgramError::InvalidInstructionData);
        }

        let mut accounts = MaybeUninit::<[InstructionAccount; MAX_TOTAL_ACCOUNTS]>::uninit();
        let accounts_ptr = accounts.as_mut_ptr() as *mut InstructionAccount;

        unsafe {
            core::ptr::write(
                accounts_ptr,
                InstructionAccount::readonly(ctx.group.address()),
            );
            core::ptr::write(
                accounts_ptr.add(1),
                InstructionAccount::writable(ctx.marginfi_account.address()),
            );
            core::ptr::write(
                accounts_ptr.add(2),
                InstructionAccount::writable_signer(ctx.authority.address()),
            );
            core::ptr::write(
                accounts_ptr.add(3),
                InstructionAccount::writable(ctx.bank.address()),
            );
            core::ptr::write(
                accounts_ptr.add(4),
                InstructionAccount::writable(ctx.destination_token_account.address()),
            );
            core::ptr::write(
                accounts_ptr.add(5),
                InstructionAccount::readonly(ctx.bank_liquidity_vault_authority.address()),
            );
            core::ptr::write(
                accounts_ptr.add(6),
                InstructionAccount::writable(ctx.liquidity_vault.address()),
            );
            core::ptr::write(
                accounts_ptr.add(7),
                InstructionAccount::readonly(ctx.token_program.address()),
            );

            for (i, account) in ctx.remaining_accounts.iter().enumerate() {
                core::ptr::write(
                    accounts_ptr.add(FIXED_ACCOUNTS_LEN + i),
                    InstructionAccount::from(account),
                );
            }
        }

        let total_accounts_len = FIXED_ACCOUNTS_LEN + ctx.remaining_accounts.len();
        let accounts = unsafe { core::slice::from_raw_parts(accounts_ptr, total_accounts_len) };

        let mut account_infos = [ctx.group; MAX_TOTAL_ACCOUNTS];
        account_infos[1] = ctx.marginfi_account;
        account_infos[2] = ctx.authority;
        account_infos[3] = ctx.bank;
        account_infos[4] = ctx.destination_token_account;
        account_infos[5] = ctx.bank_liquidity_vault_authority;
        account_infos[6] = ctx.liquidity_vault;
        account_infos[7] = ctx.token_program;

        for (i, account) in ctx.remaining_accounts.iter().enumerate() {
            account_infos[FIXED_ACCOUNTS_LEN + i] = account;
        }

        let account_infos = &account_infos[..total_accounts_len];

        let mut instruction_data = MaybeUninit::<[u8; WITHDRAW_DATA_LEN]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(LENDING_ACCOUNT_WITHDRAW_DISCRIMINATOR.as_ptr(), ptr, 8);
            core::ptr::copy_nonoverlapping(amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
            match data.withdraw_all {
                None => {
                    *ptr.add(16) = 0;
                    *ptr.add(17) = 0;
                }
                Some(v) => {
                    *ptr.add(16) = 1;
                    *ptr.add(17) = v as u8;
                }
            }
        }

        invoke_marginfi_withdraw(
            accounts,
            account_infos,
            unsafe { instruction_data.assume_init_ref() },
            signer_seeds,
        )
    }

    #[inline(never)]
    fn withdraw(
        ctx: &MarginfiWithdrawAccounts<'info>,
        amount: u64,
        data: &Self::Data,
    ) -> ProgramResult {
        Self::withdraw_signed(ctx, amount, data, &[])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_marginfi_withdraw_data_option_bool() {
        let none = MarginfiWithdrawData::try_from(&[0, 0][..]).unwrap();
        assert_eq!(none.withdraw_all, None);

        let some_false = MarginfiWithdrawData::try_from(&[1, 0][..]).unwrap();
        assert_eq!(some_false.withdraw_all, Some(false));

        let some_true = MarginfiWithdrawData::try_from(&[1, 1][..]).unwrap();
        assert_eq!(some_true.withdraw_all, Some(true));
    }

    #[test]
    fn reject_invalid_option_bool_payload() {
        assert!(MarginfiWithdrawData::try_from(&[2, 0][..]).is_err());
        assert!(MarginfiWithdrawData::try_from(&[1, 2][..]).is_err());
    }
}
