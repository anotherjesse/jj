use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::knowledge::KnowledgePatch;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub ledger_id: String,
    pub ts: DateTime<Utc>,
    pub author: String,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proposal_id: Option<String>,
    pub op: String,
    pub doc_path: String,
    pub doc_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_hash: Option<String>,
    pub new_hash: String,
    pub patch: KnowledgePatch,
}

impl LedgerEntry {
    pub fn from_change(
        author: &str,
        reason: &str,
        proposal_id: Option<String>,
        op: &str,
        patch: &KnowledgePatch,
        prior_content: Option<&str>,
        new_content: &str,
        doc_id: &str,
        doc_path: &str,
    ) -> Self {
        let prev_hash = prior_content.map(hash_str);
        let new_hash = hash_str(new_content);
        LedgerEntry {
            ledger_id: format!("led_{}", ulid::Ulid::new()),
            ts: Utc::now(),
            author: author.to_string(),
            reason: reason.to_string(),
            proposal_id,
            op: op.to_string(),
            doc_path: doc_path.to_string(),
            doc_id: doc_id.to_string(),
            prev_hash,
            new_hash,
            patch: patch.clone(),
        }
    }
}

fn hash_str(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hex::encode(hasher.finalize())
}
