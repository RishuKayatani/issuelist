use std::process::Command;

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct Issue {
    pub number: u64,
    pub title: String,
    pub created_at: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct IssueAuthor {
    pub login: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct CommentAuthor {
    pub login: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct Comment {
    pub user: CommentAuthor,
    pub body: String,
    pub created_at: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct IssueDetail {
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub user: IssueAuthor,
    pub created_at: String,
    pub comments: Vec<Comment>,
}

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct IssueDetailRaw {
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub user: IssueAuthor,
    pub created_at: String,
    pub comments: u64,
}

impl IssueDetailRaw {
    fn into_detail(self, comments: Vec<Comment>) -> IssueDetail {
        IssueDetail {
            number: self.number,
            title: self.title,
            body: self.body,
            user: self.user,
            created_at: self.created_at,
            comments,
        }
    }
}

pub fn fetch_open_issues(owner: &str, repo_name: &str) -> anyhow::Result<Vec<Issue>> {
    let endpoint = format!(
        "repos/{owner}/{repo_name}/issues?state=open&per_page=100&sort=created&direction=asc",
    );
    let mut issues: Vec<Issue> = gh_json(&endpoint)?;
    issues.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Ok(issues)
}


pub fn fetch_issue_detail(owner: &str, repo_name: &str, number: u64) -> anyhow::Result<IssueDetail> {
    let endpoint = format!("repos/{owner}/{repo_name}/issues/{number}");
    let detail: IssueDetailRaw = gh_json(&endpoint)?;
    let comments = fetch_issue_comments(owner, repo_name, number)?;
    Ok(detail.into_detail(comments))
}

pub fn fetch_issue_comments(
    owner: &str,
    repo_name: &str,
    number: u64,
) -> anyhow::Result<Vec<Comment>> {
    let endpoint = format!(
        "repos/{owner}/{repo_name}/issues/{number}/comments?sort=created&direction=asc&per_page=100",
    );
    gh_json(&endpoint)
}

fn gh_json<T: serde::de::DeserializeOwned>(endpoint: &str) -> anyhow::Result<T> {
    let output = Command::new("gh").args(["api", endpoint]).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let msg = stderr.trim();
        let msg = if msg.is_empty() {
            "gh api failed"
        } else {
            msg
        };
        return Err(anyhow::anyhow!(msg.to_string()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let data = serde_json::from_str(stdout.trim())?;
    Ok(data)
}
