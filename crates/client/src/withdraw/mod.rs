#[cfg(feature = "marginfi")]
pub mod marginfi;

use solana_address::Address;
#[cfg(feature = "resolve")]
use {
    crate::error::ClientError, solana_instruction::AccountMeta,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

/// Top-level withdraw protocol selector.
pub enum WithdrawProtocol {
    #[cfg(feature = "marginfi")]
    Marginfi {
        bank: Address,
        marginfi_account: Address,
        destination_token_account: Address,
        withdraw_all: Option<bool>,
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
        #[cfg(feature = "marginfi")]
        WithdrawProtocol::Marginfi {
            bank,
            marginfi_account,
            destination_token_account,
            withdraw_all,
        } => {
            marginfi::resolve(
                rpc,
                bank,
                marginfi_account,
                destination_token_account,
                *withdraw_all,
                user,
            )
            .await
        }
    }
}
