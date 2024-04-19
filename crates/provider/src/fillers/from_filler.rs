use alloy_network::{Network, TransactionBuilder};
use alloy_transport::TransportResult;
use alloy_primitives::Address;

use crate::{
    fillers::{FillerControlFlow, TxFiller},
    provider::SendableTx,
};

/// A [`TxFiller`] that populates the from address of a transaction.
///
/// This filler is not meant to be used on its own, but rather as part of 
/// SignerFiller. It is used to fill the from address of a transaction so that 
/// NonceFiller can fetch the nonce before the transaction is signed.
///
/// Transactions that already have a from set by the user will not be
/// modified.
///
/// # Example
///
/// ```
/// # use alloy_network::{NetworkSigner, EthereumSigner, Ethereum};
/// # use alloy_rpc_types::TransactionRequest;
/// # use alloy_provider::{ProviderBuilder, RootProvider, Provider};
/// # use alloy_primitives::Address;
/// # async fn test<S: NetworkSigner<Ethereum> + Clone>(url: url::Url, signer: S) -> Result<(), Box<dyn std::error::Error>> {
/// let provider = ProviderBuilder::new()
///     .with_from(Address::ZERO)
///     .signer(signer)
///     .on_http(url)?;
///
/// provider.send_transaction(TransactionRequest::default()).await;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FromFiller(Address);

impl FromFiller {
    /// Create a new [`FromFiller`] with a from address.
    pub fn new(address: Address) -> Self {
        Self(address)
    }
}

impl<N: Network> TxFiller<N> for FromFiller {
    type Fillable = ();

    fn status(&self, tx: &N::TransactionRequest) -> FillerControlFlow {
        let status = {
            if tx.from().is_some() {
                FillerControlFlow::Finished
            } else {
                FillerControlFlow::Ready
            }
        };
        status
    }

    async fn prepare<P, T>(
        &self,
        _provider: &P,
        _tx: &N::TransactionRequest,
    ) -> TransportResult<Self::Fillable>
    where
        P: crate::Provider<T, N>,
        T: alloy_transport::Transport + Clone,
    {
        Ok(())
    }

    async fn fill(
        &self,
        _fillable: Self::Fillable,
        mut tx: SendableTx<N>,
    ) -> TransportResult<SendableTx<N>> {
        if let Some(builder) = tx.as_mut_builder() {
            builder.set_from(self.0);
        }
        Ok(tx)
    }
}
