#![no_std]

use {
    beethoven_core::Withdraw,
    core::mem::MaybeUninit,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed, invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

pub const KAMINO_LEND_PROGRAM_ID: Address = Address::new_from_array([
    4, 178, 172, 177, 18, 88, 204, 227, 104, 44, 65, 139, 168, 114, 255, 61, 249, 17, 2, 113, 47,
    21, 175, 18, 182, 190, 105, 179, 67, 91, 0, 8,
]);
const REFRESH_RESERVE_DISCRIMINATOR: [u8; 8] = [2, 218, 138, 235, 79, 201, 25, 102];
const REFRESH_OBLIGATION_DISCRIMINATOR: [u8; 8] = [33, 132, 147, 228, 151, 192, 72, 89];
const WITHDRAW_OBLIGATION_COLLATERAL_AND_REDEEM_RESERVE_COLLATERAL_V2_DISCRIMINATOR: [u8; 8] =
    [235, 52, 119, 152, 149, 197, 20, 7];

pub struct Kamino;

pub struct KaminoWithdrawData {
    pub refresh_reserve_group_count: u8,
    pub deposit_reserve_count: u8,
    pub borrow_reserve_count: u8,
    pub borrow_referrer_token_state_count: u8,
}

impl KaminoWithdrawData {
    pub const DATA_LEN: usize = 4;
    pub const MAX_REFRESH_RESERVE_GROUPS: usize = 13;
    pub const MAX_DEPOSIT_RESERVES: usize = 8;
    pub const MAX_BORROW_RESERVES: usize = 5;
    pub const MAX_BORROW_REFERRER_TOKEN_STATES: usize = 5;
    pub const REFRESH_RESERVE_GROUP_ACCOUNTS_LEN: usize = 5;
}

impl TryFrom<&[u8]> for KaminoWithdrawData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < Self::DATA_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }

        let refresh_reserve_group_count = data[0];
        let deposit_reserve_count = data[1];
        let borrow_reserve_count = data[2];
        let borrow_referrer_token_state_count = data[3];
        if refresh_reserve_group_count as usize > Self::MAX_REFRESH_RESERVE_GROUPS
            || deposit_reserve_count as usize > Self::MAX_DEPOSIT_RESERVES
            || borrow_reserve_count as usize > Self::MAX_BORROW_RESERVES
            || borrow_referrer_token_state_count as usize > Self::MAX_BORROW_REFERRER_TOKEN_STATES
            || borrow_referrer_token_state_count > borrow_reserve_count
        {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(Self {
            refresh_reserve_group_count,
            deposit_reserve_count,
            borrow_reserve_count,
            borrow_referrer_token_state_count,
        })
    }
}

pub struct KaminoWithdrawAccounts<'info> {
    pub kamino_lending_program: &'info AccountView,
    pub owner: &'info AccountView,
    pub obligation: &'info AccountView,
    pub lending_market: &'info AccountView,
    pub lending_market_authority: &'info AccountView,
    pub withdraw_reserve: &'info AccountView,
    pub reserve_liquidity_mint: &'info AccountView,
    pub reserve_source_collateral: &'info AccountView,
    pub reserve_collateral_mint: &'info AccountView,
    pub reserve_liquidity_supply: &'info AccountView,
    pub user_destination_liquidity: &'info AccountView,
    pub placeholder_user_destination_collateral: &'info AccountView,
    pub token_program: &'info AccountView,
    pub liquidity_token_program: &'info AccountView,
    pub instruction_sysvar_account: &'info AccountView,
    pub obligation_farm_user_state: &'info AccountView,
    pub reserve_farm_state: &'info AccountView,
    pub farms_program: &'info AccountView,
    pub reserve_pyth_oracle: &'info AccountView,
    pub reserve_switchboard_price_oracle: &'info AccountView,
    pub reserve_switchboard_twap_oracle: &'info AccountView,
    pub reserve_scope_prices: &'info AccountView,
    pub remaining_accounts: &'info [AccountView],
}

impl<'info> TryFrom<&'info [AccountView]> for KaminoWithdrawAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [kamino_lending_program, owner, obligation, lending_market, lending_market_authority, withdraw_reserve, reserve_liquidity_mint, reserve_source_collateral, reserve_collateral_mint, reserve_liquidity_supply, user_destination_liquidity, placeholder_user_destination_collateral, token_program, liquidity_token_program, instruction_sysvar_account, obligation_farm_user_state, reserve_farm_state, farms_program, reserve_pyth_oracle, reserve_switchboard_price_oracle, reserve_switchboard_twap_oracle, reserve_scope_prices, remaining_accounts @ ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(KaminoWithdrawAccounts {
            kamino_lending_program,
            owner,
            obligation,
            lending_market,
            lending_market_authority,
            withdraw_reserve,
            reserve_liquidity_mint,
            reserve_source_collateral,
            reserve_collateral_mint,
            reserve_liquidity_supply,
            user_destination_liquidity,
            placeholder_user_destination_collateral,
            token_program,
            liquidity_token_program,
            instruction_sysvar_account,
            obligation_farm_user_state,
            reserve_farm_state,
            farms_program,
            reserve_pyth_oracle,
            reserve_switchboard_price_oracle,
            reserve_switchboard_twap_oracle,
            reserve_scope_prices,
            remaining_accounts,
        })
    }
}

fn refresh_reserve_signed(
    reserve: &AccountView,
    lending_market: &AccountView,
    pyth_oracle: &AccountView,
    switchboard_price_oracle: &AccountView,
    switchboard_twap_oracle: &AccountView,
    scope_prices: &AccountView,
    signer_seeds: &[Signer],
) -> ProgramResult {
    let accounts = [
        InstructionAccount::writable(reserve.address()),
        InstructionAccount::readonly(lending_market.address()),
        InstructionAccount::readonly(pyth_oracle.address()),
        InstructionAccount::readonly(switchboard_price_oracle.address()),
        InstructionAccount::readonly(switchboard_twap_oracle.address()),
        InstructionAccount::readonly(scope_prices.address()),
    ];

    let account_infos = [
        reserve,
        lending_market,
        pyth_oracle,
        switchboard_price_oracle,
        switchboard_twap_oracle,
        scope_prices,
    ];

    let instruction = InstructionView {
        program_id: &KAMINO_LEND_PROGRAM_ID,
        accounts: &accounts,
        data: &REFRESH_RESERVE_DISCRIMINATOR,
    };

    invoke_signed(&instruction, &account_infos, signer_seeds)
}

impl<'info> Withdraw<'info> for Kamino {
    type Accounts = KaminoWithdrawAccounts<'info>;
    type Data = KaminoWithdrawData;

    fn withdraw_signed(
        ctx: &KaminoWithdrawAccounts<'info>,
        amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let refresh_reserve_group_count = data.refresh_reserve_group_count as usize;
        let deposit_reserve_count = data.deposit_reserve_count as usize;
        let borrow_reserve_count = data.borrow_reserve_count as usize;
        let borrow_referrer_token_state_count = data.borrow_referrer_token_state_count as usize;
        let refresh_reserve_accounts_len =
            refresh_reserve_group_count * KaminoWithdrawData::REFRESH_RESERVE_GROUP_ACCOUNTS_LEN;
        let expected_remaining_accounts = refresh_reserve_accounts_len
            + deposit_reserve_count
            + borrow_reserve_count
            + borrow_referrer_token_state_count;
        if ctx.remaining_accounts.len() != expected_remaining_accounts {
            return Err(ProgramError::InvalidInstructionData);
        }

        let (refresh_reserve_accounts, obligation_accounts) = ctx
            .remaining_accounts
            .split_at(refresh_reserve_accounts_len);
        let (deposit_reserve_accounts, obligation_accounts) =
            obligation_accounts.split_at(deposit_reserve_count);
        let (borrow_reserve_accounts, borrow_referrer_token_state_accounts) =
            obligation_accounts.split_at(borrow_reserve_count);

        refresh_reserve_signed(
            ctx.withdraw_reserve,
            ctx.lending_market,
            ctx.reserve_pyth_oracle,
            ctx.reserve_switchboard_price_oracle,
            ctx.reserve_switchboard_twap_oracle,
            ctx.reserve_scope_prices,
            signer_seeds,
        )?;

        let reserve_refresh_groups = refresh_reserve_accounts
            .chunks_exact(KaminoWithdrawData::REFRESH_RESERVE_GROUP_ACCOUNTS_LEN);
        for group in reserve_refresh_groups {
            let [reserve, pyth_oracle, switchboard_price_oracle, switchboard_twap_oracle, scope_prices] =
                group
            else {
                return Err(ProgramError::InvalidInstructionData);
            };

            refresh_reserve_signed(
                reserve,
                ctx.lending_market,
                pyth_oracle,
                switchboard_price_oracle,
                switchboard_twap_oracle,
                scope_prices,
                signer_seeds,
            )?;
        }

        const MAX_REFRESH_OBLIGATION_ACCOUNTS: usize = 2
            + KaminoWithdrawData::MAX_DEPOSIT_RESERVES
            + KaminoWithdrawData::MAX_BORROW_RESERVES
            + KaminoWithdrawData::MAX_BORROW_REFERRER_TOKEN_STATES;

        let mut obligation_accounts =
            MaybeUninit::<[InstructionAccount; MAX_REFRESH_OBLIGATION_ACCOUNTS]>::uninit();
        let obligation_accounts_ptr = obligation_accounts.as_mut_ptr() as *mut InstructionAccount;

        unsafe {
            core::ptr::write(
                obligation_accounts_ptr,
                InstructionAccount::readonly(ctx.lending_market.address()),
            );
            core::ptr::write(
                obligation_accounts_ptr.add(1),
                InstructionAccount::writable(ctx.obligation.address()),
            );
            let mut next_account_index = 2;

            for reserve in deposit_reserve_accounts {
                core::ptr::write(
                    obligation_accounts_ptr.add(next_account_index),
                    InstructionAccount::writable(reserve.address()),
                );
                next_account_index += 1;
            }

            for reserve in borrow_reserve_accounts {
                core::ptr::write(
                    obligation_accounts_ptr.add(next_account_index),
                    InstructionAccount::writable(reserve.address()),
                );
                next_account_index += 1;
            }

            for referrer_token_state in borrow_referrer_token_state_accounts {
                core::ptr::write(
                    obligation_accounts_ptr.add(next_account_index),
                    InstructionAccount::writable(referrer_token_state.address()),
                );
                next_account_index += 1;
            }
        }

        let obligation_accounts_len =
            2 + deposit_reserve_count + borrow_reserve_count + borrow_referrer_token_state_count;
        let obligation_accounts_slice = unsafe {
            core::slice::from_raw_parts(obligation_accounts_ptr, obligation_accounts_len)
        };

        let mut obligation_account_infos = [ctx.lending_market; MAX_REFRESH_OBLIGATION_ACCOUNTS];
        obligation_account_infos[0] = ctx.lending_market;
        obligation_account_infos[1] = ctx.obligation;
        let mut next_account_index = 2;

        for reserve in deposit_reserve_accounts {
            obligation_account_infos[next_account_index] = reserve;
            next_account_index += 1;
        }

        for reserve in borrow_reserve_accounts {
            obligation_account_infos[next_account_index] = reserve;
            next_account_index += 1;
        }

        for referrer_token_state in borrow_referrer_token_state_accounts {
            obligation_account_infos[next_account_index] = referrer_token_state;
            next_account_index += 1;
        }

        let obligation_account_infos = &obligation_account_infos[..obligation_accounts_len];

        let instruction = InstructionView {
            program_id: &KAMINO_LEND_PROGRAM_ID,
            accounts: obligation_accounts_slice,
            data: &REFRESH_OBLIGATION_DISCRIMINATOR,
        };

        invoke_signed_with_bounds::<MAX_REFRESH_OBLIGATION_ACCOUNTS>(
            &instruction,
            obligation_account_infos,
            signer_seeds,
        )?;

        let accounts = [
            InstructionAccount::writable_signer(ctx.owner.address()),
            InstructionAccount::writable(ctx.obligation.address()),
            InstructionAccount::readonly(ctx.lending_market.address()),
            InstructionAccount::readonly(ctx.lending_market_authority.address()),
            InstructionAccount::writable(ctx.withdraw_reserve.address()),
            InstructionAccount::readonly(ctx.reserve_liquidity_mint.address()),
            InstructionAccount::writable(ctx.reserve_source_collateral.address()),
            InstructionAccount::writable(ctx.reserve_collateral_mint.address()),
            InstructionAccount::writable(ctx.reserve_liquidity_supply.address()),
            InstructionAccount::writable(ctx.user_destination_liquidity.address()),
            InstructionAccount::readonly(ctx.placeholder_user_destination_collateral.address()),
            InstructionAccount::readonly(ctx.token_program.address()),
            InstructionAccount::readonly(ctx.liquidity_token_program.address()),
            InstructionAccount::readonly(ctx.instruction_sysvar_account.address()),
            InstructionAccount::writable(ctx.obligation_farm_user_state.address()),
            InstructionAccount::writable(ctx.reserve_farm_state.address()),
            InstructionAccount::readonly(ctx.farms_program.address()),
        ];

        let account_infos = [
            ctx.owner,
            ctx.obligation,
            ctx.lending_market,
            ctx.lending_market_authority,
            ctx.withdraw_reserve,
            ctx.reserve_liquidity_mint,
            ctx.reserve_source_collateral,
            ctx.reserve_collateral_mint,
            ctx.reserve_liquidity_supply,
            ctx.user_destination_liquidity,
            ctx.placeholder_user_destination_collateral,
            ctx.token_program,
            ctx.liquidity_token_program,
            ctx.instruction_sysvar_account,
            ctx.obligation_farm_user_state,
            ctx.reserve_farm_state,
            ctx.farms_program,
        ];

        let mut instruction_data = [0u8; 16];
        instruction_data[..8].copy_from_slice(
            &WITHDRAW_OBLIGATION_COLLATERAL_AND_REDEEM_RESERVE_COLLATERAL_V2_DISCRIMINATOR,
        );
        instruction_data[8..].copy_from_slice(&amount.to_le_bytes());

        let instruction = InstructionView {
            program_id: &KAMINO_LEND_PROGRAM_ID,
            accounts: &accounts,
            data: &instruction_data,
        };

        invoke_signed_with_bounds::<17>(&instruction, &account_infos, signer_seeds)
    }

    fn withdraw(
        ctx: &KaminoWithdrawAccounts<'info>,
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
    fn parse_kamino_withdraw_data_counts() {
        let data = KaminoWithdrawData::try_from(&[2, 3, 1, 1][..]).unwrap();
        assert_eq!(data.refresh_reserve_group_count, 2);
        assert_eq!(data.deposit_reserve_count, 3);
        assert_eq!(data.borrow_reserve_count, 1);
        assert_eq!(data.borrow_referrer_token_state_count, 1);
    }

    #[test]
    fn reject_invalid_kamino_withdraw_counts() {
        assert!(KaminoWithdrawData::try_from(&[0, 0, 1, 2][..]).is_err());
    }
}
