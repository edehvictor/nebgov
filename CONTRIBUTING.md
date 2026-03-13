# Contributing to NebGov

NebGov is an active open-source project participating in the **Stellar Wave Program** on [Drips Network](https://drips.network/wave). Contributors who solve issues and get PRs merged earn USDC rewards from the Wave reward pool.

## How to Contribute

### 1. Find an issue
Browse [open issues](https://github.com/nebgov/nebgov/issues). Each issue is tagged with:
- `complexity: trivial` — 100 Wave points
- `complexity: medium` — 150 Wave points
- `complexity: high` — 200 Wave points

Issues tagged `good first issue` are recommended for first-time contributors.

### 2. Apply via Drips
Visit [drips.network/wave](https://drips.network/wave), find the NebGov repo, and apply to the issue you want to work on. **Do not start work until your application is accepted.**

### 3. Fork and branch
```bash
git checkout -b feat/issue-<number>-<short-description>
```

### 4. Implement and test
- Rust contracts: `cargo test --workspace`
- SDK: `pnpm test:sdk`
- Frontend: `pnpm test:app`

### 5. Open a PR
- Reference the issue: `Closes #<number>`
- Describe what you changed and why
- All CI checks must pass

## Code Standards

### Rust (contracts)
- Follow standard Rust formatting: `cargo fmt`
- No unsafe code
- All public functions must have a doc comment
- Tests live in `#[cfg(test)]` modules within each contract

### TypeScript (SDK + frontend)
- Strict TypeScript — no `any` types
- Run `pnpm lint` before pushing

### Commit messages
Use imperative mood: `Add vote checkpointing to token-votes`, not `Added...`

## Issue Scope

Each issue is scoped to be completable in **under one week** by a single contributor. If you find that an issue is larger than expected, comment on it so it can be split.

## Questions?

Open a discussion on GitHub or join the [Stellar Wave Discord](https://discord.gg/t8XBXZAEs5).
