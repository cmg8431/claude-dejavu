use std::path::{Path, PathBuf};

/// Memory hierarchy targets, inspired by claude-reflect's multi-target system.
///
/// Priority order:
/// 1. .claude/rules/*.md — modular, path-scoped rules
/// 2. CLAUDE.md — project root
/// 3. CLAUDE.local.md — personal, gitignored
/// 4. ~/.claude/CLAUDE.md — global
/// 5. AGENTS.md — industry standard
#[derive(Debug, Clone)]
pub struct RuleTarget {
    pub path: PathBuf,
    pub target_type: TargetType,
    pub exists: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TargetType {
    /// .claude/rules/*.md — modular rules with optional path-scoping
    Rules,
    /// CLAUDE.md — project-level
    ProjectClaudeMd,
    /// CLAUDE.local.md — personal, gitignored
    LocalClaudeMd,
    /// ~/.claude/CLAUDE.md — global
    GlobalClaudeMd,
    /// AGENTS.md — industry standard
    AgentsMd,
}

impl TargetType {
    pub fn priority(&self) -> u8 {
        match self {
            Self::Rules => 1,
            Self::ProjectClaudeMd => 2,
            Self::LocalClaudeMd => 3,
            Self::GlobalClaudeMd => 4,
            Self::AgentsMd => 5,
        }
    }
}

/// Discover all available rule targets in a project.
pub fn discover_targets(project_path: &Path) -> Vec<RuleTarget> {
    let mut targets = Vec::new();

    // 1. .claude/rules/ directory
    let rules_dir = project_path.join(".claude").join("rules");
    if rules_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&rules_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "md") {
                    targets.push(RuleTarget {
                        path,
                        target_type: TargetType::Rules,
                        exists: true,
                    });
                }
            }
        }
    }
    // Always offer to create new rules
    targets.push(RuleTarget {
        path: rules_dir.join("dejavu.md"),
        target_type: TargetType::Rules,
        exists: rules_dir.join("dejavu.md").exists(),
    });

    // 2. CLAUDE.md
    let claude_md = project_path.join("CLAUDE.md");
    targets.push(RuleTarget {
        path: claude_md.clone(),
        target_type: TargetType::ProjectClaudeMd,
        exists: claude_md.exists(),
    });

    // 3. CLAUDE.local.md
    let local_md = project_path.join("CLAUDE.local.md");
    targets.push(RuleTarget {
        path: local_md.clone(),
        target_type: TargetType::LocalClaudeMd,
        exists: local_md.exists(),
    });

    // 4. ~/.claude/CLAUDE.md (global)
    if let Some(home) = dirs::home_dir() {
        let global_md = home.join(".claude").join("CLAUDE.md");
        targets.push(RuleTarget {
            path: global_md.clone(),
            target_type: TargetType::GlobalClaudeMd,
            exists: global_md.exists(),
        });
    }

    // 5. AGENTS.md
    let agents_md = project_path.join("AGENTS.md");
    targets.push(RuleTarget {
        path: agents_md.clone(),
        target_type: TargetType::AgentsMd,
        exists: agents_md.exists(),
    });

    targets.sort_by_key(|t| t.target_type.priority());
    targets
}

/// Determine the best target for a given rule scope.
pub fn best_target_for_scope(
    targets: &[RuleTarget],
    scope: RuleScope,
) -> Option<&RuleTarget> {
    match scope {
        RuleScope::Global => targets
            .iter()
            .find(|t| t.target_type == TargetType::GlobalClaudeMd),
        RuleScope::Project => targets
            .iter()
            .find(|t| t.target_type == TargetType::ProjectClaudeMd)
            .or_else(|| targets.iter().find(|t| t.target_type == TargetType::Rules)),
        RuleScope::File(_) => targets
            .iter()
            .find(|t| t.target_type == TargetType::Rules),
        RuleScope::Personal => targets
            .iter()
            .find(|t| t.target_type == TargetType::LocalClaudeMd),
    }
}

#[derive(Debug, Clone)]
pub enum RuleScope {
    /// Applies to all projects
    Global,
    /// Applies to this project
    Project,
    /// Applies to specific file(s)
    File(Vec<String>),
    /// Personal preference, not shared
    Personal,
}

/// Determine the scope of a detected pattern based on evidence.
pub fn infer_scope(
    detection: &crate::detector::Detection,
    project_count: usize,
) -> RuleScope {
    // If the same pattern appears across multiple projects → global
    if project_count > 1 {
        return RuleScope::Global;
    }

    // If all evidence points to specific files → file-scoped
    if !detection.evidence.file_paths.is_empty() && detection.evidence.file_paths.len() <= 3 {
        return RuleScope::File(detection.evidence.file_paths.clone());
    }

    // Default: project-scoped
    RuleScope::Project
}
