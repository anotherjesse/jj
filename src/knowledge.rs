use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::fs;
use std::path::{Path, PathBuf};
use ulid::Ulid;

use crate::audit::LedgerEntry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRef {
    pub thread_id: String,
    pub event_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excerpt_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontMatter {
    pub id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub doc_type: String,
    pub status: String,
    pub tags: Vec<String>,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub sources: Vec<SourceRef>,
    pub supersedes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgePatch {
    pub doc_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub doc_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags_add: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags_remove: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_append: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources_add: Option<Vec<SourceRef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supersedes_add: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<JsonValue>,
}

pub struct ApplyResult {
    pub doc_path: PathBuf,
    pub ledger_entry: LedgerEntry,
}

pub fn apply_patch(
    vault_path: &Path,
    patch: KnowledgePatch,
    author: &str,
    reason: &str,
    proposal_id: Option<String>,
) -> Result<ApplyResult> {
    let doc_path = vault_path.join(&patch.doc_path);
    if !doc_path.starts_with(vault_path) {
        return Err(anyhow!("doc_path must be within vault"));
    }

    let mut body = String::new();
    let now = Utc::now();
    let mut front_matter = if doc_path.exists() {
        let content = fs::read_to_string(&doc_path)
            .with_context(|| format!("read {}", doc_path.display()))?;
        let (fm, parsed_body) = parse_markdown(&content)?;
        body = parsed_body;
        fm
    } else {
        let id = patch
            .doc_id
            .unwrap_or_else(|| format!("mem_{}", Ulid::new()));
        let title = patch.title.clone().ok_or_else(|| anyhow!("title required for new doc"))?;
        let doc_type = patch
            .doc_type
            .clone()
            .ok_or_else(|| anyhow!("type required for new doc"))?;
        FrontMatter {
            id,
            title,
            doc_type,
            status: patch.status.clone().unwrap_or_else(|| "active".to_string()),
            tags: Vec::new(),
            confidence: patch.confidence.unwrap_or(0.5),
            created_at: now,
            updated_at: now,
            sources: Vec::new(),
            supersedes: Vec::new(),
        }
    };

    if let Some(title) = patch.title {
        front_matter.title = title;
    }
    if let Some(doc_type) = patch.doc_type {
        front_matter.doc_type = doc_type;
    }
    if let Some(status) = patch.status {
        front_matter.status = status;
    }
    if let Some(confidence) = patch.confidence {
        front_matter.confidence = confidence;
    }

    if let Some(tags_add) = patch.tags_add {
        for tag in tags_add {
            if !front_matter.tags.contains(&tag) {
                front_matter.tags.push(tag);
            }
        }
    }
    if let Some(tags_remove) = patch.tags_remove {
        front_matter.tags.retain(|tag| !tags_remove.contains(tag));
    }
    if let Some(sources_add) = patch.sources_add {
        front_matter.sources.extend(sources_add);
    }
    if let Some(supersedes_add) = patch.supersedes_add {
        for id in supersedes_add {
            if !front_matter.supersedes.contains(&id) {
                front_matter.supersedes.push(id);
            }
        }
    }

    if let Some(append) = patch.body_append {
        if !body.ends_with('\n') && !body.is_empty() {
            body.push('\n');
        }
        body.push_str(&append);
        if !body.ends_with('\n') {
            body.push('\n');
        }
    }

    front_matter.updated_at = now;

    let prior_content = if doc_path.exists() {
        Some(fs::read_to_string(&doc_path)?)
    } else {
        None
    };
    let new_content = render_markdown(&front_matter, &body)?;
    fs::create_dir_all(doc_path.parent().unwrap_or(Path::new(".")))?;
    fs::write(&doc_path, new_content.as_bytes())
        .with_context(|| format!("write {}", doc_path.display()))?;

    let ledger_entry = LedgerEntry::from_change(
        author,
        reason,
        proposal_id,
        "upsert_knowledge",
        &patch,
        prior_content.as_deref(),
        &new_content,
        &front_matter.id,
        &patch.doc_path,
    );

    Ok(ApplyResult { doc_path, ledger_entry })
}

fn parse_markdown(content: &str) -> Result<(FrontMatter, String)> {
    let mut lines = content.lines();
    let first = lines.next().ok_or_else(|| anyhow!("empty doc"))?;
    if first.trim() != "---" {
        return Err(anyhow!("missing front matter"));
    }
    let mut yaml_lines = Vec::new();
    for line in lines.by_ref() {
        if line.trim() == "---" {
            break;
        }
        yaml_lines.push(line);
    }
    let yaml = yaml_lines.join("\n");
    let body = lines.collect::<Vec<_>>().join("\n");
    let front_matter: FrontMatter = serde_yaml::from_str(&yaml)?;
    Ok((front_matter, body))
}

fn render_markdown(front_matter: &FrontMatter, body: &str) -> Result<String> {
    let mut yaml = serde_yaml::to_string(front_matter)?;
    if yaml.starts_with("---") {
        yaml = yaml.trim_start_matches("---").trim_start_matches('\n').to_string();
    }
    let mut out = String::new();
    out.push_str("---\n");
    out.push_str(&yaml);
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("---\n");
    out.push_str(body);
    Ok(out)
}
