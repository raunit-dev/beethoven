use {solana_address::Address, solana_instruction::AccountMeta};

pub const MARGINFI_PROGRAM_ID: Address =
    Address::from_str_const("MFv2hWf31Z9kbCa1snEPYctwafyhdvnV7FZnsebVacA");

const BANK_ACCOUNT_LEN: usize = 8 + 1856;
const BANK_OFFSET_MINT: usize = 8;
const BANK_OFFSET_GROUP: usize = 8 + 32 + 1;
const BANK_OFFSET_LIQUIDITY_VAULT: usize = 8 + 104;

const MARGINFI_ACCOUNT_LEN: usize = 8 + 2304;
const MARGINFI_ACCOUNT_OFFSET_GROUP: usize = 8;
const MARGINFI_ACCOUNT_OFFSET_AUTHORITY: usize = 8 + 32;

/// Pre-resolved addresses for building a marginfi deposit instruction offline.
pub struct MarginfiDepositInput {
    pub user: Address,
    pub group: Address,
    pub marginfi_account: Address,
    pub bank: Address,
    pub signer_token_account: Address,
    pub liquidity_vault: Address,
    pub token_program: Address,
    pub remaining_accounts: Vec<AccountMeta>,
}

/// Build marginfi deposit AccountMeta list from pre-resolved addresses.
pub fn build_accounts(input: &MarginfiDepositInput) -> Vec<AccountMeta> {
    let mut accounts = vec![
        AccountMeta::new_readonly(MARGINFI_PROGRAM_ID, false),
        AccountMeta::new_readonly(input.group, false),
        AccountMeta::new(input.marginfi_account, false),
        AccountMeta::new_readonly(input.user, true),
        AccountMeta::new(input.bank, false),
        AccountMeta::new(input.signer_token_account, false),
        AccountMeta::new(input.liquidity_vault, false),
        AccountMeta::new_readonly(input.token_program, false),
    ];
    accounts.extend(input.remaining_accounts.iter().cloned());
    accounts
}

/// Build marginfi extra data: Anchor/Borsh `Option<bool>`.
pub fn build_extra_data(deposit_up_to_limit: Option<bool>) -> Vec<u8> {
    match deposit_up_to_limit {
        None => vec![0, 0],
        Some(v) => vec![1, v as u8],
    }
}

fn read_bank_fields(data: &[u8]) -> Result<(Address, Address, Address), crate::error::ClientError> {
    if data.len() < BANK_ACCOUNT_LEN {
        return Err(crate::error::ClientError::InvalidAccountData(format!(
            "marginfi bank data too short: {}",
            data.len()
        )));
    }

    Ok((
        crate::read_pubkey(data, BANK_OFFSET_MINT)?,
        crate::read_pubkey(data, BANK_OFFSET_GROUP)?,
        crate::read_pubkey(data, BANK_OFFSET_LIQUIDITY_VAULT)?,
    ))
}

fn read_marginfi_account_fields(
    data: &[u8],
) -> Result<(Address, Address), crate::error::ClientError> {
    if data.len() < MARGINFI_ACCOUNT_LEN {
        return Err(crate::error::ClientError::InvalidAccountData(format!(
            "marginfi account data too short: {}",
            data.len()
        )));
    }

    Ok((
        crate::read_pubkey(data, MARGINFI_ACCOUNT_OFFSET_GROUP)?,
        crate::read_pubkey(data, MARGINFI_ACCOUNT_OFFSET_AUTHORITY)?,
    ))
}

#[cfg(feature = "resolve")]
fn read_token_account_mint_and_owner(
    data: &[u8],
) -> Result<(Address, Address), crate::error::ClientError> {
    if data.len() < 64 {
        return Err(crate::error::ClientError::InvalidAccountData(
            "Token account data too short".to_string(),
        ));
    }

    let mint = Address::from(<[u8; 32]>::try_from(&data[0..32]).unwrap());
    let owner = Address::from(<[u8; 32]>::try_from(&data[32..64]).unwrap());
    Ok((mint, owner))
}

/// Resolve accounts and data for a marginfi deposit via RPC.
#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &solana_rpc_client::nonblocking::rpc_client::RpcClient,
    bank: &Address,
    marginfi_account: &Address,
    signer_token_account: &Address,
    deposit_up_to_limit: Option<bool>,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), crate::error::ClientError> {
    let bank_account = rpc.get_account(bank).await?;
    if bank_account.owner != MARGINFI_PROGRAM_ID {
        return Err(crate::error::ClientError::InvalidAccountData(format!(
            "marginfi bank {} is not owned by the marginfi program",
            bank
        )));
    }

    let (bank_mint, group, liquidity_vault) = read_bank_fields(&bank_account.data)?;

    let marginfi_account_account = rpc.get_account(marginfi_account).await?;
    if marginfi_account_account.owner != MARGINFI_PROGRAM_ID {
        return Err(crate::error::ClientError::InvalidAccountData(format!(
            "marginfi account {} is not owned by the marginfi program",
            marginfi_account
        )));
    }
    let (marginfi_account_group, authority) =
        read_marginfi_account_fields(&marginfi_account_account.data)?;
    if marginfi_account_group != group {
        return Err(crate::error::ClientError::InvalidAccountData(format!(
            "marginfi account {} group {} does not match bank group {}",
            marginfi_account, marginfi_account_group, group
        )));
    }

    if authority != *user {
        return Err(crate::error::ClientError::InvalidAccountData(format!(
            "marginfi account {} authority {} does not match user {}",
            marginfi_account, authority, user
        )));
    }

    let signer_token_account_account = rpc.get_account(signer_token_account).await?;
    let token_program = crate::get_token_program_for_mint(rpc, &bank_mint).await?;
    if signer_token_account_account.owner != token_program {
        return Err(crate::error::ClientError::InvalidAccountData(format!(
            "token account {} is owned by {}, expected {}",
            signer_token_account, signer_token_account_account.owner, token_program
        )));
    }

    let (token_account_mint, token_account_owner) =
        read_token_account_mint_and_owner(&signer_token_account_account.data)?;
    if token_account_owner != *user {
        return Err(crate::error::ClientError::InvalidAccountData(format!(
            "token account {} owner {} does not match user {}",
            signer_token_account, token_account_owner, user
        )));
    }

    if token_account_mint != bank_mint {
        return Err(crate::error::ClientError::MintMismatch {
            expected: bank_mint.to_string(),
            got: token_account_mint.to_string(),
        });
    }

    let remaining_accounts = if token_program == crate::TOKEN_2022_PROGRAM_ID {
        vec![AccountMeta::new_readonly(bank_mint, false)]
    } else {
        vec![]
    };

    let input = MarginfiDepositInput {
        user: *user,
        group,
        marginfi_account: *marginfi_account,
        bank: *bank,
        signer_token_account: *signer_token_account,
        liquidity_vault,
        token_program,
        remaining_accounts,
    };

    Ok((
        build_accounts(&input),
        build_extra_data(deposit_up_to_limit),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put_pubkey(data: &mut [u8], offset: usize, address: Address) {
        data[offset..offset + 32].copy_from_slice(address.as_ref());
    }

    #[test]
    fn build_extra_data_matches_anchor_option_bool_shape() {
        assert_eq!(build_extra_data(None), vec![0, 0]);
        assert_eq!(build_extra_data(Some(false)), vec![1, 0]);
        assert_eq!(build_extra_data(Some(true)), vec![1, 1]);
    }

    #[test]
    fn build_accounts_appends_remaining_accounts() {
        let input = MarginfiDepositInput {
            user: Address::new_from_array([1; 32]),
            group: Address::new_from_array([2; 32]),
            marginfi_account: Address::new_from_array([3; 32]),
            bank: Address::new_from_array([4; 32]),
            signer_token_account: Address::new_from_array([5; 32]),
            liquidity_vault: Address::new_from_array([6; 32]),
            token_program: Address::new_from_array([7; 32]),
            remaining_accounts: vec![AccountMeta::new_readonly(
                Address::new_from_array([8; 32]),
                false,
            )],
        };

        let accounts = build_accounts(&input);

        assert_eq!(accounts.len(), 9);
        assert_eq!(accounts[0].pubkey, MARGINFI_PROGRAM_ID);
        assert_eq!(accounts[1].pubkey, input.group);
        assert_eq!(accounts[2].pubkey, input.marginfi_account);
        assert_eq!(accounts[3].pubkey, input.user);
        assert!(accounts[3].is_signer);
        assert_eq!(accounts[8].pubkey, Address::new_from_array([8; 32]));
    }

    #[test]
    fn read_bank_fields_use_verified_offsets() {
        let mint = Address::new_from_array([11; 32]);
        let group = Address::new_from_array([12; 32]);
        let liquidity_vault = Address::new_from_array([13; 32]);
        let mut data = vec![0u8; BANK_ACCOUNT_LEN];

        put_pubkey(&mut data, BANK_OFFSET_MINT, mint);
        put_pubkey(&mut data, BANK_OFFSET_GROUP, group);
        put_pubkey(&mut data, BANK_OFFSET_LIQUIDITY_VAULT, liquidity_vault);

        let parsed = read_bank_fields(&data).unwrap();
        assert_eq!(parsed, (mint, group, liquidity_vault));
    }

    #[test]
    fn read_marginfi_account_fields_use_verified_offsets() {
        let group = Address::new_from_array([21; 32]);
        let authority = Address::new_from_array([22; 32]);
        let mut data = vec![0u8; MARGINFI_ACCOUNT_LEN];

        put_pubkey(&mut data, MARGINFI_ACCOUNT_OFFSET_GROUP, group);
        put_pubkey(&mut data, MARGINFI_ACCOUNT_OFFSET_AUTHORITY, authority);

        let parsed = read_marginfi_account_fields(&data).unwrap();
        assert_eq!(parsed, (group, authority));
    }
}
