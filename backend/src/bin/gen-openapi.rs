//! Generate `docs/openapi.json` from the live router definition.
//! Used by `npm run openapi`.
//!
//! Usage:
//!   cargo run --manifest-path backend/Cargo.toml --bin gen-openapi -- [path]
//!
//! Default path is `docs/openapi.json` relative to the repo root.

use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let path: PathBuf = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            // backend/ is CWD when running with --manifest-path from repo root,
            // so default lives one level up under docs/.
            let mut p = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            if p.ends_with("backend") {
                p.pop();
            }
            p.push("docs");
            p.push("openapi.json");
            p
        });

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let spec = pubmed_backend::openapi();
    let json = spec.to_pretty_json()?;
    std::fs::write(&path, json)?;

    eprintln!("wrote {}", path.display());
    Ok(())
}
