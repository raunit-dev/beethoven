#![allow(dead_code)]

use {borsh::BorshDeserialize, solana_address::Address, solana_instruction::AccountMeta};

pub const KAMINO_LEND_PROGRAM_ID: Address =
    Address::from_str_const("KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD");
pub const FARMS_PROGRAM_ID: Address =
    Address::from_str_const("FarmsPZpWu9i7Kky8tPN37rs2TpmMrAZrC7S7vJa91Hr");

const RESERVE_ACCOUNT_DISCRIMINATOR: [u8; 8] = [43, 242, 204, 202, 26, 247, 59, 127];
const OBLIGATION_ACCOUNT_DISCRIMINATOR: [u8; 8] = [168, 206, 141, 106, 88, 76, 172, 167];

const BASE_SEED_REFERRER_TOKEN_STATE: &[u8] = b"referrer_acc";
const BASE_SEED_USER_STATE: &[u8] = b"user";
const LENDING_MARKET_AUTH_SEED: &[u8] = b"lma";

#[derive(BorshDeserialize)]
struct KaminoLastUpdatePrefix {
    slot: u64,
    stale: u8,
    price_status: u8,
    placeholder: [u8; 6],
}

#[derive(BorshDeserialize)]
struct KaminoBigFractionBytesPrefix {
    value: [u64; 4],
    padding: [u64; 2],
}

#[derive(BorshDeserialize)]
struct KaminoReserveLiquidityPrefix {
    mint_pubkey: [u8; 32],
    supply_vault: [u8; 32],
    fee_vault: [u8; 32],
    available_amount: u64,
    borrowed_amount_sf: u128,
    market_price_sf: u128,
    market_price_last_updated_ts: u64,
    mint_decimals: u64,
    deposit_limit_crossed_timestamp: u64,
    borrow_limit_crossed_timestamp: u64,
    cumulative_borrow_rate_bsf: KaminoBigFractionBytesPrefix,
    accumulated_protocol_fees_sf: u128,
    accumulated_referrer_fees_sf: u128,
    pending_referrer_fees_sf: u128,
    absolute_referral_rate_sf: u128,
    token_program: [u8; 32],
}

#[derive(BorshDeserialize)]
struct KaminoReserveCollateralPrefix {
    mint_pubkey: [u8; 32],
    mint_total_supply: u64,
    supply_vault: [u8; 32],
}

#[derive(BorshDeserialize)]
struct KaminoPriceHeuristicPrefix {
    lower: u64,
    upper: u64,
    exp: u64,
}

#[derive(BorshDeserialize)]
struct KaminoScopeConfigurationPrefix {
    price_feed: [u8; 32],
    price_chain: [u16; 4],
    twap_chain: [u16; 4],
}

#[derive(BorshDeserialize)]
struct KaminoSwitchboardConfigurationPrefix {
    price_aggregator: [u8; 32],
    twap_aggregator: [u8; 32],
}

#[derive(BorshDeserialize)]
struct KaminoPythConfigurationPrefix {
    price: [u8; 32],
}

#[derive(BorshDeserialize)]
struct KaminoTokenInfoPrefix {
    name: [u8; 32],
    heuristic: KaminoPriceHeuristicPrefix,
    max_twap_divergence_bps: u64,
    max_age_price_seconds: u64,
    max_age_twap_seconds: u64,
    scope_configuration: KaminoScopeConfigurationPrefix,
    switchboard_configuration: KaminoSwitchboardConfigurationPrefix,
    pyth_configuration: KaminoPythConfigurationPrefix,
}

#[derive(BorshDeserialize)]
struct KaminoReserveFeesPrefix {
    origination_fee_sf: u64,
    flash_loan_fee_sf: u64,
    padding: [u8; 8],
}

#[derive(BorshDeserialize)]
struct KaminoCurvePointPrefix {
    utilization_rate_bps: u32,
    borrow_rate_bps: u32,
}

#[derive(BorshDeserialize)]
struct KaminoBorrowRateCurvePrefix {
    points: [KaminoCurvePointPrefix; 11],
}

#[derive(BorshDeserialize)]
struct KaminoReserveConfigPrefix {
    status: u8,
    padding_deprecated_asset_tier: u8,
    host_fixed_interest_rate_bps: u16,
    min_deleveraging_bonus_bps: u16,
    block_ctoken_usage: u8,
    reserved1: [u8; 6],
    protocol_order_execution_fee_pct: u8,
    protocol_take_rate_pct: u8,
    protocol_liquidation_fee_pct: u8,
    loan_to_value_pct: u8,
    liquidation_threshold_pct: u8,
    min_liquidation_bonus_bps: u16,
    max_liquidation_bonus_bps: u16,
    bad_debt_liquidation_bonus_bps: u16,
    deleveraging_margin_call_period_secs: u64,
    deleveraging_threshold_decrease_bps_per_day: u64,
    fees: KaminoReserveFeesPrefix,
    borrow_rate_curve: KaminoBorrowRateCurvePrefix,
    borrow_factor_pct: u64,
    deposit_limit: u64,
    borrow_limit: u64,
    token_info: KaminoTokenInfoPrefix,
}

#[derive(BorshDeserialize)]
struct KaminoReservePrefix {
    version: u64,
    last_update: KaminoLastUpdatePrefix,
    lending_market: [u8; 32],
    farm_collateral: [u8; 32],
    farm_debt: [u8; 32],
    liquidity: KaminoReserveLiquidityPrefix,
    reserve_liquidity_padding: [u64; 150],
    collateral: KaminoReserveCollateralPrefix,
    reserve_collateral_padding: [u64; 150],
    config: KaminoReserveConfigPrefix,
}

#[derive(BorshDeserialize)]
struct KaminoObligationCollateralPrefix {
    deposit_reserve: [u8; 32],
    deposited_amount: u64,
    market_value_sf: u128,
    borrowed_amount_against_this_collateral_in_elevation_group: u64,
    padding: [u64; 9],
}

#[derive(BorshDeserialize)]
struct KaminoObligationLiquidityPrefix {
    borrow_reserve: [u8; 32],
    cumulative_borrow_rate_bsf: KaminoBigFractionBytesPrefix,
    first_borrowed_at_timestamp: u64,
    borrowed_amount_sf: u128,
    market_value_sf: u128,
    borrow_factor_adjusted_market_value_sf: u128,
    borrowed_amount_outside_elevation_groups: u64,
    padding2: [u64; 7],
}

#[derive(BorshDeserialize)]
struct KaminoObligationPrefix {
    tag: u64,
    last_update: KaminoLastUpdatePrefix,
    lending_market: [u8; 32],
    owner: [u8; 32],
    deposits: [KaminoObligationCollateralPrefix; 8],
    lowest_reserve_deposit_liquidation_ltv: u64,
    deposited_value_sf: u128,
    borrows: [KaminoObligationLiquidityPrefix; 5],
    borrow_factor_adjusted_debt_value_sf: u128,
    borrowed_assets_market_value_sf: u128,
    allowed_borrow_value_sf: u128,
    unhealthy_borrow_value_sf: u128,
    padding_deprecated_asset_tiers: [u8; 13],
    elevation_group: u8,
    num_of_obsolete_deposit_reserves: u8,
    has_debt: u8,
    referrer: [u8; 32],
}

#[derive(Clone, Copy)]
struct KaminoOracleAddresses {
    pyth: Address,
    switchboard_price: Address,
    switchboard_twap: Address,
    scope_prices: Address,
}

#[derive(Clone, Copy)]
struct KaminoReserveSnapshot {
    lending_market: Address,
    farm_collateral: Address,
    liquidity_mint: Address,
    liquidity_supply_vault: Address,
    liquidity_token_program: Address,
    collateral_mint: Address,
    collateral_supply_vault: Address,
    oracles: KaminoOracleAddresses,
}

struct KaminoObligationSnapshot {
    lending_market: Address,
    owner: Address,
    referrer: Address,
    active_deposit_reserves: Vec<Address>,
    active_borrow_reserves: Vec<Address>,
}

/// Pre-resolved addresses for building a Kamino deposit instruction offline.
pub struct KaminoDepositInput {
    pub user: Address,
    pub obligation: Address,
    pub lending_market: Address,
    pub lending_market_authority: Address,
    pub reserve: Address,
    pub reserve_liquidity_mint: Address,
    pub reserve_liquidity_supply: Address,
    pub reserve_collateral_mint: Address,
    pub reserve_destination_deposit_collateral: Address,
    pub user_source_liquidity: Address,
    pub placeholder_user_destination_collateral: Address,
    pub collateral_token_program: Address,
    pub liquidity_token_program: Address,
    pub instruction_sysvar_account: Address,
    pub obligation_farm_user_state: Address,
    pub reserve_farm_state: Address,
    pub farms_program: Address,
    pub reserve_pyth_oracle: Address,
    pub reserve_switchboard_price_oracle: Address,
    pub reserve_switchboard_twap_oracle: Address,
    pub reserve_scope_prices: Address,
    pub remaining_accounts: Vec<AccountMeta>,
}

/// Build Kamino deposit AccountMeta list from pre-resolved addresses.
pub fn build_accounts(input: &KaminoDepositInput) -> Vec<AccountMeta> {
    let mut accounts = vec![
        AccountMeta::new_readonly(KAMINO_LEND_PROGRAM_ID, false),
        AccountMeta::new(input.user, true),
        AccountMeta::new(input.obligation, false),
        AccountMeta::new_readonly(input.lending_market, false),
        AccountMeta::new_readonly(input.lending_market_authority, false),
        AccountMeta::new(input.reserve, false),
        AccountMeta::new_readonly(input.reserve_liquidity_mint, false),
        AccountMeta::new(input.reserve_liquidity_supply, false),
        AccountMeta::new(input.reserve_collateral_mint, false),
        AccountMeta::new(input.reserve_destination_deposit_collateral, false),
        AccountMeta::new(input.user_source_liquidity, false),
        AccountMeta::new_readonly(input.placeholder_user_destination_collateral, false),
        AccountMeta::new_readonly(input.collateral_token_program, false),
        AccountMeta::new_readonly(input.liquidity_token_program, false),
        AccountMeta::new_readonly(input.instruction_sysvar_account, false),
        AccountMeta::new(input.obligation_farm_user_state, false),
        AccountMeta::new(input.reserve_farm_state, false),
        AccountMeta::new_readonly(input.farms_program, false),
        AccountMeta::new_readonly(input.reserve_pyth_oracle, false),
        AccountMeta::new_readonly(input.reserve_switchboard_price_oracle, false),
        AccountMeta::new_readonly(input.reserve_switchboard_twap_oracle, false),
        AccountMeta::new_readonly(input.reserve_scope_prices, false),
    ];
    accounts.extend(input.remaining_accounts.iter().cloned());
    accounts
}

/// Build Kamino extra data for the Beethoven-side refresh grouping.
pub fn build_extra_data(
    refresh_reserve_group_count: u8,
    deposit_reserve_count: u8,
    borrow_reserve_count: u8,
    borrow_referrer_token_state_count: u8,
) -> Vec<u8> {
    vec![
        refresh_reserve_group_count,
        deposit_reserve_count,
        borrow_reserve_count,
        borrow_referrer_token_state_count,
    ]
}

fn default_address() -> Address {
    Address::new_from_array([0; 32])
}

fn address_from_bytes(bytes: [u8; 32]) -> Address {
    Address::from(bytes)
}

fn optional_or_program_id(address: Address) -> Address {
    if address == default_address() {
        KAMINO_LEND_PROGRAM_ID
    } else {
        address
    }
}

fn decode_account_prefix<T: BorshDeserialize>(
    data: &[u8],
    discriminator: &[u8; 8],
    label: &str,
) -> Result<T, crate::error::ClientError> {
    if data.len() < 8 {
        return Err(crate::error::ClientError::InvalidAccountData(format!(
            "{label} data too short: {}",
            data.len()
        )));
    }

    if data[..8] != discriminator[..] {
        return Err(crate::error::ClientError::InvalidAccountData(format!(
            "{label} discriminator mismatch"
        )));
    }

    let mut payload = &data[8..];
    T::deserialize(&mut payload).map_err(|err| {
        crate::error::ClientError::InvalidAccountData(format!(
            "failed to deserialize {label}: {err}"
        ))
    })
}

fn read_reserve_snapshot(data: &[u8]) -> Result<KaminoReserveSnapshot, crate::error::ClientError> {
    let reserve: KaminoReservePrefix =
        decode_account_prefix(data, &RESERVE_ACCOUNT_DISCRIMINATOR, "kamino reserve")?;

    Ok(KaminoReserveSnapshot {
        lending_market: address_from_bytes(reserve.lending_market),
        farm_collateral: address_from_bytes(reserve.farm_collateral),
        liquidity_mint: address_from_bytes(reserve.liquidity.mint_pubkey),
        liquidity_supply_vault: address_from_bytes(reserve.liquidity.supply_vault),
        liquidity_token_program: address_from_bytes(reserve.liquidity.token_program),
        collateral_mint: address_from_bytes(reserve.collateral.mint_pubkey),
        collateral_supply_vault: address_from_bytes(reserve.collateral.supply_vault),
        oracles: KaminoOracleAddresses {
            pyth: optional_or_program_id(address_from_bytes(
                reserve.config.token_info.pyth_configuration.price,
            )),
            switchboard_price: optional_or_program_id(address_from_bytes(
                reserve
                    .config
                    .token_info
                    .switchboard_configuration
                    .price_aggregator,
            )),
            switchboard_twap: optional_or_program_id(address_from_bytes(
                reserve
                    .config
                    .token_info
                    .switchboard_configuration
                    .twap_aggregator,
            )),
            scope_prices: optional_or_program_id(address_from_bytes(
                reserve.config.token_info.scope_configuration.price_feed,
            )),
        },
    })
}

fn read_obligation_snapshot(
    data: &[u8],
) -> Result<KaminoObligationSnapshot, crate::error::ClientError> {
    let obligation: KaminoObligationPrefix =
        decode_account_prefix(data, &OBLIGATION_ACCOUNT_DISCRIMINATOR, "kamino obligation")?;

    let active_deposit_reserves = obligation
        .deposits
        .iter()
        .map(|deposit| address_from_bytes(deposit.deposit_reserve))
        .filter(|address| *address != default_address())
        .collect();
    let active_borrow_reserves = obligation
        .borrows
        .iter()
        .map(|borrow| address_from_bytes(borrow.borrow_reserve))
        .filter(|address| *address != default_address())
        .collect();

    Ok(KaminoObligationSnapshot {
        lending_market: address_from_bytes(obligation.lending_market),
        owner: address_from_bytes(obligation.owner),
        referrer: address_from_bytes(obligation.referrer),
        active_deposit_reserves,
        active_borrow_reserves,
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

fn append_unique(vec: &mut Vec<Address>, address: Address) {
    if !vec.contains(&address) {
        vec.push(address);
    }
}

fn derive_non_current_refresh_reserves(
    active_deposit_reserves: &[Address],
    active_borrow_reserves: &[Address],
    current_reserve: &Address,
) -> Vec<Address> {
    let mut reserves = Vec::new();
    for reserve in active_deposit_reserves {
        if reserve != current_reserve {
            append_unique(&mut reserves, *reserve);
        }
    }
    for reserve in active_borrow_reserves {
        if reserve != current_reserve {
            append_unique(&mut reserves, *reserve);
        }
    }
    reserves
}

fn derive_lending_market_authority(lending_market: &Address) -> Address {
    Address::find_program_address(
        &[LENDING_MARKET_AUTH_SEED, lending_market.as_ref()],
        &KAMINO_LEND_PROGRAM_ID,
    )
    .0
}

fn derive_referrer_token_state(referrer: &Address, reserve: &Address) -> Address {
    Address::find_program_address(
        &[
            BASE_SEED_REFERRER_TOKEN_STATE,
            referrer.as_ref(),
            reserve.as_ref(),
        ],
        &KAMINO_LEND_PROGRAM_ID,
    )
    .0
}

fn derive_obligation_farm_user_state(farm: &Address, obligation: &Address) -> Address {
    Address::find_program_address(
        &[BASE_SEED_USER_STATE, farm.as_ref(), obligation.as_ref()],
        &FARMS_PROGRAM_ID,
    )
    .0
}

/// Resolve accounts and data for a Kamino deposit via RPC.
#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &solana_rpc_client::nonblocking::rpc_client::RpcClient,
    reserve: &Address,
    obligation: &Address,
    user_source_liquidity: &Address,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), crate::error::ClientError> {
    let reserve_account = rpc.get_account(reserve).await?;
    if reserve_account.owner != KAMINO_LEND_PROGRAM_ID {
        return Err(crate::error::ClientError::InvalidAccountData(format!(
            "kamino reserve {} is not owned by the Kamino program",
            reserve
        )));
    }
    let reserve_snapshot = read_reserve_snapshot(&reserve_account.data)?;

    let obligation_account = rpc.get_account(obligation).await?;
    if obligation_account.owner != KAMINO_LEND_PROGRAM_ID {
        return Err(crate::error::ClientError::InvalidAccountData(format!(
            "kamino obligation {} is not owned by the Kamino program",
            obligation
        )));
    }
    let obligation_snapshot = read_obligation_snapshot(&obligation_account.data)?;

    if obligation_snapshot.lending_market != reserve_snapshot.lending_market {
        return Err(crate::error::ClientError::InvalidAccountData(format!(
            "kamino obligation {} lending market {} does not match reserve lending market {}",
            obligation, obligation_snapshot.lending_market, reserve_snapshot.lending_market
        )));
    }

    if obligation_snapshot.owner != *user {
        return Err(crate::error::ClientError::InvalidAccountData(format!(
            "kamino obligation {} owner {} does not match user {}",
            obligation, obligation_snapshot.owner, user
        )));
    }

    if reserve_snapshot.liquidity_token_program != crate::TOKEN_PROGRAM_ID
        && reserve_snapshot.liquidity_token_program != crate::TOKEN_2022_PROGRAM_ID
    {
        return Err(crate::error::ClientError::InvalidAccountData(format!(
            "kamino reserve {} uses unsupported liquidity token program {}",
            reserve, reserve_snapshot.liquidity_token_program
        )));
    }

    let user_source_liquidity_account = rpc.get_account(user_source_liquidity).await?;
    if user_source_liquidity_account.owner != reserve_snapshot.liquidity_token_program {
        return Err(crate::error::ClientError::InvalidAccountData(format!(
            "token account {} is owned by {}, expected {}",
            user_source_liquidity,
            user_source_liquidity_account.owner,
            reserve_snapshot.liquidity_token_program
        )));
    }

    let (token_account_mint, token_account_owner) =
        read_token_account_mint_and_owner(&user_source_liquidity_account.data)?;
    if token_account_owner != *user {
        return Err(crate::error::ClientError::InvalidAccountData(format!(
            "token account {} owner {} does not match user {}",
            user_source_liquidity, token_account_owner, user
        )));
    }

    if token_account_mint != reserve_snapshot.liquidity_mint {
        return Err(crate::error::ClientError::MintMismatch {
            expected: reserve_snapshot.liquidity_mint.to_string(),
            got: token_account_mint.to_string(),
        });
    }

    let refresh_reserve_addresses = derive_non_current_refresh_reserves(
        &obligation_snapshot.active_deposit_reserves,
        &obligation_snapshot.active_borrow_reserves,
        reserve,
    );

    let mut remaining_accounts = Vec::with_capacity(
        refresh_reserve_addresses.len() * 5
            + obligation_snapshot.active_deposit_reserves.len()
            + obligation_snapshot.active_borrow_reserves.len()
            + obligation_snapshot.active_borrow_reserves.len(),
    );

    for refresh_reserve_address in &refresh_reserve_addresses {
        let refresh_reserve_account = rpc.get_account(refresh_reserve_address).await?;
        if refresh_reserve_account.owner != KAMINO_LEND_PROGRAM_ID {
            return Err(crate::error::ClientError::InvalidAccountData(format!(
                "kamino reserve {} is not owned by the Kamino program",
                refresh_reserve_address
            )));
        }
        let refresh_reserve = read_reserve_snapshot(&refresh_reserve_account.data)?;
        remaining_accounts.push(AccountMeta::new(*refresh_reserve_address, false));
        remaining_accounts.push(AccountMeta::new_readonly(
            refresh_reserve.oracles.pyth,
            false,
        ));
        remaining_accounts.push(AccountMeta::new_readonly(
            refresh_reserve.oracles.switchboard_price,
            false,
        ));
        remaining_accounts.push(AccountMeta::new_readonly(
            refresh_reserve.oracles.switchboard_twap,
            false,
        ));
        remaining_accounts.push(AccountMeta::new_readonly(
            refresh_reserve.oracles.scope_prices,
            false,
        ));
    }

    for deposit_reserve in &obligation_snapshot.active_deposit_reserves {
        remaining_accounts.push(AccountMeta::new(*deposit_reserve, false));
    }

    for borrow_reserve in &obligation_snapshot.active_borrow_reserves {
        remaining_accounts.push(AccountMeta::new(*borrow_reserve, false));
    }

    let borrow_referrer_token_state_count = if obligation_snapshot.referrer == default_address() {
        0
    } else {
        for borrow_reserve in &obligation_snapshot.active_borrow_reserves {
            remaining_accounts.push(AccountMeta::new(
                derive_referrer_token_state(&obligation_snapshot.referrer, borrow_reserve),
                false,
            ));
        }
        obligation_snapshot.active_borrow_reserves.len() as u8
    };

    let reserve_farm_state = if reserve_snapshot.farm_collateral == default_address() {
        KAMINO_LEND_PROGRAM_ID
    } else {
        reserve_snapshot.farm_collateral
    };
    let obligation_farm_user_state = if reserve_snapshot.farm_collateral == default_address() {
        KAMINO_LEND_PROGRAM_ID
    } else {
        derive_obligation_farm_user_state(&reserve_snapshot.farm_collateral, obligation)
    };

    let input = KaminoDepositInput {
        user: *user,
        obligation: *obligation,
        lending_market: reserve_snapshot.lending_market,
        lending_market_authority: derive_lending_market_authority(&reserve_snapshot.lending_market),
        reserve: *reserve,
        reserve_liquidity_mint: reserve_snapshot.liquidity_mint,
        reserve_liquidity_supply: reserve_snapshot.liquidity_supply_vault,
        reserve_collateral_mint: reserve_snapshot.collateral_mint,
        reserve_destination_deposit_collateral: reserve_snapshot.collateral_supply_vault,
        user_source_liquidity: *user_source_liquidity,
        placeholder_user_destination_collateral: KAMINO_LEND_PROGRAM_ID,
        collateral_token_program: crate::TOKEN_PROGRAM_ID,
        liquidity_token_program: reserve_snapshot.liquidity_token_program,
        instruction_sysvar_account: crate::SYSVAR_INSTRUCTIONS_ID,
        obligation_farm_user_state,
        reserve_farm_state,
        farms_program: FARMS_PROGRAM_ID,
        reserve_pyth_oracle: reserve_snapshot.oracles.pyth,
        reserve_switchboard_price_oracle: reserve_snapshot.oracles.switchboard_price,
        reserve_switchboard_twap_oracle: reserve_snapshot.oracles.switchboard_twap,
        reserve_scope_prices: reserve_snapshot.oracles.scope_prices,
        remaining_accounts,
    };

    Ok((
        build_accounts(&input),
        build_extra_data(
            refresh_reserve_addresses.len() as u8,
            obligation_snapshot.active_deposit_reserves.len() as u8,
            obligation_snapshot.active_borrow_reserves.len() as u8,
            borrow_referrer_token_state_count,
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_extra_data_keeps_explicit_counts() {
        assert_eq!(build_extra_data(2, 3, 1, 1), vec![2, 3, 1, 1]);
    }

    #[test]
    fn build_accounts_keeps_program_first_and_appends_tail() {
        let input = KaminoDepositInput {
            user: Address::new_from_array([1; 32]),
            obligation: Address::new_from_array([2; 32]),
            lending_market: Address::new_from_array([3; 32]),
            lending_market_authority: Address::new_from_array([4; 32]),
            reserve: Address::new_from_array([5; 32]),
            reserve_liquidity_mint: Address::new_from_array([6; 32]),
            reserve_liquidity_supply: Address::new_from_array([7; 32]),
            reserve_collateral_mint: Address::new_from_array([8; 32]),
            reserve_destination_deposit_collateral: Address::new_from_array([9; 32]),
            user_source_liquidity: Address::new_from_array([10; 32]),
            placeholder_user_destination_collateral: KAMINO_LEND_PROGRAM_ID,
            collateral_token_program: crate::TOKEN_PROGRAM_ID,
            liquidity_token_program: crate::TOKEN_2022_PROGRAM_ID,
            instruction_sysvar_account: crate::SYSVAR_INSTRUCTIONS_ID,
            obligation_farm_user_state: Address::new_from_array([11; 32]),
            reserve_farm_state: Address::new_from_array([12; 32]),
            farms_program: FARMS_PROGRAM_ID,
            reserve_pyth_oracle: Address::new_from_array([13; 32]),
            reserve_switchboard_price_oracle: Address::new_from_array([14; 32]),
            reserve_switchboard_twap_oracle: Address::new_from_array([15; 32]),
            reserve_scope_prices: Address::new_from_array([16; 32]),
            remaining_accounts: vec![AccountMeta::new(Address::new_from_array([17; 32]), false)],
        };

        let accounts = build_accounts(&input);

        assert_eq!(accounts.len(), 23);
        assert_eq!(accounts[0].pubkey, KAMINO_LEND_PROGRAM_ID);
        assert_eq!(accounts[1].pubkey, input.user);
        assert!(accounts[1].is_signer);
        assert_eq!(accounts[2].pubkey, input.obligation);
        assert_eq!(accounts[21].pubkey, input.reserve_scope_prices);
        assert_eq!(accounts[22].pubkey, Address::new_from_array([17; 32]));
    }

    #[test]
    fn derive_non_current_refresh_reserves_keeps_deposit_then_borrow_order() {
        let current = Address::new_from_array([1; 32]);
        let deposit_a = Address::new_from_array([2; 32]);
        let deposit_b = Address::new_from_array([3; 32]);
        let borrow_a = Address::new_from_array([4; 32]);

        let reserves = derive_non_current_refresh_reserves(
            &[deposit_a, current, deposit_b],
            &[deposit_b, borrow_a],
            &current,
        );

        assert_eq!(reserves, vec![deposit_a, deposit_b, borrow_a]);
    }
}
