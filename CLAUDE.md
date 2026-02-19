# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`envrac-rust` is a Rust HTTP service that automates publishing weekly "En Vrac" blog posts. When triggered, it:
1. Fetches tasks from a Todoist project via the Todoist REST API
2. Fetches the two most recent articles from a GitHub-hosted Hugo blog
3. Generates a formatted Markdown article grouped by category
4. Pushes the new article to the GitHub repo via the GitHub Contents API

## Repository Structure

This repo contains two independent Cargo crates:

- **Root crate** (`envrac-rust`): Skeleton crate with no dependencies; integration tests live in `tests/`.
- **`run/` crate**: The actual application binary. All meaningful code is here.
  - `run/src/main.rs` — HTTP server (warp), endpoint handlers, article generation logic, API calls
  - `run/src/lib.rs` — Re-exports `Task` from models
  - `run/src/models/task.rs` — `Task` struct (Todoist task; `category` is populated post-deserialization via `post_deserialize()`)
  - `run/src/models/category.rs` — `Category` enum; Todoist section IDs are hardcoded and mapped to categories
  - `run/src/models/github_object.rs` — `GithubRequest`, `Author`, `Committer` structs for GitHub API

## Points d'attention API

- **Todoist API v1** : l'URL est `https://api.todoist.com/api/v1/tasks` et la réponse est enveloppée `{ "results": [...] }` — pas un tableau direct comme en v2.
- Le `project_id` Todoist est un identifiant alphanumérique (ex: `"6V79vpFpwHpQpRJm"`), pas numérique.

## Commands

All commands should be run from the `run/` directory unless otherwise noted.

```bash
# Build
cargo build

# Run locally (requires .env to be sourced or env vars set)
cargo run

# Run tests
cargo test

# Build Docker image and run via docker-compose (from repo root)
docker-compose up --build
```

## Environment Variables

The service reads these from the environment at startup (see `run/.env` for local values):

| Variable | Purpose |
|---|---|
| `TODOIST_API_TOKEN` | Todoist REST API bearer token |
| `GITHUB_API_TOKEN` | GitHub personal access token (needs repo write access) |
| `GITHUB_USER_AGENT` | User-Agent header for GitHub API requests |
| `EXECUTOR` | Identifier included in the commit message (e.g. `macbook`, `docker`) |
| `SECRET` | Simple shared secret for the HTTP endpoint |

## HTTP API

The server listens on port `3030`.

- `POST /en-vrac?secret=<SECRET>` — triggers article generation and publish
- `GET /healthcheck` — returns `"ok"`

## Key Architecture Notes

- **Category mapping**: Todoist section IDs are hardcoded in `category.rs::convert_to_category()`. To add a new category, add a new `Category` variant and map the corresponding Todoist section ID.
- **`PutAside` category**: Tasks in this category are excluded from the generated article. The filtering happens in `exclude_put_aside_category_tasks()` in `main.rs`.
- **Article format**: `create_head_of_article()` builds the Hugo frontmatter and intro; `create_body_of_article()` builds the grouped content sections.
- **`post_deserialize()`**: Because serde cannot directly deserialize `category` (it's derived from `section_id`), `Task::post_deserialize()` must be called manually after deserializing each task from the Todoist API response.
- **Docker port mapping**: `docker-compose.yml` maps host port `3030` → container port `8080`, but the Dockerfile `EXPOSE`s `3030` and the binary binds to `3030`. The compose port mapping may need adjustment.
