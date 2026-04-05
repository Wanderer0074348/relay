# Contributing to Relay

Thanks for your interest in contributing! Here's how to get started.

## Development Setup

```bash
# Clone
git clone https://github.com/Manavarya09/relay
cd relay

# Build
cd core && cargo build

# Run
./target/debug/relay --help

# Run tests
cargo test
```

## Adding a New Agent

1. Create `core/src/agents/youragent.rs`
2. Implement the `Agent` trait:

```rust
use super::Agent;
use crate::{AgentStatus, HandoffResult};

pub struct YourAgent;

impl Agent for YourAgent {
    fn name(&self) -> &str { "youragent" }
    fn check_available(&self) -> AgentStatus { /* ... */ }
    fn execute(&self, handoff_prompt: &str, project_dir: &str) -> Result<HandoffResult> { /* ... */ }
}
```

3. Add `pub mod youragent;` to `core/src/agents/mod.rs`
4. Register in `get_agents()` match block
5. Add to default priority in `lib.rs`

## Code Style

- `cargo fmt` before committing
- `cargo clippy` should pass
- Keep functions small and focused

## Submitting Changes

1. Fork the repo
2. Create a feature branch
3. Make your changes
4. Run `cargo check && cargo test`
5. Submit a PR with a clear description

## Reporting Bugs

Use the [bug report template](https://github.com/Manavarya09/relay/issues/new?template=bug_report.md).

## Feature Requests

Use the [feature request template](https://github.com/Manavarya09/relay/issues/new?template=feature_request.md).
