use kzg::{Error as KzgError, Kzg, KzgPreset};
use types::{Blob, EthSpec, Hash256, KzgCommitment, KzgProof};

/// Converts a blob ssz List object to an array to be used with the kzg
/// crypto library.
fn ssz_blob_to_crypto_blob<T: EthSpec>(
    blob: &Blob<T>,
) -> Result<Box<<<T as EthSpec>::Kzg as KzgPreset>::Blob>, KzgError> {
    T::blob_from_bytes(blob.as_ref())
}

/// Validate a single blob-commitment-proof triplet from a `BlobSidecar`.
pub fn validate_blob<T: EthSpec>(
    kzg: &Kzg<T::Kzg>,
    blob: &Blob<T>,
    kzg_commitment: &KzgCommitment,
    kzg_proof: &KzgProof,
) -> Result<bool, KzgError> {
    kzg.verify_blob_kzg_proof(
        &*ssz_blob_to_crypto_blob::<T>(blob)?,
        kzg_commitment,
        kzg_proof,
    )
}

/// Validate a batch of blob-commitment-proof triplets from multiple `BlobSidecars`.
pub fn validate_blobs<T: EthSpec>(
    kzg: &Kzg<T::Kzg>,
    expected_kzg_commitments: &[KzgCommitment],
    blobs: &[Blob<T>],
    kzg_proofs: &[KzgProof],
) -> Result<bool, KzgError> {
    // TODO(sean) batch verification fails with a single element, it's unclear to me why
    if blobs.len() == 1 && kzg_proofs.len() == 1 && expected_kzg_commitments.len() == 1 {
        if let (Some(blob), Some(kzg_proof), Some(kzg_commitment)) = (
            blobs.get(0),
            kzg_proofs.get(0),
            expected_kzg_commitments.get(0),
        ) {
            return validate_blob::<T>(kzg, blob, kzg_commitment, kzg_proof);
        } else {
            return Ok(false);
        }
    }

    let blobs = blobs
        .iter()
        .map(|blob| ssz_blob_to_crypto_blob::<T>(blob))
        .collect::<Result<Vec<Box<_>>, KzgError>>()?
        .into_iter()
        .map(|boxed_blob| *boxed_blob)
        .collect::<Vec<_>>();

    kzg.verify_blob_kzg_proof_batch(&blobs, expected_kzg_commitments, kzg_proofs)
}

/// Compute the kzg proof given an ssz blob and its kzg commitment.
pub fn compute_blob_kzg_proof<T: EthSpec>(
    kzg: &Kzg<T::Kzg>,
    blob: &Blob<T>,
    kzg_commitment: &KzgCommitment,
) -> Result<KzgProof, KzgError> {
    kzg.compute_blob_kzg_proof(&*ssz_blob_to_crypto_blob::<T>(blob)?, kzg_commitment)
}

/// Compute the kzg commitment for a given blob.
pub fn blob_to_kzg_commitment<T: EthSpec>(
    kzg: &Kzg<T::Kzg>,
    blob: &Blob<T>,
) -> Result<KzgCommitment, KzgError> {
    kzg.blob_to_kzg_commitment(&*ssz_blob_to_crypto_blob::<T>(blob)?)
}

/// Compute the kzg proof for a given blob and an evaluation point z.
pub fn compute_kzg_proof<T: EthSpec>(
    kzg: &Kzg<T::Kzg>,
    blob: &Blob<T>,
    z: &Hash256,
) -> Result<(KzgProof, Hash256), KzgError> {
    let z = &z.0.into();
    kzg.compute_kzg_proof(&*ssz_blob_to_crypto_blob::<T>(blob)?, z)
        .map(|(proof, z)| (proof, Hash256::from_slice(&z.to_vec())))
}

/// Verify a `kzg_proof` for a `kzg_commitment` that evaluating a polynomial at `z` results in `y`
pub fn verify_kzg_proof<T: EthSpec>(
    kzg: &Kzg<T::Kzg>,
    kzg_commitment: &KzgCommitment,
    kzg_proof: &KzgProof,
    z: &Hash256,
    y: &Hash256,
) -> Result<bool, KzgError> {
    kzg.verify_kzg_proof(kzg_commitment, &z.0.into(), &y.0.into(), kzg_proof)
}
