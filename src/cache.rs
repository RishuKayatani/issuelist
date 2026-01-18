use std::fs;
use std::path::{Path, PathBuf};

use crate::github::{Issue, IssueDetail};

const CACHE_DIR_NAME: &str = ".issuelist";
const ISSUES_FILE: &str = "issues.json";
const DETAIL_DIR: &str = "details";

#[derive(serde::Serialize, serde::Deserialize)]
struct IssuesCache {
    issues: Vec<Issue>,
}

pub fn cache_dir() -> anyhow::Result<PathBuf> {
    let root = std::env::current_dir()?;
    Ok(root.join(CACHE_DIR_NAME))
}

pub fn ensure_cache_dirs() -> anyhow::Result<PathBuf> {
    let dir = cache_dir()?;
    fs::create_dir_all(dir.join(DETAIL_DIR))?;
    Ok(dir)
}

pub fn write_issues(issues: &[Issue]) -> anyhow::Result<()> {
    let dir = ensure_cache_dirs()?;
    let cache = IssuesCache {
        issues: issues.to_vec(),
    };
    let json = serde_json::to_string_pretty(&cache)?;
    fs::write(dir.join(ISSUES_FILE), json)?;
    Ok(())
}

pub fn read_issues() -> anyhow::Result<Vec<Issue>> {
    let dir = cache_dir()?;
    let data = fs::read_to_string(dir.join(ISSUES_FILE))?;
    let cache: IssuesCache = serde_json::from_str(&data)?;
    Ok(cache.issues)
}

pub fn write_detail(number: u64, detail: &IssueDetail) -> anyhow::Result<()> {
    let dir = ensure_cache_dirs()?;
    let path = detail_path(&dir, number);
    let json = serde_json::to_string_pretty(detail)?;
    fs::write(path, json)?;
    Ok(())
}

pub fn read_detail(number: u64) -> anyhow::Result<IssueDetail> {
    let dir = cache_dir()?;
    let path = detail_path(&dir, number);
    let data = fs::read_to_string(path)?;
    let detail: IssueDetail = serde_json::from_str(&data)?;
    Ok(detail)
}

fn detail_path(root: &Path, number: u64) -> PathBuf {
    root.join(DETAIL_DIR).join(format!("{number}.json"))
}
