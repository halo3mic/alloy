use std::sync::{Arc, OnceLock};

use alloy_network::{Network, TransactionBuilder};
use alloy_transport::TransportResult;

use crate::{
    fillers::{FillerControlFlow, TxFiller},
    provider::SendableTx,
};

/// A [`TxFiller`] that populates the gas price of a transaction.
///
/// If a gas price is provided, it will be used for filling, otherwise the 
/// filler will fetch the gas price from the provider each time a transaction
/// is prepared.
///
/// Transactions that already have a gas price set by the user will not be
/// modified.
///
/// # Example
///
/// ```
/// # use alloy_network::{NetworkSigner, EthereumSigner, Ethereum};
/// # use alloy_rpc_types::TransactionRequest;
/// # use alloy_provider::{ProviderBuilder, RootProvider, Provider};
/// # async fn test<S: NetworkSigner<Ethereum> + Clone>(url: url::Url, signer: S) -> Result<(), Box<dyn std::error::Error>> {
/// let provider = ProviderBuilder::new()
///     .with_gas_pricing()
///     .signer(signer)
///     .on_http(url)?;
///
/// provider.send_transaction(TransactionRequest::default()).await;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GasPriceFiller(Arc<OnceLock<u128>>);

impl GasPriceFiller {
    /// Create a new [`GasPriceFiller`] with an optional gas price.
    ///
    /// If gas price is provided, it will be used for filling. If gas price is 
    /// not provided it will be fetched from the provider every time a transaction
    /// is prepared.
    pub fn new(gas_price: Option<u128>) -> Self {
        let lock = OnceLock::new();
        if let Some(gas_price) = gas_price {
            lock.set(gas_price).expect("brand new");
        }
        Self(Arc::new(lock))
    }
}

impl<N: Network> TxFiller<N> for GasPriceFiller {
    type Fillable = u128;

    fn status(&self, tx: &N::TransactionRequest) -> FillerControlFlow {
        if tx.gas_price().is_some() {
            FillerControlFlow::Finished
        } else {
            FillerControlFlow::Ready
        }
    }

    async fn prepare<P, T>(
        &self,
        provider: &P,
        _tx: &N::TransactionRequest,
    ) -> TransportResult<Self::Fillable>
    where
        P: crate::Provider<T, N>,
        T: alloy_transport::Transport + Clone,
    {
        match self.0.get().copied() {
            Some(gas_price) => Ok(gas_price),
            None => {
                let gas_price = provider.get_gas_price().await?;
                let gas_price = *self.0.get_or_init(|| gas_price);
                Ok(gas_price)
            }
        }
    }

    async fn fill(
        &self,
        fillable: Self::Fillable,
        mut tx: SendableTx<N>,
    ) -> TransportResult<SendableTx<N>> {
        if let Some(builder) = tx.as_mut_builder() {
            if builder.gas_price().is_none() {
                builder.set_gas_price(fillable)
            }
        };
        Ok(tx)
    }
}
