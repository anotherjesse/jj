use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use ulid::Ulid;

use crate::embeddings::EmbeddingClient;
use crate::knowledge::read_doc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRecord {
    pub doc_path: String,
    pub chunk_id: String,
    pub text: String,
    pub embedding: Vec<f32>,
    pub ts: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub doc_path: String,
    pub chunk_id: String,
    pub score: f32,
    pub excerpt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub doc_count: usize,
    pub chunk_count: usize,
    pub index_path: PathBuf,
    pub provider: String,
    pub model: String,
}

pub fn build_knowledge_index(vault: &Path, client: &EmbeddingClient) -> Result<IndexStats> {
    let knowledge_root = vault.join("knowledge");
    let index_dir = vault.join("index");
    fs::create_dir_all(&index_dir)?;
    let index_path = index_dir.join("knowledge_embeddings.jsonl");
    let mut file = fs::File::create(&index_path)
        .with_context(|| format!("create index {}", index_path.display()))?;

    let mut doc_count = 0;
    let mut chunk_count = 0;
    for path in walk_markdown(&knowledge_root)? {
        let doc = read_doc(&path)?;
        let title = doc.front_matter.title;
        let mut combined = String::new();
        combined.push_str(&title);
        combined.push_str("\n\n");
        combined.push_str(&doc.body);

        let chunks = chunk_text(&combined, 2000);
        if chunks.is_empty() {
            continue;
        }
        doc_count += 1;
        let rel_path = path.strip_prefix(vault).unwrap_or(&path).to_string_lossy().to_string();
        for chunk in chunks {
            let embedding = client.embed_text(&chunk)?;
            let record = EmbeddingRecord {
                doc_path: rel_path.clone(),
                chunk_id: format!("chk_{}", Ulid::new()),
                text: chunk,
                embedding,
                ts: Utc::now(),
            };
            let line = serde_json::to_string(&record)?;
            file.write_all(line.as_bytes())?;
            file.write_all(b"\n")?;
            chunk_count += 1;
        }
    }

    Ok(IndexStats {
        doc_count,
        chunk_count,
        index_path,
        provider: format!("{:?}", client.provider()),
        model: client.model().to_string(),
    })
}

pub fn search_knowledge_index(
    vault: &Path,
    client: &EmbeddingClient,
    query: &str,
    limit: usize,
) -> Result<Vec<SearchHit>> {
    let index_path = vault.join("index/knowledge_embeddings.jsonl");
    if !index_path.exists() {
        return Err(anyhow!("embedding index not found"));
    }
    let query_embedding = client.embed_text(query)?;
    let query_norm = vector_norm(&query_embedding);

    let file = fs::File::open(&index_path)?;
    let reader = BufReader::new(file);
    let mut hits = Vec::new();
    for line in reader.lines() {
        let line = line?;
        let record: EmbeddingRecord = serde_json::from_str(&line)?;
        if record.embedding.is_empty() {
            continue;
        }
        let score = cosine_similarity(&query_embedding, query_norm, &record.embedding);
        hits.push(SearchHit {
            doc_path: record.doc_path,
            chunk_id: record.chunk_id,
            score,
            excerpt: excerpt_at(&record.text, 160),
        });
    }

    hits.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    if hits.len() > limit {
        hits.truncate(limit);
    }
    Ok(hits)
}

fn walk_markdown(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if !root.exists() {
        return Ok(files);
    }
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|s| s.to_str()) == Some("md") {
                files.push(path);
            }
        }
    }
    Ok(files)
}

fn chunk_text(text: &str, max_len: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();
    for line in text.lines() {
        if line.starts_with('#') && !current.is_empty() {
            push_chunk(&mut chunks, &mut current, max_len);
        }
        current.push_str(line);
        current.push('\n');
        if current.len() >= max_len {
            push_chunk(&mut chunks, &mut current, max_len);
        }
    }
    if !current.is_empty() {
        push_chunk(&mut chunks, &mut current, max_len);
    }
    chunks
}

fn push_chunk(chunks: &mut Vec<String>, current: &mut String, max_len: usize) {
    if current.len() <= max_len {
        chunks.push(current.trim().to_string());
        current.clear();
        return;
    }
    let mut start = 0;
    let bytes = current.as_bytes();
    while start < bytes.len() {
        let end = usize::min(start + max_len, bytes.len());
        let slice = &current[start..end];
        chunks.push(slice.trim().to_string());
        start = end;
    }
    current.clear();
}

fn vector_norm(values: &[f32]) -> f32 {
    values.iter().map(|v| v * v).sum::<f32>().sqrt()
}

fn cosine_similarity(query: &[f32], query_norm: f32, doc: &[f32]) -> f32 {
    let doc_norm = vector_norm(doc);
    if query_norm == 0.0 || doc_norm == 0.0 {
        return 0.0;
    }
    let dot = query.iter().zip(doc).map(|(a, b)| a * b).sum::<f32>();
    dot / (query_norm * doc_norm)
}

fn excerpt_at(text: &str, max_len: usize) -> String {
    let mut snippet = text.trim().replace('\n', " ");
    if snippet.len() > max_len {
        snippet.truncate(max_len);
        snippet.push('…');
    }
    snippet
}
