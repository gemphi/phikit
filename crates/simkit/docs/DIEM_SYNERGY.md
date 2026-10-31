---
id: simkit-diem
title: Diem Synergy
custom_edit_url: https://github.com/gemphi/phi/edit/main/crates/simkit/docs/DIEM_SYNERGY.md
---

# How Diem Helps SimKit

The `diem` root directory acts as the orchestration and platform layer for the workspace. Here is how it facilitates `SimKit` development:

## 1. Documentation Pipeline
All `.md` files in this `docs` folder utilize Diem's frontmatter standard (`id`, `title`, `custom_edit_url`). Diem's static site generators (likely Docusaurus based on this format) will automatically crawl these files and publish them to the workspace's documentation site, making them readable for the client and team.

## 2. CI/CD & Safety
During the heavy <200-line refactoring of `SimKit`:
- **GitHub Actions**: The `.github/workflows` in `diem` will automatically run `cargo check` and `cargo clippy` to ensure no logic is broken while splitting files.
- **Dependency Auditing**: `diem/.github/dep-auditors.json` ensures that new dependencies (if any are needed for UI data binding) are legally compliant and secure.

## 3. Git Hooks
Diem provides `.git/hooks` that enforce code formatting (`cargo fmt`) and the aforementioned workspace constraints prior to committing the refactored `SimKit` chunks.
