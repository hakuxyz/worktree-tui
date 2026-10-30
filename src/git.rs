use anyhow::{Context, Result};
use tokio::process::Command;

/// Async wrapper around the `git` CLI.
#[derive(Clone)]
pub struct Git {
    repo_path: std::path::PathBuf,
}

impl std::fmt::Debug for Git {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Git")
            .field("repo_path", &self.repo_path)
            .finish()
    }
}

impl Git {
    pub fn new(repo_path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            repo_path: repo_path.into(),
        }
    }

    /// Run a git command and capture stdout.
    async fn run(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("git")
            .current_dir(&self.repo_path)
            .args(args)
            .output()
            .await
            .with_context(|| format!("failed to run git {:?}", args))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("git {:?} failed: {}", args, stderr);
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// List all worktrees via `git worktree list --porcelain`.
    pub async fn worktree_list(&self) -> Result<Vec<crate::app::Worktree>> {
        let output = self.run(&["worktree", "list", "--porcelain"]).await?;
        Ok(parse_worktree_list(&output))
    }

    /// Add a new worktree via `git worktree add <path> <branch>`.
    pub async fn worktree_add(&self, path: &str, branch: &str) -> Result<()> {
        self.run(&["worktree", "add", path, branch]).await?;
        Ok(())
    }

    /// Remove a worktree via `git worktree remove <path>`.
    pub async fn worktree_remove(&self, path: &str) -> Result<()> {
        self.run(&["worktree", "remove", path]).await?;
        Ok(())
    }

    /// List local branches via `git branch --format`.
    pub async fn branch_list(&self) -> Result<Vec<String>> {
        let output = self
            .run(&["branch", "--format=%(refname:short)"])
            .await?;
        Ok(output
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect())
    }

    /// Create a new branch via `git branch <name>`.
    pub async fn branch_create(&self, name: &str) -> Result<()> {
        self.run(&["branch", name]).await?;
        Ok(())
    }

    /// Get current branch via `git branch --show-current`.
    pub async fn current_branch(&self) -> Result<String> {
        self.run(&["branch", "--show-current"]).await
    }

    /// Get git status short output via `git status --short --branch`.
    pub async fn status_short(&self) -> Result<String> {
        self.run(&["status", "--short", "--branch"]).await
    }
}

/// Parse `git worktree list --porcelain` output into Worktree structs.
fn parse_worktree_list(output: &str) -> Vec<crate::app::Worktree> {
    let mut worktrees = Vec::new();
    let mut current: Option<crate::app::Worktree> = None;

    for line in output.lines() {
        let line = line.trim();

        if line.is_empty() {
            // End of a worktree block.
            if let Some(wt) = current.take() {
                worktrees.push(wt);
            }
            continue;
        }

        if line.starts_with("worktree ") {
            // Save previous worktree before starting a new one.
            if let Some(wt) = current.take() {
                worktrees.push(wt);
            }
            let path = line.strip_prefix("worktree ").unwrap_or("").to_string();
            current = Some(crate::app::Worktree {
                id: path.clone(),
                path,
                branch: String::new(),
                commit: String::new(),
                is_locked: false,
                is_main: false,
                commit_date: None,
                ahead: 0,
                behind: 0,
            });
            continue;
        }

        let Some(wt) = current.as_mut() else {
            continue;
        };

        if let Some(branch) = line.strip_prefix("HEAD ") {
            wt.branch = branch.to_string();
        } else if let Some(branch) = line.strip_prefix("branch ") {
            // `refs/heads/feature/auth` -> `feature/auth`
            let b = branch.strip_prefix("refs/heads/").unwrap_or(branch);
            wt.branch = b.to_string();
        } else if let Some(commit) = line.strip_prefix("commit ") {
            wt.commit = commit.to_string();
        } else if line == "locked" {
            wt.is_locked = true;
        }
    }

    // Don't forget the last block.
    if let Some(wt) = current {
        worktrees.push(wt);
    }

    worktrees
}
