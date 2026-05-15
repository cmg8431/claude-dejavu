#![allow(clippy::collapsible_if)]
use super::{Detection, DetectorType, Evidence};
use std::path::Path;

/// Detector ⑥: Project Context Scanner
///
/// Reads project configuration files (lockfiles, tsconfig, etc.)
/// and generates rules based on project setup. No session history needed.
pub fn detect_from_project(project_path: &Path) -> Vec<Detection> {
    let mut detections = Vec::new();

    // Package manager detection
    if let Some(rule) = detect_package_manager(project_path) {
        detections.push(rule);
    }

    // Framework detection
    if let Some(rule) = detect_framework(project_path) {
        detections.push(rule);
    }

    // TypeScript config
    if let Some(rule) = detect_typescript(project_path) {
        detections.push(rule);
    }

    // Monorepo detection
    if let Some(rule) = detect_monorepo(project_path) {
        detections.push(rule);
    }

    // Linter/formatter
    if let Some(rule) = detect_linter(project_path) {
        detections.push(rule);
    }

    // Rust project
    if let Some(rule) = detect_rust(project_path) {
        detections.push(rule);
    }

    // Python project
    if let Some(rule) = detect_python(project_path) {
        detections.push(rule);
    }

    detections
}

fn make_detection(rule_text: &str, details_key: &str) -> Detection {
    Detection {
        detector_type: DetectorType::ProjectContext,
        evidence: Evidence {
            sessions: vec![],
            file_paths: vec![],
            occurrences: 1,
            details: serde_json::json!({ "source": details_key }),
        },
        confidence: 1.0,
        suggested_rule: rule_text.to_string(),
    }
}

fn detect_package_manager(path: &Path) -> Option<Detection> {
    if path.join("pnpm-lock.yaml").exists() {
        Some(make_detection(
            "This project uses pnpm. Run `pnpm install`, `pnpm add`, `pnpm run` — never npm or yarn.",
            "pnpm-lock.yaml",
        ))
    } else if path.join("yarn.lock").exists() {
        Some(make_detection(
            "This project uses yarn. Run `yarn install`, `yarn add`, `yarn run` — never npm.",
            "yarn.lock",
        ))
    } else if path.join("bun.lockb").exists() || path.join("bun.lock").exists() {
        Some(make_detection(
            "This project uses bun. Run `bun install`, `bun add`, `bun run` — never npm or yarn.",
            "bun.lockb",
        ))
    } else if path.join("package-lock.json").exists() {
        Some(make_detection(
            "This project uses npm. Use `npm install`, `npm run`.",
            "package-lock.json",
        ))
    } else {
        None
    }
}

fn detect_framework(path: &Path) -> Option<Detection> {
    // Next.js
    if path.join("next.config.js").exists()
        || path.join("next.config.mjs").exists()
        || path.join("next.config.ts").exists()
    {
        // Check for App Router vs Pages Router
        if path.join("app").exists() && path.join("app").join("layout.tsx").exists() {
            return Some(make_detection(
                "Next.js project using App Router. Use `app/` directory with server components by default. Use `'use client'` directive only when needed.",
                "next.config + app/layout.tsx",
            ));
        }
        if path.join("pages").exists() {
            return Some(make_detection(
                "Next.js project using Pages Router. Use `pages/` directory for routes.",
                "next.config + pages/",
            ));
        }
        return Some(make_detection("Next.js project detected.", "next.config"));
    }

    // Nuxt
    if path.join("nuxt.config.ts").exists() || path.join("nuxt.config.js").exists() {
        return Some(make_detection(
            "Nuxt.js project. Use `pages/` for routes, `composables/` for shared logic.",
            "nuxt.config",
        ));
    }

    // Vue
    if path.join("vue.config.js").exists() || path.join("vite.config.ts").exists() {
        if let Ok(content) = std::fs::read_to_string(path.join("package.json")) {
            if content.contains("\"vue\"") {
                return Some(make_detection(
                    "Vue.js project. Use Composition API with `<script setup>` syntax.",
                    "vue in package.json",
                ));
            }
        }
    }

    // React (generic)
    if let Ok(content) = std::fs::read_to_string(path.join("package.json")) {
        if content.contains("\"react\"") && !content.contains("\"next\"") {
            return Some(make_detection(
                "React project. Use functional components with hooks.",
                "react in package.json",
            ));
        }
    }

    None
}

fn detect_typescript(path: &Path) -> Option<Detection> {
    let tsconfig_path = path.join("tsconfig.json");
    if !tsconfig_path.exists() {
        return None;
    }

    if let Ok(content) = std::fs::read_to_string(&tsconfig_path) {
        if content.contains("\"strict\": true") || content.contains("\"strict\":true") {
            return Some(make_detection(
                "TypeScript strict mode enabled. All function parameters need explicit types. No implicit any.",
                "tsconfig.json strict:true",
            ));
        }
        return Some(make_detection(
            "TypeScript project. Follow existing type patterns in the codebase.",
            "tsconfig.json",
        ));
    }

    None
}

fn detect_monorepo(path: &Path) -> Option<Detection> {
    // pnpm workspaces
    if path.join("pnpm-workspace.yaml").exists() {
        return Some(make_detection(
            "pnpm monorepo workspace. Run commands from the specific package directory, not the repo root. Use `pnpm --filter <package>` for targeted operations.",
            "pnpm-workspace.yaml",
        ));
    }

    // npm/yarn workspaces in package.json
    if let Ok(content) = std::fs::read_to_string(path.join("package.json")) {
        if content.contains("\"workspaces\"") {
            return Some(make_detection(
                "Monorepo with workspaces. Run commands from the specific package directory, not the repo root.",
                "workspaces in package.json",
            ));
        }
    }

    // Cargo workspace
    if let Ok(content) = std::fs::read_to_string(path.join("Cargo.toml")) {
        if content.contains("[workspace]") {
            return Some(make_detection(
                "Cargo workspace. Use `-p <crate>` to target specific crates. Run `cargo check --workspace` to verify all crates.",
                "Cargo.toml [workspace]",
            ));
        }
    }

    None
}

fn detect_linter(path: &Path) -> Option<Detection> {
    if path.join(".eslintrc.js").exists()
        || path.join(".eslintrc.json").exists()
        || path.join("eslint.config.js").exists()
        || path.join("eslint.config.mjs").exists()
    {
        // Check for prettier too
        let has_prettier = path.join(".prettierrc").exists()
            || path.join(".prettierrc.json").exists()
            || path.join("prettier.config.js").exists();

        if has_prettier {
            return Some(make_detection(
                "ESLint + Prettier configured. Code must pass both lint and format checks. Run lint before committing.",
                "eslintrc + prettierrc",
            ));
        }

        return Some(make_detection(
            "ESLint configured. Run lint check before committing changes.",
            "eslintrc",
        ));
    }

    if path.join("biome.json").exists() || path.join("biome.jsonc").exists() {
        return Some(make_detection(
            "Biome configured for linting and formatting. Run `biome check` before committing.",
            "biome.json",
        ));
    }

    None
}

fn detect_rust(path: &Path) -> Option<Detection> {
    if !path.join("Cargo.toml").exists() {
        return None;
    }

    // Already handled workspace in detect_monorepo
    if let Ok(content) = std::fs::read_to_string(path.join("Cargo.toml")) {
        if content.contains("[workspace]") {
            return None; // Handled by monorepo detector
        }
    }

    Some(make_detection(
        "Rust project. Run `cargo check` before committing. Use `cargo clippy` for linting.",
        "Cargo.toml",
    ))
}

fn detect_python(path: &Path) -> Option<Detection> {
    if path.join("pyproject.toml").exists() {
        if let Ok(content) = std::fs::read_to_string(path.join("pyproject.toml")) {
            if content.contains("[tool.poetry]") {
                return Some(make_detection(
                    "Python project using Poetry. Run `poetry install`, `poetry run` — never raw pip.",
                    "pyproject.toml + poetry",
                ));
            }
            if content.contains("uv") {
                return Some(make_detection(
                    "Python project using uv. Run `uv sync`, `uv run` for package management.",
                    "pyproject.toml + uv",
                ));
            }
        }
        return Some(make_detection(
            "Python project. Follow existing patterns in the codebase.",
            "pyproject.toml",
        ));
    }

    if path.join("requirements.txt").exists() {
        return Some(make_detection(
            "Python project with requirements.txt. Use `pip install -r requirements.txt`.",
            "requirements.txt",
        ));
    }

    None
}
