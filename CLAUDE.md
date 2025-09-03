# CLAUDE.md – Development Guidelines
- Ignore everything in the directory _wb
- All code should follow the requirements in docs-vibe/prd.md . Do not deviate unless the PRD is updated.
- Use Rust and the Axum crate for all backend code.
- Document backend REST API endpoints with OpenAPI, using the utoipa crate for Rust.
- Use Postgres with Docker and the SQLx Rust crate, for database persistence.
- Use React 18 with TypeScript for all frontend code.
- Use TailwindCSS for all styling; no inline styles.
- Use GitHub Actions for CI/CD