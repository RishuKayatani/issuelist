# issuelist

[English README](README.en.md)

GitHub Issue を TUI で一覧・プレビューする CLI ツールです。現在のディレクトリの Git リポジトリに紐づく Open Issue を表示し、本文とコメントをプレビューできます。

## 特徴
- Open Issue 一覧表示（作成日時の古い順）
- 本文 + コメントのプレビュー
- キャッシュによる高速表示（`.issuelist/`）
- 非同期取得で UI が固まらない
- フォーカス切替と横スクロール/折り返し

## 必要条件
- Rust 1.70+
- GitHub CLI `gh`（認証済み）

```bash
gh auth login
```

## インストール（公開後）

```bash
cargo install issuelist
```

## 使い方

```bash
issuelist
```

Git 管理されているディレクトリで実行すると、対応する GitHub リポジトリの Open Issue が表示されます。

## キー操作

- `Enter`: フォーカス切替（List ⇄ Preview）
- `↑`/`↓`: List フォーカス時は選択移動 / Preview フォーカス時は縦スクロール
- `j`/`k`: 上下スクロール（フォーカスに応じて動作）
- `h`/`l` or `←`/`→`: 横スクロール（フォーカス側のみ）
- `f`/`b`: ページ送り/戻り
- `g`/`G`: 先頭/末尾へ
- `w`: 折り返し ON/OFF
- `r`/`F5`: リロード
- `q`: 終了

## キャッシュ

- 一覧キャッシュ: `/.issuelist/issues.json`
- 詳細キャッシュ: `/.issuelist/details/<number>.json`

## 免責

本ツールは GitHub CLI に依存します。認証やネットワークの状態により取得に失敗する場合があります。
