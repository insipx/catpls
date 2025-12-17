use alloy::primitives::{
    Address,
    BlockNumber,
    Bytes,
};
use alloy::signers::Signer;
use alloy::signers::aws::AwsSigner;
use color_eyre::eyre::{
    Result,
    WrapErr,
};
use futures::future::OptionFuture;
use futures::{
    FutureExt,
    future,
};
use xmtp_id::associations::builder::SignatureRequest;
use xmtp_id::associations::unverified::UnverifiedSignature;
use xmtp_id::associations::{
    AccountId,
    Identifier,
};
use xmtp_id::scw_verifier::{
    SmartContractSignatureVerifier,
    ValidationResponse,
    VerifierError,
};
use xmtp_mls::identity::IdentityStrategy;
use xmtp_id::InboxId;


pub struct CatBotIdentity {
    /// signer
    signer: AwsSigner,
    /// xmtp identifier
    xmtp_identifier: Identifier,
    /// the ethereum addresss
    address: Address,
    inbox_id: InboxId,
    /// nonce of this identity
    nonce: u64,
}

impl CatBotIdentity {
    pub async fn new(chain_id: u64, aws_key_id: String, nonce: u64) -> Result<CatBotIdentity> {
        let config = crate::aws_config().await?;
        let client = aws_sdk_kms::Client::new(&config);
        let signer = AwsSigner::new(client, aws_key_id, Some(chain_id)).await.unwrap();
        let address = signer.address();
        info!(key = %address, "using aws identity");
        let xmtp_identifier = to_identifier(&address)?;
        let inbox_id = xmtp_identifier.inbox_id(nonce)?;
        Ok(CatBotIdentity { xmtp_identifier, address, signer, inbox_id, nonce })
    }

    pub async fn sign<F>(&self, f: F) -> Result<Option<SignatureRequest>>
    where
        F: Fn() -> Option<SignatureRequest>,
    {
        let fut: OptionFuture<_> = f()
            .map(async |mut request| {
                let text = request.signature_text();
                let signature = self.signer.sign_message(text.as_bytes()).await?;
                let signature = UnverifiedSignature::new_recoverable_ecdsa(signature.into());
                request.add_signature(signature, ScwAgentNotSupported).await?;
                Ok(request)
            })
            .into();
        fut.await.transpose()
    }

    pub fn strategy(&self, cached: bool) -> IdentityStrategy {
        if cached {
           IdentityStrategy::CachedOnly
        } else {
            IdentityStrategy::CreateIfNotFound {
                inbox_id: self.inbox_id.clone(),
                identifier: self.xmtp_identifier.clone(),
                nonce: self.nonce,
                legacy_signed_private_key: None
            }
        }
    }
}

async fn sign() -> Result<()> {
    Ok(())
}

fn to_identifier(addr: &Address) -> Result<Identifier> {
    Ok(Identifier::eth(format!("{addr:?}").to_lowercase())?)
}

struct ScwAgentNotSupported;

#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
impl SmartContractSignatureVerifier for ScwAgentNotSupported {
    async fn is_valid_signature(
        &self,
        account_id: AccountId,
        hash: [u8; 32],
        signature: Bytes,
        block_number: Option<BlockNumber>,
    ) -> Result<ValidationResponse, VerifierError> {
        Ok(ValidationResponse {
            is_valid: false,
            block_number: None,
            error: Some("Catbot does not support SCW Identity on agent".to_string()),
        })
    }
}
