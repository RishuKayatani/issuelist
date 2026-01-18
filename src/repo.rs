use std::path::PathBuf;
use std::process::Command;

pub fn git_root() -> anyhow::Result<PathBuf> {
    let output = run_git(["rev-parse", "--show-toplevel"])?;
    Ok(PathBuf::from(output))
}

pub fn origin_url() -> anyhow::Result<String> {
    run_git(["remote", "get-url", "origin"])
}

pub fn parse_owner_repo(url: &str) -> anyhow::Result<(String, String)> {
    let trimmed = url.trim();

    let path = if let Some(idx) = trimmed.find("github.com") {
        let after = &trimmed[idx + "github.com".len()..];
        if let Some(after) = after.strip_prefix(':') {
            after
        } else {
            after.strip_prefix('/').unwrap_or(after)
        }
    } else {
        return Err(anyhow::anyhow!("Unsupported remote URL: {}", url));
    };

    let path = path.trim_start_matches('/');
    let path = path.trim_end_matches(".git");
    let mut parts = path.split('/').filter(|p| !p.is_empty());

    let owner = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("Missing owner in URL: {}", url))?;
    let repo = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("Missing repo in URL: {}", url))?;

    Ok((owner.to_string(), repo.to_string()))
}

pub fn current_repo() -> anyhow::Result<(String, String)> {
    let _root = git_root()?;
    let url = origin_url()?;
    parse_owner_repo(&url)
}

fn run_git<const N: usize>(args: [&str; N]) -> anyhow::Result<String> {
    let output = Command::new("git").args(args).output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let msg = stderr.trim();
        let msg = if msg.is_empty() {
            "git command failed"
        } else {
            msg
        };
        return Err(anyhow::anyhow!(msg.to_string()));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::parse_owner_repo;

    #[test]
    fn parse_https_url() {
        let (owner, repo) = parse_owner_repo("https://github.com/octo/hello.git").unwrap();
        assert_eq!(owner, "octo");
        assert_eq!(repo, "hello");
    }

    #[test]
    fn parse_https_url_without_git_suffix() {
        let (owner, repo) = parse_owner_repo("https://github.com/octo/hello").unwrap();
        assert_eq!(owner, "octo");
        assert_eq!(repo, "hello");
    }

    #[test]
    fn parse_ssh_url() {
        let (owner, repo) = parse_owner_repo("git@github.com:octo/hello.git").unwrap();
        assert_eq!(owner, "octo");
        assert_eq!(repo, "hello");
    }

    #[test]
    fn parse_ssh_url_with_scheme() {
        let (owner, repo) = parse_owner_repo("ssh://git@github.com/octo/hello.git").unwrap();
        assert_eq!(owner, "octo");
        assert_eq!(repo, "hello");
    }
}
