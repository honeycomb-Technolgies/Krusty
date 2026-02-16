//! Lightweight git helpers shared by server and clients.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::Stdio;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Context, Result};
use once_cell::sync::Lazy;
use regex::Regex;

static PR_CACHE: Lazy<Mutex<HashMap<String, (Instant, Option<u64>)>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static GH_AVAILABLE: Lazy<bool> = Lazy::new(|| {
    Command::new("gh")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
});
const PR_CACHE_TTL: Duration = Duration::from_secs(60);
const PR_CACHE_MAX_ENTRIES: usize = 1024;

/// Condensed repository status for UI surfaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitStatusSummary {
    pub repo_root: PathBuf,
    pub branch: Option<String>,
    pub head: Option<String>,
    pub upstream: Option<String>,
    /// Files changed in branch diff (typically merge-base..HEAD).
    pub branch_files: usize,
    /// Added lines in branch diff.
    pub branch_additions: usize,
    /// Deleted lines in branch diff.
    pub branch_deletions: usize,
    /// Current branch PR number (if discoverable).
    pub pr_number: Option<u64>,
    pub ahead: usize,
    pub behind: usize,
    pub staged: usize,
    pub modified: usize,
    pub untracked: usize,
    pub conflicted: usize,
}

impl GitStatusSummary {
    pub fn total_changes(&self) -> usize {
        self.staged + self.modified + self.untracked + self.conflicted
    }
}

/// Local branch metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitBranchSummary {
    pub name: String,
    pub is_current: bool,
    pub upstream: Option<String>,
}

/// Worktree metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitWorktreeSummary {
    pub path: PathBuf,
    pub branch: Option<String>,
    pub head: Option<String>,
    pub is_current: bool,
}

/// Resolve the current worktree root for a path, or `None` if path is not inside a git repo.
pub fn resolve_repo_root(path: &Path) -> Result<Option<PathBuf>> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(path)
        .output()
        .with_context(|| format!("Failed to run git in {}", path.display()))?;

    if output.status.success() {
        let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if root.is_empty() {
            return Ok(None);
        }
        return Ok(Some(PathBuf::from(root)));
    }

    let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
    if stderr.contains("not a git repository") {
        return Ok(None);
    }

    let detail = command_error_detail(&output.stdout, &output.stderr);
    Err(anyhow!("git rev-parse failed: {}", detail))
}

/// Get repository status for a given path.
pub fn status(path: &Path) -> Result<Option<GitStatusSummary>> {
    let repo_root = match resolve_repo_root(path)? {
        Some(root) => root,
        None => return Ok(None),
    };

    let output = run_git(
        &[
            "status",
            "--porcelain=2",
            "--branch",
            "--untracked-files=all",
        ],
        &repo_root,
    )?;
    let mut status = parse_status_output(repo_root, &String::from_utf8_lossy(&output.stdout));
    if let Some(diff) = compute_branch_diff_summary(&status.repo_root, status.upstream.as_deref()) {
        status.branch_files = diff.files;
        status.branch_additions = diff.additions;
        status.branch_deletions = diff.deletions;
    }
    status.pr_number = resolve_pr_number(&status.repo_root, status.branch.as_deref());

    Ok(Some(status))
}

/// List local branches for a repository path.
pub fn branches(path: &Path) -> Result<Option<Vec<GitBranchSummary>>> {
    let repo_root = match resolve_repo_root(path)? {
        Some(root) => root,
        None => return Ok(None),
    };

    let output = run_git(
        &[
            "for-each-ref",
            "--format=%(refname:short)%09%(HEAD)%09%(upstream:short)",
            "refs/heads",
        ],
        &repo_root,
    )?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut branches = Vec::new();
    for line in stdout.lines().filter(|line| !line.trim().is_empty()) {
        branches.push(parse_branch_summary_line(line));
    }

    branches.sort_by(|a, b| b.is_current.cmp(&a.is_current).then(a.name.cmp(&b.name)));
    Ok(Some(branches))
}

fn parse_branch_summary_line(line: &str) -> GitBranchSummary {
    let mut parts = line.split('\t');
    let name = parts.next().unwrap_or_default().to_string();
    let is_current = parts.next().unwrap_or_default().trim() == "*";
    let upstream = parts.next().and_then(|u| {
        let trimmed = u.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    });

    GitBranchSummary {
        name,
        is_current,
        upstream,
    }
}

/// List worktrees for a repository path.
pub fn worktrees(path: &Path) -> Result<Option<Vec<GitWorktreeSummary>>> {
    let repo_root = match resolve_repo_root(path)? {
        Some(root) => root,
        None => return Ok(None),
    };

    let output = run_git(&["worktree", "list", "--porcelain"], &repo_root)?;
    let mut worktrees = parse_worktree_output(&String::from_utf8_lossy(&output.stdout));

    let current_root = repo_root.canonicalize().unwrap_or(repo_root);
    for wt in &mut worktrees {
        let wt_path = wt.path.canonicalize().unwrap_or_else(|_| wt.path.clone());
        wt.is_current = wt_path == current_root;
    }

    worktrees.sort_by(|a, b| b.is_current.cmp(&a.is_current).then(a.path.cmp(&b.path)));
    Ok(Some(worktrees))
}

/// Checkout or create a branch in the repository at `path`.
pub fn checkout(path: &Path, branch: &str, create: bool, start_point: Option<&str>) -> Result<()> {
    let repo_root = resolve_repo_root(path)?
        .ok_or_else(|| anyhow!("Path is not inside a git repository: {}", path.display()))?;

    let branch = branch.trim();
    if branch.is_empty() {
        bail!("Branch name cannot be empty");
    }

    let mut args = vec!["checkout"];
    if create {
        args.push("-b");
        args.push(branch);
        if let Some(start_point) = start_point.map(str::trim).filter(|s| !s.is_empty()) {
            args.push(start_point);
        }
    } else {
        args.push(branch);
    }

    let _ = run_git(&args, &repo_root)?;
    Ok(())
}

fn run_git(args: &[&str], cwd: &Path) -> Result<std::process::Output> {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .with_context(|| {
            format!(
                "Failed to execute git {} in {}",
                args.join(" "),
                cwd.display()
            )
        })?;

    if output.status.success() {
        Ok(output)
    } else {
        let detail = command_error_detail(&output.stdout, &output.stderr);
        Err(anyhow!("git {} failed: {}", args.join(" "), detail))
    }
}

fn command_error_detail(stdout: &[u8], stderr: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr).trim().to_string();
    if !stderr.is_empty() {
        return stderr;
    }
    let stdout = String::from_utf8_lossy(stdout).trim().to_string();
    if !stdout.is_empty() {
        return stdout;
    }
    "unknown git error".to_string()
}

fn parse_status_output(repo_root: PathBuf, output: &str) -> GitStatusSummary {
    let mut status = GitStatusSummary {
        repo_root,
        branch: None,
        head: None,
        upstream: None,
        branch_files: 0,
        branch_additions: 0,
        branch_deletions: 0,
        pr_number: None,
        ahead: 0,
        behind: 0,
        staged: 0,
        modified: 0,
        untracked: 0,
        conflicted: 0,
    };

    for line in output.lines() {
        if let Some(rest) = line.strip_prefix("# ") {
            if let Some(head) = rest.strip_prefix("branch.head ") {
                let head = head.trim();
                if head != "(detached)" && head != "(unknown)" {
                    status.branch = Some(head.to_string());
                }
                continue;
            }

            if let Some(oid) = rest.strip_prefix("branch.oid ") {
                let oid = oid.trim();
                if oid != "(initial)" && !oid.is_empty() {
                    status.head = Some(short_sha(oid));
                }
                continue;
            }

            if let Some(upstream) = rest.strip_prefix("branch.upstream ") {
                let upstream = upstream.trim();
                if !upstream.is_empty() {
                    status.upstream = Some(upstream.to_string());
                }
                continue;
            }

            if let Some(ab) = rest.strip_prefix("branch.ab ") {
                for part in ab.split_whitespace() {
                    if let Some(ahead) = part.strip_prefix('+') {
                        status.ahead = ahead.parse::<usize>().unwrap_or(0);
                    } else if let Some(behind) = part.strip_prefix('-') {
                        status.behind = behind.parse::<usize>().unwrap_or(0);
                    }
                }
            }
            continue;
        }

        if line.starts_with("1 ") || line.starts_with("2 ") {
            let xy = line.split_whitespace().nth(1).unwrap_or("..");
            let mut chars = xy.chars();
            let x = chars.next().unwrap_or('.');
            let y = chars.next().unwrap_or('.');

            if x != '.' {
                status.staged += 1;
            }
            if y != '.' {
                status.modified += 1;
            }
            continue;
        }

        if line.starts_with("u ") {
            status.conflicted += 1;
            continue;
        }

        if line.starts_with("? ") {
            status.untracked += 1;
        }
    }

    status
}

fn parse_worktree_output(output: &str) -> Vec<GitWorktreeSummary> {
    let mut result = Vec::new();
    let mut current: Option<GitWorktreeSummary> = None;

    for line in output.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            if let Some(prev) = current.take() {
                result.push(prev);
            }
            current = Some(GitWorktreeSummary {
                path: PathBuf::from(path.trim()),
                branch: None,
                head: None,
                is_current: false,
            });
            continue;
        }

        if line.is_empty() {
            if let Some(prev) = current.take() {
                result.push(prev);
            }
            continue;
        }

        let Some(ref mut wt) = current else {
            continue;
        };

        if let Some(head) = line.strip_prefix("HEAD ") {
            wt.head = Some(short_sha(head.trim()));
            continue;
        }

        if let Some(branch_ref) = line.strip_prefix("branch ") {
            let short = branch_ref
                .trim()
                .strip_prefix("refs/heads/")
                .unwrap_or(branch_ref.trim());
            wt.branch = Some(short.to_string());
            continue;
        }

        if line == "detached" {
            wt.branch = None;
        }
    }

    if let Some(prev) = current {
        result.push(prev);
    }

    result
}

fn short_sha(sha: &str) -> String {
    sha.chars().take(8).collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct BranchDiffSummary {
    files: usize,
    additions: usize,
    deletions: usize,
}

fn compute_branch_diff_summary(
    repo_root: &Path,
    upstream: Option<&str>,
) -> Option<BranchDiffSummary> {
    let base_ref = resolve_base_ref(repo_root, upstream)?;
    let merge_base_output = run_git(&["merge-base", "HEAD", base_ref.as_str()], repo_root).ok()?;
    let merge_base = String::from_utf8_lossy(&merge_base_output.stdout)
        .trim()
        .to_string();
    if merge_base.is_empty() {
        return None;
    }

    let range = format!("{}..HEAD", merge_base);
    let diff_output = run_git(&["diff", "--numstat", range.as_str()], repo_root).ok()?;
    let stdout = String::from_utf8_lossy(&diff_output.stdout);
    Some(parse_numstat(&stdout))
}

fn parse_numstat(output: &str) -> BranchDiffSummary {
    let mut summary = BranchDiffSummary::default();
    for line in output.lines().filter(|line| !line.trim().is_empty()) {
        let mut parts = line.split('\t');
        let added = parts.next().unwrap_or_default().trim();
        let deleted = parts.next().unwrap_or_default().trim();
        let path = parts.next().unwrap_or_default().trim();
        if path.is_empty() {
            continue;
        }

        summary.files += 1;
        if let Ok(v) = added.parse::<usize>() {
            summary.additions += v;
        }
        if let Ok(v) = deleted.parse::<usize>() {
            summary.deletions += v;
        }
    }
    summary
}

fn resolve_base_ref(repo_root: &Path, upstream: Option<&str>) -> Option<String> {
    if let Some(upstream) = upstream.filter(|u| !u.trim().is_empty()) {
        if ref_exists(repo_root, upstream) {
            return Some(upstream.to_string());
        }
    }

    for candidate in ["origin/main", "origin/master", "main", "master"] {
        if ref_exists(repo_root, candidate) {
            return Some(candidate.to_string());
        }
    }

    None
}

fn ref_exists(repo_root: &Path, reference: &str) -> bool {
    Command::new("git")
        .args(["rev-parse", "--verify", "--quiet", reference])
        .current_dir(repo_root)
        // `rev-parse --verify` prints the resolved object ID on success;
        // suppress it to avoid polluting TUI stdout.
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn resolve_pr_number(repo_root: &Path, branch: Option<&str>) -> Option<u64> {
    let branch = branch.unwrap_or_default();
    let cache_key = format!("{}::{}", repo_root.display(), branch);
    let now = Instant::now();

    if let Ok(mut cache) = PR_CACHE.lock() {
        if let Some((timestamp, value)) = cache.get(&cache_key) {
            if now.duration_since(*timestamp) < PR_CACHE_TTL {
                return *value;
            }
        }
        // Drop stale entry so cache doesn't grow indefinitely from inactive branches/repos.
        cache.remove(&cache_key);
    }

    let resolved =
        extract_pr_from_branch_name(branch).or_else(|| query_pr_number_from_gh(repo_root));

    if let Ok(mut cache) = PR_CACHE.lock() {
        if cache.len() >= PR_CACHE_MAX_ENTRIES {
            prune_pr_cache(&mut cache, now);
        }
        cache.insert(cache_key, (now, resolved));
    }

    resolved
}

fn prune_pr_cache(cache: &mut HashMap<String, (Instant, Option<u64>)>, now: Instant) {
    cache.retain(|_, (timestamp, _)| now.duration_since(*timestamp) < PR_CACHE_TTL);

    while cache.len() >= PR_CACHE_MAX_ENTRIES {
        let Some(oldest_key) = cache
            .iter()
            .min_by_key(|(_, (timestamp, _))| *timestamp)
            .map(|(key, _)| key.clone())
        else {
            break;
        };
        cache.remove(&oldest_key);
    }
}

fn extract_pr_from_branch_name(branch: &str) -> Option<u64> {
    if branch.trim().is_empty() {
        return None;
    }

    static PR_BRANCH_PATTERN: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(?i)(?:^|/)(?:pr|pull|pull-request)[/-]?(\d+)$")
            .expect("valid PR branch regex")
    });

    PR_BRANCH_PATTERN
        .captures(branch)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse::<u64>().ok())
}

fn query_pr_number_from_gh(repo_root: &Path) -> Option<u64> {
    if !*GH_AVAILABLE {
        return None;
    }

    let output = Command::new("gh")
        .args(["pr", "view", "--json", "number", "--jq", ".number"])
        .current_dir(repo_root)
        .env("GH_FORCE_TTY", "0")
        .env("NO_COLOR", "1")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<u64>()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_porcelain_v2_status_counts() {
        let output = "\
# branch.oid 0123456789abcdef\n\
# branch.head feature-x\n\
# branch.upstream origin/feature-x\n\
# branch.ab +2 -1\n\
1 M. N... 100644 100644 100644 abcdef0 abcdef0 src/main.rs\n\
1 .M N... 100644 100644 100644 abcdef0 abcdef0 README.md\n\
2 R. N... 100644 100644 100644 abcdef0 abcdef0 R100 old new\n\
u UU N... 100644 100644 100644 100644 abcdef0 abcdef0 abcdef0 conflicted.txt\n\
? new_file.rs\n";

        let status = parse_status_output(PathBuf::from("/tmp/repo"), output);
        assert_eq!(status.branch.as_deref(), Some("feature-x"));
        assert_eq!(status.upstream.as_deref(), Some("origin/feature-x"));
        assert_eq!(status.head.as_deref(), Some("01234567"));
        assert_eq!(status.branch_files, 0);
        assert_eq!(status.branch_additions, 0);
        assert_eq!(status.branch_deletions, 0);
        assert_eq!(status.pr_number, None);
        assert_eq!(status.ahead, 2);
        assert_eq!(status.behind, 1);
        assert_eq!(status.staged, 2);
        assert_eq!(status.modified, 1);
        assert_eq!(status.untracked, 1);
        assert_eq!(status.conflicted, 1);
        assert_eq!(status.total_changes(), 5);
    }

    #[test]
    fn parses_worktree_porcelain_output() {
        let output = "\
worktree /repo\n\
HEAD 0123456789abcdef\n\
branch refs/heads/main\n\
\n\
worktree /repo-feature\n\
HEAD fedcba9876543210\n\
branch refs/heads/feature-x\n";

        let worktrees = parse_worktree_output(output);
        assert_eq!(worktrees.len(), 2);
        assert_eq!(worktrees[0].path, PathBuf::from("/repo"));
        assert_eq!(worktrees[0].branch.as_deref(), Some("main"));
        assert_eq!(worktrees[0].head.as_deref(), Some("01234567"));
        assert_eq!(worktrees[1].branch.as_deref(), Some("feature-x"));
        assert_eq!(worktrees[1].head.as_deref(), Some("fedcba98"));
    }

    #[test]
    fn parses_numstat_branch_diff_summary() {
        let output = "\
10\t2\tsrc/main.rs\n\
4\t0\tsrc/lib.rs\n\
-\t-\tbinary.file\n";

        let summary = parse_numstat(output);
        assert_eq!(summary.files, 3);
        assert_eq!(summary.additions, 14);
        assert_eq!(summary.deletions, 2);
    }

    #[test]
    fn extracts_pr_number_from_branch_name() {
        assert_eq!(extract_pr_from_branch_name("pr-29"), Some(29));
        assert_eq!(extract_pr_from_branch_name("feature/pull/104"), Some(104));
        assert_eq!(extract_pr_from_branch_name("feature/new-ui"), None);
    }
}
