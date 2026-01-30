use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn git_commit(repo_root: &Path, files: &[PathBuf], message: &str) -> Result<()> {
    let mut add = Command::new("git");
    add.arg("-C").arg(repo_root).arg("add");
    for file in files {
        add.arg(file);
    }
    let status = add.status().context("git add")?;
    if !status.success() {
        return Err(anyhow!("git add failed"));
    }
    let status = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("commit")
        .arg("-m")
        .arg(message)
        .status()
        .context("git commit")?;
    if !status.success() {
        return Err(anyhow!("git commit failed"));
    }
    Ok(())
}
