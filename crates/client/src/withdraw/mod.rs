#[cfg(feature = "kamino")]
pub mod kamino;

use solana_address::Address;
#[cfg(feature = "resolve")]
use {
    crate::error::ClientError, solana_instruction::AccountMeta,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

/// Top-level withdraw protocol selector.
pub enum WithdrawProtocol {
    #[cfg(feature = "kamino")]
    Kamino {
        reserve: Address,
        obligation: Address,
        user_destination_liquidity: Address,
    },
}

/// Resolve accounts and data for a withdraw protocol.
#[cfg(feature = "resolve")]
pub async fn resolve_withdraw(
    rpc: &RpcClient,
    protocol: &WithdrawProtocol,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    match protocol {
        #[cfg(feature = "kamino")]
        WithdrawProtocol::Kamino {
            reserve,
            obligation,
            user_destination_liquidity,
        } => kamino::resolve(rpc, reserve, obligation, user_destination_liquidity, user).await,
    }
}
