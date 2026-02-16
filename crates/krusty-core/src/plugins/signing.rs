use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

pub fn verify_artifact_signature(
    artifact_bytes: &[u8],
    signature_b64: &str,
    public_key_b64: &str,
) -> Result<()> {
    let key_raw = BASE64
        .decode(public_key_b64)
        .context("invalid trusted key encoding (expected base64)")?;
    let key_raw: [u8; 32] = key_raw
        .try_into()
        .map_err(|_| anyhow!("invalid trusted key length (expected 32 bytes)"))?;

    let signature_raw = BASE64
        .decode(signature_b64)
        .context("invalid signature encoding (expected base64)")?;
    let signature = Signature::from_slice(&signature_raw)
        .map_err(|e| anyhow!("invalid signature bytes: {}", e))?;

    let verifying_key = VerifyingKey::from_bytes(&key_raw)
        .map_err(|e| anyhow!("invalid ed25519 public key: {}", e))?;

    verifying_key
        .verify(artifact_bytes, &signature)
        .map_err(|e| anyhow!("signature verification failed: {}", e))
}
