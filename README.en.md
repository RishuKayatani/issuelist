# issuelist

A TUI CLI that lists GitHub issues with preview. It shows open issues for the Git repo in the current directory and previews the body and comments.

## Features
- List open issues (oldest first)
- Preview issue body + comments
- Cache for fast startup (`.issuelist/`)
- Async fetch so UI does not block
- Focus switching, horizontal scroll, and wrap toggle

## Requirements
- Rust 1.70+
- GitHub CLI `gh` (authenticated)

```bash
gh auth login
```

## Install (after publish)

```bash
cargo install issuelist
```

## Usage

```bash
issuelist
```

Run inside a Git repository to show its open issues.

## Keybindings

- `Enter`: Switch focus (List ⇄ Preview)
- `↑`/`↓`: Move selection (List focus) / vertical scroll (Preview focus)
- `j`/`k`: Same as above
- `h`/`l` or `←`/`→`: Horizontal scroll (focused pane)
- `f`/`b`: Page down/up
- `g`/`G`: Top/bottom
- `w`: Toggle wrap
- `r`/`F5`: Reload
- `q`: Quit

## Cache

- Issue list cache: `/.issuelist/issues.json`
- Detail cache: `/.issuelist/details/<number>.json`

## Disclaimer

This tool depends on GitHub CLI. Network/auth failures may prevent fetching.
