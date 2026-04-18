use {solana_address::Address, solana_instruction::AccountMeta};

pub const MARGINFI_PROGRAM_ID: Address =
    Address::from_str_const("MFv2hWf31Z9kbCa1snEPYctwafyhdvnV7FZnsebVacA");

const LIQUIDITY_VAULT_AUTHORITY_SEED: &[u8] = b"liquidity_vault_auth";
const BANK_ACCOUNT_LEN: usize = 8 + 1856;
const BANK_OFFSET_MINT: usize = 8;
const BANK_OFFSET_GROUP: usize = 8 + 32 + 1;
const BANK_OFFSET_LIQUIDITY_VAULT: usize = 8 + 104;
const BANK_OFFSET_ORACLE_SETUP: usize = 8 + 288 + 312 + 1;
const BANK_OFFSET_ORACLE_KEYS: usize = 8 + 288 + 314;
const BANK_OFFSET_ASSET_TAG: usize = 8 + 288 + 489;
const BANK_OFFSET_INTEGRATION_ACC_1: usize = 8 + 1552;
const MAX_ORACLE_KEYS: usize = 5;

const ASSET_TAG_DEFAULT: u8 = 0;
const ASSET_TAG_SOL: u8 = 1;
const ASSET_TAG_STAKED: u8 = 2;

const ORACLE_SETUP_NONE: u8 = 0;
const ORACLE_SETUP_PYTH_PUSH: u8 = 3;
const ORACLE_SETUP_SWITCHBOARD_PULL: u8 = 4;
const ORACLE_SETUP_STAKED_WITH_PYTH_PUSH: u8 = 5;
const ORACLE_SETUP_KAMINO_PYTH_PUSH: u8 = 6;
const ORACLE_SETUP_KAMINO_SWITCHBOARD_PULL: u8 = 7;
const ORACLE_SETUP_FIXED: u8 = 8;
const ORACLE_SETUP_DRIFT_PYTH_PULL: u8 = 9;
const ORACLE_SETUP_DRIFT_SWITCHBOARD_PULL: u8 = 10;
const ORACLE_SETUP_SOLEND_PYTH_PULL: u8 = 11;
const ORACLE_SETUP_SOLEND_SWITCHBOARD_PULL: u8 = 12;
const ORACLE_SETUP_FIXED_KAMINO: u8 = 13;
const ORACLE_SETUP_FIXED_DRIFT: u8 = 14;
const ORACLE_SETUP_JUPLEND_PYTH_PULL: u8 = 15;
const ORACLE_SETUP_JUPLEND_SWITCHBOARD_PULL: u8 = 16;
const ORACLE_SETUP_FIXED_JUPLEND: u8 = 17;

const MARGINFI_ACCOUNT_LEN: usize = 8 + 2304;
const MARGINFI_ACCOUNT_OFFSET_GROUP: usize = 8;
const MARGINFI_ACCOUNT_OFFSET_AUTHORITY: usize = 8 + 32;
const MARGINFI_ACCOUNT_OFFSET_BALANCES: usize = 8 + 32 + 32;
const MAX_LENDING_ACCOUNT_BALANCES: usize = 16;
const BALANCE_LEN: usize = 104;
const BALANCE_OFFSET_ACTIVE: usize = 0;
const BALANCE_OFFSET_BANK_PK: usize = 1;

const MAX_WITHDRAW_REMAINING_ACCOUNTS: usize = 56;

pub struct MarginfiWithdrawInput {
    pub user: Address,
    pub group: Address,
    pub marginfi_account: Address,
    pub bank: Address,
    pub destination_token_account: Address,
    pub bank_liquidity_vault_authority: Address,
    pub liquidity_vault: Address,
    pub token_program: Address,
    pub remaining_accounts: Vec<AccountMeta>,
}

pub fn build_accounts(input: &MarginfiWithdrawInput) -> Vec<AccountMeta> {
    let mut accounts = vec![
        AccountMeta::new_readonly(MARGINFI_PROGRAM_ID, false),
        AccountMeta::new_readonly(input.group, false),
        AccountMeta::new(input.marginfi_account, false),
        AccountMeta::new_readonly(input.user, true),
        AccountMeta::new(input.bank, false),
        AccountMeta::new(input.destination_token_account, false),
        AccountMeta::new_readonly(input.bank_liquidity_vault_authority, false),
        AccountMeta::new(input.liquidity_vault, false),
        AccountMeta::new_readonly(input.token_program, false),
    ];
    accounts.extend(input.remaining_accounts.iter().cloned());
    accounts
}

pub fn build_extra_data(withdraw_all: Option<bool>) -> Vec<u8> {
    match withdraw_all {
        None => vec![0, 0],
        Some(v) => vec![1, v as u8],
    }
}

struct MarginfiBankFields {
    mint: Address,
    group: Address,
    liquidity_vault: Address,
    asset_tag: u8,
}

#[derive(Clone, Copy)]
struct BankHealthFields {
    oracle_setup: u8,
    oracle_keys: [Address; MAX_ORACLE_KEYS],
    integration_acc_1: Address,
}

fn read_bank_fields(data: &[u8]) -> Result<MarginfiBankFields, crate::error::ClientError> {
    if data.len() < BANK_ACCOUNT_LEN {
        return Err(crate::error::ClientError::InvalidAccountData(format!(
            "marginfi bank data too short: {}",
            data.len()
        )));
    }

    Ok(MarginfiBankFields {
        mint: crate::read_pubkey(data, BANK_OFFSET_MINT)?,
        group: crate::read_pubkey(data, BANK_OFFSET_GROUP)?,
        liquidity_vault: crate::read_pubkey(data, BANK_OFFSET_LIQUIDITY_VAULT)?,
        asset_tag: data[BANK_OFFSET_ASSET_TAG],
    })
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

fn read_active_balance_bank_keys(data: &[u8]) -> Result<Vec<Address>, crate::error::ClientError> {
    if data.len() < MARGINFI_ACCOUNT_LEN {
        return Err(crate::error::ClientError::InvalidAccountData(format!(
            "marginfi account data too short: {}",
            data.len()
        )));
    }

    let mut bank_keys = Vec::new();
    for index in 0..MAX_LENDING_ACCOUNT_BALANCES {
        let base = MARGINFI_ACCOUNT_OFFSET_BALANCES + index * BALANCE_LEN;
        if data[base + BALANCE_OFFSET_ACTIVE] == 0 {
            continue;
        }
        bank_keys.push(crate::read_pubkey(data, base + BALANCE_OFFSET_BANK_PK)?);
    }

    Ok(bank_keys)
}

fn read_bank_health_fields(data: &[u8]) -> Result<BankHealthFields, crate::error::ClientError> {
    if data.len() < BANK_ACCOUNT_LEN {
        return Err(crate::error::ClientError::InvalidAccountData(format!(
            "marginfi bank data too short: {}",
            data.len()
        )));
    }

    let mut oracle_keys = [Address::new_from_array([0; 32]); MAX_ORACLE_KEYS];
    for (index, key) in oracle_keys.iter_mut().enumerate() {
        *key = crate::read_pubkey(data, BANK_OFFSET_ORACLE_KEYS + index * 32)?;
    }

    Ok(BankHealthFields {
        oracle_setup: data[BANK_OFFSET_ORACLE_SETUP],
        oracle_keys,
        integration_acc_1: crate::read_pubkey(data, BANK_OFFSET_INTEGRATION_ACC_1)?,
    })
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

fn derive_bank_liquidity_vault_authority(bank: &Address) -> Address {
    Address::find_program_address(
        &[LIQUIDITY_VAULT_AUTHORITY_SEED, bank.as_ref()],
        &MARGINFI_PROGRAM_ID,
    )
    .0
}

fn is_native_marginfi_asset_tag(asset_tag: u8) -> bool {
    matches!(
        asset_tag,
        ASSET_TAG_DEFAULT | ASSET_TAG_SOL | ASSET_TAG_STAKED
    )
}

fn is_default_address(address: &Address) -> bool {
    *address == Address::new_from_array([0; 32])
}

fn push_required_remaining_account(
    remaining_accounts: &mut Vec<AccountMeta>,
    account: Address,
    label: &str,
    bank: &Address,
) -> Result<(), crate::error::ClientError> {
    if is_default_address(&account) {
        return Err(crate::error::ClientError::InvalidAccountData(format!(
            "marginfi bank {} is missing required {} account",
            bank, label
        )));
    }

    remaining_accounts.push(AccountMeta::new_readonly(account, false));
    Ok(())
}

fn append_bank_health_accounts(
    remaining_accounts: &mut Vec<AccountMeta>,
    bank: &Address,
    fields: &BankHealthFields,
) -> Result<(), crate::error::ClientError> {
    remaining_accounts.push(AccountMeta::new_readonly(*bank, false));

    match fields.oracle_setup {
        ORACLE_SETUP_FIXED => {}
        ORACLE_SETUP_PYTH_PUSH | ORACLE_SETUP_SWITCHBOARD_PULL => {
            push_required_remaining_account(
                remaining_accounts,
                fields.oracle_keys[0],
                "primary oracle",
                bank,
            )?;
        }
        ORACLE_SETUP_STAKED_WITH_PYTH_PUSH => {
            push_required_remaining_account(
                remaining_accounts,
                fields.oracle_keys[0],
                "primary oracle",
                bank,
            )?;
            push_required_remaining_account(
                remaining_accounts,
                fields.oracle_keys[1],
                "staked lst mint",
                bank,
            )?;
            push_required_remaining_account(
                remaining_accounts,
                fields.oracle_keys[2],
                "staked pool",
                bank,
            )?;
        }
        ORACLE_SETUP_KAMINO_PYTH_PUSH
        | ORACLE_SETUP_KAMINO_SWITCHBOARD_PULL
        | ORACLE_SETUP_DRIFT_PYTH_PULL
        | ORACLE_SETUP_DRIFT_SWITCHBOARD_PULL
        | ORACLE_SETUP_SOLEND_PYTH_PULL
        | ORACLE_SETUP_SOLEND_SWITCHBOARD_PULL
        | ORACLE_SETUP_JUPLEND_PYTH_PULL
        | ORACLE_SETUP_JUPLEND_SWITCHBOARD_PULL => {
            push_required_remaining_account(
                remaining_accounts,
                fields.oracle_keys[0],
                "primary oracle",
                bank,
            )?;
            push_required_remaining_account(
                remaining_accounts,
                fields.integration_acc_1,
                "integration account",
                bank,
            )?;
        }
        ORACLE_SETUP_FIXED_KAMINO | ORACLE_SETUP_FIXED_DRIFT | ORACLE_SETUP_FIXED_JUPLEND => {
            push_required_remaining_account(
                remaining_accounts,
                fields.integration_acc_1,
                "integration account",
                bank,
            )?;
        }
        ORACLE_SETUP_NONE => {
            return Err(crate::error::ClientError::InvalidAccountData(format!(
                "marginfi bank {} has no oracle setup",
                bank
            )));
        }
        deprecated_or_unknown => {
            return Err(crate::error::ClientError::InvalidAccountData(format!(
                "marginfi bank {} uses unsupported oracle setup {}",
                bank, deprecated_or_unknown
            )));
        }
    }

    Ok(())
}

#[cfg(feature = "resolve")]
async fn resolve_health_check_remaining_accounts(
    rpc: &solana_rpc_client::nonblocking::rpc_client::RpcClient,
    marginfi_account_data: &[u8],
) -> Result<Vec<AccountMeta>, crate::error::ClientError> {
    let active_bank_keys = read_active_balance_bank_keys(marginfi_account_data)?;
    let bank_accounts = rpc.get_multiple_accounts(&active_bank_keys).await?;

    let mut remaining_accounts = Vec::new();
    for (bank_key, maybe_bank_account) in active_bank_keys.iter().zip(bank_accounts) {
        let bank_account = maybe_bank_account.ok_or_else(|| {
            crate::error::ClientError::AccountNotFound(format!("marginfi bank {}", bank_key))
        })?;

        if bank_account.owner != MARGINFI_PROGRAM_ID {
            return Err(crate::error::ClientError::InvalidAccountData(format!(
                "marginfi bank {} is not owned by the marginfi program",
                bank_key
            )));
        }

        let fields = read_bank_health_fields(&bank_account.data)?;
        append_bank_health_accounts(&mut remaining_accounts, bank_key, &fields)?;
    }

    Ok(remaining_accounts)
}

#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &solana_rpc_client::nonblocking::rpc_client::RpcClient,
    bank: &Address,
    marginfi_account: &Address,
    destination_token_account: &Address,
    withdraw_all: Option<bool>,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), crate::error::ClientError> {
    let bank_account = rpc.get_account(bank).await?;
    if bank_account.owner != MARGINFI_PROGRAM_ID {
        return Err(crate::error::ClientError::InvalidAccountData(format!(
            "marginfi bank {} is not owned by the marginfi program",
            bank
        )));
    }

    let bank_fields = read_bank_fields(&bank_account.data)?;
    if !is_native_marginfi_asset_tag(bank_fields.asset_tag) {
        return Err(crate::error::ClientError::InvalidAccountData(format!(
            "marginfi withdraw only supports native marginfi banks, got asset tag {} for bank {}",
            bank_fields.asset_tag, bank
        )));
    }

    let marginfi_account_account = rpc.get_account(marginfi_account).await?;
    if marginfi_account_account.owner != MARGINFI_PROGRAM_ID {
        return Err(crate::error::ClientError::InvalidAccountData(format!(
            "marginfi account {} is not owned by the marginfi program",
            marginfi_account
        )));
    }
    let (marginfi_account_group, authority) =
        read_marginfi_account_fields(&marginfi_account_account.data)?;
    if marginfi_account_group != bank_fields.group {
        return Err(crate::error::ClientError::InvalidAccountData(format!(
            "marginfi account {} group {} does not match bank group {}",
            marginfi_account, marginfi_account_group, bank_fields.group
        )));
    }

    if authority != *user {
        return Err(crate::error::ClientError::InvalidAccountData(format!(
            "marginfi account {} authority {} does not match user {}",
            marginfi_account, authority, user
        )));
    }

    let destination_token_account_account = rpc.get_account(destination_token_account).await?;
    let token_program = crate::get_token_program_for_mint(rpc, &bank_fields.mint).await?;
    if destination_token_account_account.owner != token_program {
        return Err(crate::error::ClientError::InvalidAccountData(format!(
            "token account {} is owned by {}, expected {}",
            destination_token_account, destination_token_account_account.owner, token_program
        )));
    }

    let (token_account_mint, token_account_owner) =
        read_token_account_mint_and_owner(&destination_token_account_account.data)?;
    if token_account_owner != *user {
        return Err(crate::error::ClientError::InvalidAccountData(format!(
            "token account {} owner {} does not match user {}",
            destination_token_account, token_account_owner, user
        )));
    }

    if token_account_mint != bank_fields.mint {
        return Err(crate::error::ClientError::MintMismatch {
            expected: bank_fields.mint.to_string(),
            got: token_account_mint.to_string(),
        });
    }

    let active_bank_keys = read_active_balance_bank_keys(&marginfi_account_account.data)?;
    if !active_bank_keys
        .iter()
        .any(|active_bank| active_bank == bank)
    {
        return Err(crate::error::ClientError::InvalidAccountData(format!(
            "marginfi account {} has no active balance for bank {}",
            marginfi_account, bank
        )));
    }

    let mut remaining_accounts =
        resolve_health_check_remaining_accounts(rpc, &marginfi_account_account.data).await?;
    if token_program == crate::TOKEN_2022_PROGRAM_ID {
        remaining_accounts.insert(0, AccountMeta::new_readonly(bank_fields.mint, false));
    }

    if remaining_accounts.len() > MAX_WITHDRAW_REMAINING_ACCOUNTS {
        return Err(crate::error::ClientError::InvalidAccountData(format!(
            "marginfi withdraw requires {} trailing remaining accounts, but the current Beethoven adapter supports at most {}",
            remaining_accounts.len(),
            MAX_WITHDRAW_REMAINING_ACCOUNTS
        )));
    }

    let input = MarginfiWithdrawInput {
        user: *user,
        group: bank_fields.group,
        marginfi_account: *marginfi_account,
        bank: *bank,
        destination_token_account: *destination_token_account,
        bank_liquidity_vault_authority: derive_bank_liquidity_vault_authority(bank),
        liquidity_vault: bank_fields.liquidity_vault,
        token_program,
        remaining_accounts,
    };

    Ok((build_accounts(&input), build_extra_data(withdraw_all)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put_pubkey(data: &mut [u8], offset: usize, address: Address) {
        data[offset..offset + 32].copy_from_slice(address.as_ref());
    }

    #[test]
    fn build_extra_data_matches_option_bool_encoding() {
        assert_eq!(build_extra_data(None), vec![0, 0]);
        assert_eq!(build_extra_data(Some(false)), vec![1, 0]);
        assert_eq!(build_extra_data(Some(true)), vec![1, 1]);
    }

    #[test]
    fn build_accounts_keeps_program_first_and_preserves_tail() {
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

    #[test]
    fn read_active_balance_bank_keys_uses_verified_offsets() {
        let bank_a = Address::new_from_array([11; 32]);
        let bank_b = Address::new_from_array([12; 32]);
        let mut data = vec![0u8; MARGINFI_ACCOUNT_LEN];

        data[MARGINFI_ACCOUNT_OFFSET_BALANCES + BALANCE_OFFSET_ACTIVE] = 1;
        put_pubkey(
            &mut data,
            MARGINFI_ACCOUNT_OFFSET_BALANCES + BALANCE_OFFSET_BANK_PK,
            bank_a,
        );

        let second_balance_offset = MARGINFI_ACCOUNT_OFFSET_BALANCES + BALANCE_LEN;
        data[second_balance_offset + BALANCE_OFFSET_ACTIVE] = 1;
        put_pubkey(
            &mut data,
            second_balance_offset + BALANCE_OFFSET_BANK_PK,
            bank_b,
        );

        let parsed = read_active_balance_bank_keys(&data).unwrap();
        assert_eq!(parsed, vec![bank_a, bank_b]);
    }

    #[test]
    fn read_bank_fields_and_health_fields_use_verified_offsets() {
        let mint = Address::new_from_array([21; 32]);
        let group = Address::new_from_array([22; 32]);
        let liquidity_vault = Address::new_from_array([23; 32]);
        let oracle = Address::new_from_array([24; 32]);
        let reserve = Address::new_from_array([25; 32]);
        let mut data = vec![0u8; BANK_ACCOUNT_LEN];

        put_pubkey(&mut data, BANK_OFFSET_MINT, mint);
        put_pubkey(&mut data, BANK_OFFSET_GROUP, group);
        put_pubkey(&mut data, BANK_OFFSET_LIQUIDITY_VAULT, liquidity_vault);
        data[BANK_OFFSET_ASSET_TAG] = ASSET_TAG_DEFAULT;
        data[BANK_OFFSET_ORACLE_SETUP] = ORACLE_SETUP_KAMINO_PYTH_PUSH;
        put_pubkey(&mut data, BANK_OFFSET_ORACLE_KEYS, oracle);
        put_pubkey(&mut data, BANK_OFFSET_INTEGRATION_ACC_1, reserve);

        let fixed_fields = read_bank_fields(&data).unwrap();
        assert_eq!(fixed_fields.mint, mint);
        assert_eq!(fixed_fields.group, group);
        assert_eq!(fixed_fields.liquidity_vault, liquidity_vault);
        assert_eq!(fixed_fields.asset_tag, ASSET_TAG_DEFAULT);

        let health_fields = read_bank_health_fields(&data).unwrap();
        assert_eq!(health_fields.oracle_setup, ORACLE_SETUP_KAMINO_PYTH_PUSH);
        assert_eq!(health_fields.oracle_keys[0], oracle);
        assert_eq!(health_fields.integration_acc_1, reserve);
    }

    #[test]
    fn append_bank_health_accounts_matches_upstream_grouping() {
        let native_bank = Address::new_from_array([31; 32]);
        let native_oracle = Address::new_from_array([32; 32]);
        let kamino_bank = Address::new_from_array([33; 32]);
        let kamino_oracle = Address::new_from_array([34; 32]);
        let kamino_reserve = Address::new_from_array([35; 32]);

        let native_fields = BankHealthFields {
            oracle_setup: ORACLE_SETUP_PYTH_PUSH,
            oracle_keys: [
                native_oracle,
                Address::new_from_array([0; 32]),
                Address::new_from_array([0; 32]),
                Address::new_from_array([0; 32]),
                Address::new_from_array([0; 32]),
            ],
            integration_acc_1: Address::new_from_array([0; 32]),
        };
        let kamino_fields = BankHealthFields {
            oracle_setup: ORACLE_SETUP_KAMINO_PYTH_PUSH,
            oracle_keys: [
                kamino_oracle,
                Address::new_from_array([0; 32]),
                Address::new_from_array([0; 32]),
                Address::new_from_array([0; 32]),
                Address::new_from_array([0; 32]),
            ],
            integration_acc_1: kamino_reserve,
        };

        let mut remaining_accounts = Vec::new();
        append_bank_health_accounts(&mut remaining_accounts, &native_bank, &native_fields).unwrap();
        append_bank_health_accounts(&mut remaining_accounts, &kamino_bank, &kamino_fields).unwrap();

        assert_eq!(remaining_accounts[0].pubkey, native_bank);
        assert_eq!(remaining_accounts[1].pubkey, native_oracle);
        assert_eq!(remaining_accounts[2].pubkey, kamino_bank);
        assert_eq!(remaining_accounts[3].pubkey, kamino_oracle);
        assert_eq!(remaining_accounts[4].pubkey, kamino_reserve);
    }
}
