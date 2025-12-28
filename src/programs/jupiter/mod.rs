use core::mem::MaybeUninit;

use pinocchio::{
    ProgramResult,
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Signer},
    program_error::ProgramError,
};

use crate::{Deposit, Withdraw};

// Jupiter Earn program ID: jup3YeL8QhtSx1e253b2FDvsMNC87fDrgQZivbrndc9
pub const JUPITER_EARN_PROGRAM_ID: [u8; 32] = [
    131, 233, 98, 217, 161, 25, 94, 125, 132, 113, 90, 89, 134, 24, 60, 117, 252, 181, 60, 164, 17,
    131, 167, 203, 226, 180, 132, 24, 149, 204, 24, 194,
];
pub const DEPOSIT_DISCRIMINATOR: [u8; 8] = [242, 35, 198, 137, 82, 225, 242, 182];
pub const WITHDRAW_DISCRIMINATOR: [u8; 8] = [183, 18, 70, 156, 148, 109, 161, 34];

/// Jupiter earn protocol integration
pub struct JupiterEarn;

/// Account context for JupiterEarn's deposit instruction.
pub struct JupiterEarnDepositAccounts<'info> {
    /// Target lending program
    pub lending_program: &'info AccountInfo,
    /// User signer (mutable, signer)
    pub signer: &'info AccountInfo,
    /// User's token account to deposit from (mutable)
    pub depositor_token_account: &'info AccountInfo,
    /// Recipient's token account to receive fTokens (mutable)
    pub recipient_token_account: &'info AccountInfo,
    /// Token mint being deposited
    pub mint: &'info AccountInfo,
    /// Lending admin account (readonly)
    pub lending_admin: &'info AccountInfo,
    /// Lending account (mutable)
    pub lending: &'info AccountInfo,
    /// fToken mint (mutable)
    pub f_token_mint: &'info AccountInfo,
    /// Supply token reserves liquidity (mutable)
    pub supply_token_reserves_liquidity: &'info AccountInfo,
    /// Lending supply position on liquidity (mutable)
    pub lending_supply_position_on_liquidity: &'info AccountInfo,
    /// Rate model (readonly)
    pub rate_model: &'info AccountInfo,
    /// Vault (mutable)
    pub vault: &'info AccountInfo,
    /// Liquidity (mutable)
    pub liquidity: &'info AccountInfo,
    /// Liquidity program (mutable)
    pub liquidity_program: &'info AccountInfo,
    /// Rewards rate model (readonly)
    pub rewards_rate_model: &'info AccountInfo,
    /// Token program
    pub token_program: &'info AccountInfo,
    /// Associated token program
    pub associated_token_program: &'info AccountInfo,
    /// System program
    pub system_program: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for JupiterEarnDepositAccounts<'info> {
    type Error = ProgramError;

    /// Converts a slice of `AccountInfo` into validated `JupiterEarnDepositAccounts`.
    ///
    /// # Arguments
    /// * `accounts` - Slice containing at least 18 accounts in the correct order
    ///
    /// # Returns
    /// * `Ok(JupiterEarnDepositAccounts)` - Successfully parsed account context
    /// * `Err(ProgramError::NotEnoughAccountKeys)` - Fewer than 18 accounts provided
    ///
    /// # Notes
    /// * No upper bound is enforced - extra accounts are ignored (useful for `remaining_accounts`)
    /// * Mutability and signer constraints are NOT validated here; Jupiter's program will
    ///   enforce them during CPI, providing clearer error messages
    /// * The `..` pattern allows passing more than 18 accounts without error
    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        // Require minimum of 18 accounts to prevent undefined behavior
        if accounts.len() < 18 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let [
            lending_program,
            signer,
            depositor_token_account,
            recipient_token_account,
            mint,
            lending_admin,
            lending,
            f_token_mint,
            supply_token_reserves_liquidity,
            lending_supply_position_on_liquidity,
            rate_model,
            vault,
            liquidity,
            liquidity_program,
            rewards_rate_model,
            token_program,
            associated_token_program,
            system_program,
            ..,
        ] = accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(JupiterEarnDepositAccounts {
            signer,
            depositor_token_account,
            recipient_token_account,
            mint,
            lending_admin,
            lending,
            f_token_mint,
            supply_token_reserves_liquidity,
            lending_supply_position_on_liquidity,
            rate_model,
            vault,
            liquidity,
            liquidity_program,
            rewards_rate_model,
            token_program,
            associated_token_program,
            system_program,
            lending_program,
        })
    }
}

impl<'info> Deposit<'info> for JupiterEarn {
    type Accounts = JupiterEarnDepositAccounts<'info>;

    /// Executes a deposit into Jupiter Earn protocol via CPI.
    ///
    /// This deposits liquidity tokens into Jupiter Earn and receives fTokens
    /// in return, which represent the deposited position.
    ///
    /// # Arguments
    /// * `ctx` - Account context required for the deposit (see `JupiterEarnDepositAccounts`)
    /// * `amount` - Amount of liquidity tokens to deposit
    /// * `signer_seeds` - Optional PDA signer seeds for CPI with signing
    ///
    /// # Returns
    /// * `Ok(())` - Deposit completed successfully
    /// * `Err(ProgramError)` - Invalid accounts or CPI failure
    fn deposit_signed(
        ctx: &JupiterEarnDepositAccounts<'info>,
        amount: u64,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        // Build account metas for the Jupiter Earn deposit instruction
        let accounts = [
            AccountMeta::writable_signer(ctx.signer.key()),
            AccountMeta::writable(ctx.depositor_token_account.key()),
            AccountMeta::writable(ctx.recipient_token_account.key()),
            AccountMeta::readonly(ctx.mint.key()),
            AccountMeta::readonly(ctx.lending_admin.key()),
            AccountMeta::writable(ctx.lending.key()),
            AccountMeta::writable(ctx.f_token_mint.key()),
            AccountMeta::writable(ctx.supply_token_reserves_liquidity.key()),
            AccountMeta::writable(ctx.lending_supply_position_on_liquidity.key()),
            AccountMeta::readonly(ctx.rate_model.key()),
            AccountMeta::writable(ctx.vault.key()),
            AccountMeta::writable(ctx.liquidity.key()),
            AccountMeta::writable(ctx.liquidity_program.key()),
            AccountMeta::readonly(ctx.rewards_rate_model.key()),
            AccountMeta::readonly(ctx.token_program.key()),
            AccountMeta::readonly(ctx.associated_token_program.key()),
            AccountMeta::readonly(ctx.system_program.key()),
        ];

        let account_infos = [
            ctx.signer,
            ctx.depositor_token_account,
            ctx.recipient_token_account,
            ctx.mint,
            ctx.lending_admin,
            ctx.lending,
            ctx.f_token_mint,
            ctx.supply_token_reserves_liquidity,
            ctx.lending_supply_position_on_liquidity,
            ctx.rate_model,
            ctx.vault,
            ctx.liquidity,
            ctx.liquidity_program,
            ctx.rewards_rate_model,
            ctx.token_program,
            ctx.associated_token_program,
            ctx.system_program,
        ];

        // Build instruction data: discriminator (8 bytes) + amount (8 bytes)
        let mut instruction_data = MaybeUninit::<[u8; 16]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(DEPOSIT_DISCRIMINATOR.as_ptr(), ptr, 8);
            core::ptr::copy_nonoverlapping(amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
        }

        let deposit_ix = Instruction {
            program_id: &JUPITER_EARN_PROGRAM_ID,
            accounts: &accounts,
            data: unsafe {
                core::slice::from_raw_parts(instruction_data.as_ptr() as *const u8, 16)
            },
        };

        invoke_signed(&deposit_ix, &account_infos, signer_seeds)?;

        Ok(())
    }

    fn deposit(ctx: &JupiterEarnDepositAccounts<'info>, amount: u64) -> ProgramResult {
        Self::deposit_signed(ctx, amount, &[])
    }
}

/// Account context for JupiterEarn's withdraw instruction.
pub struct JupiterEarnWithdrawAccounts<'info> {
    /// Target lending program
    pub lending_program: &'info AccountInfo,
    /// User signer (mutable, signer)
    pub signer: &'info AccountInfo,
    /// Owner's token account to receive withdrawn tokens (mutable)
    pub owner_token_account: &'info AccountInfo,
    /// Recipient's token account with fTokens to burn (mutable)
    pub recipient_token_account: &'info AccountInfo,
    /// Token mint being withdrawn
    pub mint: &'info AccountInfo,
    /// Lending admin account (readonly)
    pub lending_admin: &'info AccountInfo,
    /// Lending account (mutable)
    pub lending: &'info AccountInfo,
    /// fToken mint (mutable)
    pub f_token_mint: &'info AccountInfo,
    /// Supply token reserves liquidity (mutable)
    pub supply_token_reserves_liquidity: &'info AccountInfo,
    /// Lending supply position on liquidity (mutable)
    pub lending_supply_position_on_liquidity: &'info AccountInfo,
    /// Rate model (readonly)
    pub rate_model: &'info AccountInfo,
    /// Vault (mutable)
    pub vault: &'info AccountInfo,
    /// Claim account (mutable)
    pub claim_account: &'info AccountInfo,
    /// Liquidity (mutable)
    pub liquidity: &'info AccountInfo,
    /// Liquidity program (mutable)
    pub liquidity_program: &'info AccountInfo,
    /// Rewards rate model (readonly)
    pub rewards_rate_model: &'info AccountInfo,
    /// Token program
    pub token_program: &'info AccountInfo,
    /// Associated token program
    pub associated_token_program: &'info AccountInfo,
    /// System program
    pub system_program: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for JupiterEarnWithdrawAccounts<'info> {
    type Error = ProgramError;

    /// Converts a slice of `AccountInfo` into validated `JupiterEarnWithdrawAccounts`.
    ///
    /// # Arguments
    /// * `accounts` - Slice containing at least 19 accounts in the correct order
    ///
    /// # Returns
    /// * `Ok(JupiterEarnWithdrawAccounts)` - Successfully parsed account context
    /// * `Err(ProgramError::NotEnoughAccountKeys)` - Fewer than 19 accounts provided
    ///
    /// # Notes
    /// * No upper bound is enforced - extra accounts are ignored (useful for `remaining_accounts`)
    /// * Mutability and signer constraints are NOT validated here; Jupiter's program will
    ///   enforce them during CPI, providing clearer error messages
    /// * The `..` pattern allows passing more than 19 accounts without error
    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        // Require minimum of 19 accounts to prevent undefined behavior
        if accounts.len() < 19 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let [
            lending_program,
            signer,
            owner_token_account,
            recipient_token_account,
            mint,
            lending_admin,
            lending,
            f_token_mint,
            supply_token_reserves_liquidity,
            lending_supply_position_on_liquidity,
            rate_model,
            vault,
            claim_account,
            liquidity,
            liquidity_program,
            rewards_rate_model,
            token_program,
            associated_token_program,
            system_program,
            ..,
        ] = accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(JupiterEarnWithdrawAccounts {
            lending_program,
            signer,
            owner_token_account,
            recipient_token_account,
            mint,
            lending_admin,
            lending,
            f_token_mint,
            supply_token_reserves_liquidity,
            lending_supply_position_on_liquidity,
            rate_model,
            vault,
            claim_account,
            liquidity,
            liquidity_program,
            rewards_rate_model,
            token_program,
            associated_token_program,
            system_program,
        })
    }
}

impl<'info> Withdraw<'info> for JupiterEarn {
    type Accounts = JupiterEarnWithdrawAccounts<'info>;

    /// Executes a withdraw from Jupiter Earn protocol via CPI.
    ///
    /// This withdraws liquidity tokens from Jupiter Earn by burning fTokens,
    /// which represent the deposited position.
    ///
    /// # Arguments
    /// * `ctx` - Account context required for the withdraw (see `JupiterEarnWithdrawAccounts`)
    /// * `collateral_amount` - Amount of collateral tokens to withdraw
    /// * `signer_seeds` - Optional PDA signer seeds for CPI with signing
    ///
    /// # Returns
    /// * `Ok(())` - Withdraw completed successfully
    /// * `Err(ProgramError)` - Invalid accounts or CPI failure
    fn withdraw_signed(
        ctx: &JupiterEarnWithdrawAccounts<'info>,
        collateral_amount: u64,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        // Build account metas for the Jupiter Earn withdraw instruction
        let accounts = [
            AccountMeta::writable_signer(ctx.signer.key()),
            AccountMeta::writable(ctx.owner_token_account.key()),
            AccountMeta::writable(ctx.recipient_token_account.key()),
            AccountMeta::readonly(ctx.mint.key()),
            AccountMeta::readonly(ctx.lending_admin.key()),
            AccountMeta::writable(ctx.lending.key()),
            AccountMeta::writable(ctx.f_token_mint.key()),
            AccountMeta::writable(ctx.supply_token_reserves_liquidity.key()),
            AccountMeta::writable(ctx.lending_supply_position_on_liquidity.key()),
            AccountMeta::readonly(ctx.rate_model.key()),
            AccountMeta::writable(ctx.vault.key()),
            AccountMeta::writable(ctx.claim_account.key()),
            AccountMeta::writable(ctx.liquidity.key()),
            AccountMeta::writable(ctx.liquidity_program.key()),
            AccountMeta::readonly(ctx.rewards_rate_model.key()),
            AccountMeta::readonly(ctx.token_program.key()),
            AccountMeta::readonly(ctx.associated_token_program.key()),
            AccountMeta::readonly(ctx.system_program.key()),
        ];

        let account_infos = [
            ctx.signer,
            ctx.owner_token_account,
            ctx.recipient_token_account,
            ctx.mint,
            ctx.lending_admin,
            ctx.lending,
            ctx.f_token_mint,
            ctx.supply_token_reserves_liquidity,
            ctx.lending_supply_position_on_liquidity,
            ctx.rate_model,
            ctx.vault,
            ctx.claim_account,
            ctx.liquidity,
            ctx.liquidity_program,
            ctx.rewards_rate_model,
            ctx.token_program,
            ctx.associated_token_program,
            ctx.system_program,
        ];

        // Build instruction data: discriminator (8 bytes) + collateral_amount (8 bytes)
        let mut instruction_data = MaybeUninit::<[u8; 16]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(WITHDRAW_DISCRIMINATOR.as_ptr(), ptr, 8);
            core::ptr::copy_nonoverlapping(collateral_amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
        }

        let withdraw_ix = Instruction {
            program_id: &JUPITER_EARN_PROGRAM_ID,
            accounts: &accounts,
            data: unsafe {
                core::slice::from_raw_parts(instruction_data.as_ptr() as *const u8, 16)
            },
        };

        invoke_signed(&withdraw_ix, &account_infos, signer_seeds)?;

        Ok(())
    }

    fn withdraw(ctx: &JupiterEarnWithdrawAccounts<'info>, collateral_amount: u64) -> ProgramResult {
        Self::withdraw_signed(ctx, collateral_amount, &[])
    }
}
