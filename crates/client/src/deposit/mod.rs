#[cfg(feature = "kamino")]
pub mod kamino;

use solana_address::Address;
#[cfg(feature = "resolve")]
use {
    crate::error::ClientError, solana_instruction::AccountMeta,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

/// Top-level deposit protocol selector.
///
/// Each variant carries the protocol-specific config needed to resolve the
/// exact accounts and extra data for a Beethoven deposit instruction.
pub enum DepositProtocol {
    #[cfg(feature = "kamino")]
    Kamino {
        reserve: Address,
        obligation: Address,
        user_source_liquidity: Address,
    },
}

/// Resolve accounts and data for a deposit protocol.
///
/// Returns `(accounts, extra_data)` ready for the Beethoven on-chain program.
#[cfg(feature = "resolve")]
pub async fn resolve_deposit(
    rpc: &RpcClient,
    protocol: &DepositProtocol,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    match protocol {
        #[cfg(feature = "kamino")]
        DepositProtocol::Kamino {
            reserve,
            obligation,
            user_source_liquidity,
        } => kamino::resolve(rpc, reserve, obligation, user_source_liquidity, user).await,
    }
}
