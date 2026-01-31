use anyhow::{anyhow, Context, Result};
use chrono::{Datelike, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use ulid::Ulid;

use crate::agent::{run_agent_loop, AgentConfig};
use crate::embedding_index::build_knowledge_index;
use crate::embeddings::EmbeddingClient;
use crate::git_utils::git_commit;
use crate::openai::OpenAIClient;
use crate::thread_store::create_thread;
use crate::vault::resolve_vault;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceFrontMatter {
    pub id: String,
    pub title: String,
    pub ingested_at: String,
    pub original_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub processing_status: String,
    pub content_hash: String,
}

pub struct IngestOptions {
    pub vault: Option<PathBuf>,
    pub file: PathBuf,
    pub source: Option<String>,
    pub tags: Vec<String>,
    pub title: Option<String>,
    pub model: Option<String>,
}

pub struct IngestResult {
    pub source_path: PathBuf,
    pub summary_path: PathBuf,
    pub thread_path: PathBuf,
    pub proposal_count: usize,
}

pub fn run_ingest(options: IngestOptions) -> Result<IngestResult> {
    dotenvy::dotenv().ok();

    let vault = resolve_vault(options.vault);
    if !vault.exists() {
        return Err(anyhow!("vault does not exist: {}. Run `jay vault init` first.", vault.display()));
    }

    // Validate input file
    let file_content = fs::read_to_string(&options.file)
        .with_context(|| format!("read input file {}", options.file.display()))?;

    let original_path = fs::canonicalize(&options.file)
        .unwrap_or_else(|_| options.file.clone())
        .to_string_lossy()
        .to_string();

    // Generate slug and title
    let title = options.title.unwrap_or_else(|| {
        options
            .file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("untitled")
            .to_string()
    });
    let slug = slugify(&title);

    // Compute content hash
    let content_hash = {
        let mut hasher = Sha256::new();
        hasher.update(file_content.as_bytes());
        format!("sha256:{}", hex::encode(hasher.finalize()))
    };

    // Build source frontmatter
    let now = Utc::now();
    let source_id = format!("src_{}", Ulid::new());
    let front_matter = SourceFrontMatter {
        id: source_id.clone(),
        title: title.clone(),
        ingested_at: now.to_rfc3339(),
        original_path,
        source: options.source.clone(),
        tags: options.tags.clone(),
        processing_status: "processing".to_string(),
        content_hash: content_hash.clone(),
    };

    // Write source file
    let date_dir = format!("{:04}/{:02}/{:02}", now.year(), now.month(), now.day());
    let source_dir = vault.join("sources").join(&date_dir);
    fs::create_dir_all(&source_dir)?;
    let source_path = unique_path(&source_dir, &slug, "md");
    let source_content = render_source_file(&front_matter, &file_content)?;
    fs::write(&source_path, &source_content)?;

    println!("Copied source to {}", source_path.display());

    // Create ingestion thread
    let thread_path = create_thread(&vault, None, None)?;

    // Load ingestion system prompt
    let system_prompt = load_ingest_prompt(&vault, &slug, &source_id)?;

    // Set up LLM client
    let api_key = env::var("OPENAI_API_KEY").context("OPENAI_API_KEY is not set")?;
    let base_url = env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com".to_string());
    let model = options
        .model
        .or_else(|| env::var("OPENAI_MODEL").ok())
        .unwrap_or_else(|| "gpt-5.2-2025-12-11".to_string());
    let client = OpenAIClient::new(api_key, base_url, model);

    // Build initial messages: system prompt + document content as user message
    let initial_messages = vec![
        json!({"role": "system", "content": system_prompt}),
        json!({"role": "user", "content": format!(
            "Please process the following document for ingestion.\n\nTitle: {title}\nSource ID: {source_id}\nSource: {}\nTags: {}\n\n---\n\n{file_content}",
            options.source.as_deref().unwrap_or("unknown"),
            options.tags.join(", "),
        )}),
    ];

    let config = AgentConfig {
        vault_path: vault.clone(),
        thread_path: thread_path.clone(),
        max_turns: 20,
        allow_commit: false,
    };

    println!("Running ingestion agent...");
    let _final_messages = run_agent_loop(&config, initial_messages, &client)?;

    // Update processing status to complete
    update_processing_status(&source_path, "complete")?;

    // Count proposals created
    let proposal_count = count_new_proposals(&vault);

    // Embed (best effort)
    let summary_path = vault.join("summaries/sources").join(format!("{slug}.md"));
    if let Ok(embed_client) = EmbeddingClient::from_env() {
        match build_knowledge_index(&vault, &embed_client) {
            Ok(stats) => println!("Re-indexed: {} docs / {} chunks", stats.doc_count, stats.chunk_count),
            Err(e) => eprintln!("Warning: embedding failed: {e}"),
        }
    }

    // Git commit (best effort)
    let repo_root = PathBuf::from(".");
    let msg = format!(
        "ingest: {} from {}",
        slug,
        options.source.as_deref().unwrap_or("cli")
    );
    let mut files_to_commit = vec![source_path.clone()];
    if summary_path.exists() {
        files_to_commit.push(summary_path.clone());
    }
    let ledger = vault.join("audit/ledger.jsonl");
    if ledger.exists() {
        files_to_commit.push(ledger);
    }
    if let Err(e) = git_commit(&repo_root, &files_to_commit, &msg) {
        eprintln!("Warning: git commit failed: {e}");
    }

    Ok(IngestResult {
        source_path,
        summary_path,
        thread_path,
        proposal_count,
    })
}

fn slugify(input: &str) -> String {
    let mut slug: String = input
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    // Collapse consecutive hyphens
    while slug.contains("--") {
        slug = slug.replace("--", "-");
    }
    slug.trim_matches('-').to_string()
}

fn unique_path(dir: &Path, slug: &str, ext: &str) -> PathBuf {
    let candidate = dir.join(format!("{slug}.{ext}"));
    if !candidate.exists() {
        return candidate;
    }
    // Collision: append ULID suffix
    dir.join(format!("{slug}-{}.{ext}", Ulid::new()))
}

fn render_source_file(fm: &SourceFrontMatter, original_content: &str) -> Result<String> {
    let yaml = serde_yaml::to_string(fm)?;
    let yaml = yaml.trim_start_matches("---\n").trim_start_matches("---");
    let mut out = String::new();
    out.push_str("---\n");
    out.push_str(yaml);
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("---\n");
    out.push_str(original_content);
    Ok(out)
}

fn load_ingest_prompt(vault: &Path, slug: &str, source_id: &str) -> Result<String> {
    let prompt_path = vault.join("prompts/ingest.system.md");
    let base = if prompt_path.exists() {
        fs::read_to_string(&prompt_path)?
    } else {
        default_ingest_prompt()
    };
    // Template in the slug and source_id
    Ok(base
        .replace("{slug}", slug)
        .replace("{source_id}", source_id))
}

fn default_ingest_prompt() -> String {
    concat!(
        "You are JJ's ingestion agent. You have been given an external document to process.\n\n",
        "Your tasks:\n",
        "1. Read and understand the document thoroughly.\n",
        "2. Search existing knowledge for related content using knowledge_search.\n",
        "3. Write a concise summary (200-500 words) to summaries/sources/{slug}.md using knowledge_apply.\n",
        "4. Extract discrete knowledge items and create them using knowledge_apply:\n",
        "   - People mentioned -> knowledge/people/<name>.md\n",
        "   - Projects described -> knowledge/projects/<name>.md\n",
        "   - Preferences stated -> knowledge/prefs/<name>.md\n",
        "   - System facts -> knowledge/system/<name>.md\n",
        "5. For each extraction, search existing knowledge first to avoid duplicates or to supersede existing docs.\n\n",
        "## knowledge_apply patch format\n\n",
        "The patch object supports: doc_path (required), title (required for new), type (required for new),\n",
        "status, confidence (0-1), tags_add, body_append, sources_add, supersedes_add.\n\n",
        "IMPORTANT: body_append is how you write body content. Without it, the doc will have an empty body.\n",
        "Always include body_append with meaningful markdown content for every knowledge_apply call.\n\n",
        "Every knowledge_apply call needs:\n",
        "- author: \"ingest-agent\"\n",
        "- reason: explain why this knowledge is being extracted\n",
        "- patch.body_append: the actual markdown content (NEVER omit this)\n\n",
        "Follow the invariants. Every write needs a reason and source references.",
    )
        .to_string()
}

fn update_processing_status(source_path: &Path, status: &str) -> Result<()> {
    let content = fs::read_to_string(source_path)?;
    let updated = content.replace(
        "processing_status: processing",
        &format!("processing_status: {status}"),
    );
    fs::write(source_path, updated)?;
    Ok(())
}

fn count_new_proposals(vault: &Path) -> usize {
    let proposals_dir = vault.join("inbox/proposals");
    if !proposals_dir.exists() {
        return 0;
    }
    fs::read_dir(&proposals_dir)
        .map(|entries| entries.filter_map(|e| e.ok()).count())
        .unwrap_or(0)
}
