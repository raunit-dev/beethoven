use pinocchio::{instruction::Signer, program_error::ProgramError, account_info::AccountInfo, ProgramResult};

/// Core trait for withdraw operations across different protocols (Kamino, Jupiter, etc.)
///
/// Each protocol implements this trait with its specific account requirements and CPI logic.
/// The trait provides both signed and unsigned withdraw methods.
pub trait Withdraw<'info> {
    /// Protocol-specific accounts required for the withdraw CPI
    type Accounts;

    /// Execute a withdraw with PDA signing capability
    ///
    /// # Arguments
    /// * `ctx` - Protocol-specific account context
    /// * `amount` - Amount to withdraw
    /// * `signer_seeds` - Seeds for PDA signing
    fn withdraw_signed(ctx: &Self::Accounts, amount: u64, signer_seeds: &[Signer]) -> ProgramResult;

    /// Execute a withdraw without signing (user is direct signer)
    ///
    /// # Arguments
    /// * `ctx` - Protocol-specific account context
    /// * `amount` - Amount to withdraw
    fn withdraw(ctx: &Self::Accounts, amount: u64) -> ProgramResult;
}

/// Typed context for withdraw operations, discriminated by protocol.
///
/// This enum contains the protocol-specific account structures after parsing
/// and discrimination. Users can pattern match on this to perform custom
/// validation before executing the withdraw.
pub enum WithdrawContext<'info> {
    #[cfg(feature = "jupiter")]
    Jupiter(crate::programs::jupiter::JupiterEarnWithdrawAccounts<'info>),
}

impl<'info> Withdraw<'info> for WithdrawContext<'info> {
    type Accounts = Self;

    fn withdraw_signed(ctx: &Self::Accounts, amount: u64, signer_seeds: &[Signer]) -> ProgramResult {
        match ctx {
            #[cfg(feature = "jupiter")]
            WithdrawContext::Jupiter(jupiter_ctx) => {
                crate::programs::jupiter::JupiterEarn::withdraw_signed(jupiter_ctx, amount, signer_seeds)
            }
        }
    }

    fn withdraw(ctx: &Self::Accounts, amount: u64) -> ProgramResult {
        Self::withdraw_signed(ctx, amount, &[])
    }
}

/// Parses accounts and discriminates the protocol based on the first account's owner.
///
/// This function returns a typed `WithdrawContext` that allows users to:
/// - Pattern match on the protocol type
/// - Access typed account fields for custom validation
/// - Inspect account properties before executing the withdraw
///
/// # Arguments
/// * `accounts` - Slice of accounts where the first account's owner determines the protocol
///
/// # Returns
/// * `Ok(WithdrawContext)` - Typed context for the detected protocol
/// * `Err(ProgramError::NotEnoughAccountKeys)` - Empty account slice provided
/// * `Err(ProgramError::InvalidAccountData)` - No matching protocol found or invalid account structure
///
/// # Example
/// ```ignore
/// let ctx = try_from_withdraw_context(remaining_accounts)?;
///
/// match &ctx {
///     WithdrawContext::Jupiter(jupiter_accounts) => {
///         // Custom validation for Jupiter
///     }
/// }
///
/// // Use the trait method to execute
/// WithdrawContext::withdraw(&ctx, amount)?;
/// ```
pub fn try_from_withdraw_context<'info>(
    accounts: &'info [AccountInfo]
) -> Result<WithdrawContext<'info>, ProgramError> {
    let detector_account = accounts
        .first()
        .ok_or(ProgramError::NotEnoughAccountKeys)?;

    #[cfg(feature = "jupiter")]
    if detector_account.key().eq(&crate::programs::jupiter::JUPITER_EARN_PROGRAM_ID) {
        let ctx = crate::programs::jupiter::JupiterEarnWithdrawAccounts::try_from(accounts)?;
        return Ok(WithdrawContext::Jupiter(ctx));
    }

    Err(ProgramError::InvalidAccountData)
}

/// Convenience function: Parses accounts, discriminates protocol, and executes withdraw with PDA signing.
///
/// This is equivalent to calling `try_from_withdraw_context` followed by `WithdrawContext::withdraw_signed`.
/// For custom validation, use those functions separately instead.
///
/// # Arguments
/// * `accounts` - Slice of accounts where the first account's owner determines the protocol
/// * `amount` - Amount of tokens to withdraw
/// * `signer_seeds` - Seeds for PDA signing
///
/// # Returns
/// * `Ok(())` - Withdraw executed successfully
/// * `Err(ProgramError)` - Parsing, discrimination, or CPI failed
pub fn withdraw_signed(
    accounts: &[AccountInfo],
    amount: u64,
    signer_seeds: &[Signer]
) -> ProgramResult {
    let ctx = try_from_withdraw_context(accounts)?;
    WithdrawContext::withdraw_signed(&ctx, amount, signer_seeds)
}

/// Convenience function: Parses accounts, discriminates protocol, and executes withdraw.
///
/// This is equivalent to calling `try_from_withdraw_context` followed by `WithdrawContext::withdraw`.
/// For custom validation, use those functions separately instead.
///
/// # Arguments
/// * `accounts` - Slice of accounts where the first account's owner determines the protocol
/// * `amount` - Amount of tokens to withdraw
///
/// # Returns
/// * `Ok(())` - Withdraw executed successfully
/// * `Err(ProgramError)` - Parsing, discrimination, or CPI failed
pub fn withdraw(
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    withdraw_signed(accounts, amount, &[])
}
