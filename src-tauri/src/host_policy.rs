//! Per-host injection policy.
//!
//! Hosts differ in what synthetic input they accept: Orca's agent terminal
//! rejects ALL synthetic input — Direct (Enigo Unicode) included, verified
//! 2026-08-09 even as administrator — so only Copy + manual Ctrl+V works
//! there. This module decides the effective mode for the focused window so
//! the user doesn't have to flip settings every time they switch windows.
//!
//! Since v0.3.12 it also powers the `Auto` mode itself:
//! - `global_mode == Auto` — the policy decides everything: a matching host
//!   rule wins, otherwise Direct (plain terminals and VSCode-as-admin accept
//!   typing; the job layer's clipboard insurance covers unverifiable
//!   delivery, see P0 in docs/ISSUES_20260809.md).
//! - `global_mode == Direct/Copy` (explicit user choice) — a matching host
//!   rule still overrides (Orca can never accept Direct), otherwise the
//!   user's choice stands.
//!
//! This is a SEPARATE layer from `routing`: routing decides WHERE the file
//! goes; this decides HOW the path is injected. The `RouteResolver` trait
//! contract explicitly forbids injection-mode decisions, so host policy lives
//! here as a pure lookup called at injection time (`job::process_job`).
//!
//! Detection is window-title-substring only for now (cross-platform; reuses
//! `daemon::get_active_window_title`). Process-exe detection (Windows) is a
//! future robustness upgrade. Title matching is the same approach
//! `ManualRuleResolver` already uses for routing.
//!
//! Editorial rule (see docs/ISSUES_20260809.md): only add a host to the table
//! AFTER its behavior is confirmed by testing — record the finding as observed
//! behavior, not assumed mechanism.

use crate::config::InjectionMode;

/// A built-in host rule: case-insensitive title substring → forced mode.
///
/// First match wins. Kept short on purpose so behavior is auditable.
const HOST_RULES: &[(&str, InjectionMode)] = &[
    // Orca (onorca.dev) — desktop IDE whose agent terminal rejects all
    // synthetic input incl. Direct+admin. Copy is the only reliable mode.
    ("orca", InjectionMode::Copy),
];

/// Resolve the effective injection mode for the focused window. Pure +
/// testable. See the module doc for the Auto vs explicit-mode semantics.
pub fn resolve_injection_mode(
    foreground_title: Option<&str>,
    global_mode: InjectionMode,
) -> InjectionMode {
    match match_title_rule(foreground_title) {
        Some(mode) => mode,
        None => match global_mode {
            InjectionMode::Auto => InjectionMode::Direct,
            other => other,
        },
    }
}

/// First host rule whose needle appears (case-insensitively) in the window
/// title. Empty needles never match — mirrors ManualRuleResolver's guard
/// against `contains("")` matching everything.
fn match_title_rule(foreground_title: Option<&str>) -> Option<InjectionMode> {
    let title = foreground_title?;
    let lower = title.to_lowercase();
    HOST_RULES
        .iter()
        .find(|(needle, _)| !needle.is_empty() && lower.contains(*needle))
        .map(|(_, mode)| *mode)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orca_title_forces_copy_regardless_of_global() {
        // Orca rejects all synthetic input (Direct+admin verified to fail) —
        // Copy is the only working mode, for Auto AND explicit Direct.
        assert_eq!(
            resolve_injection_mode(Some("Orca — Task: fix bug"), InjectionMode::Auto),
            InjectionMode::Copy
        );
        assert_eq!(
            resolve_injection_mode(Some("my orca session"), InjectionMode::Direct),
            InjectionMode::Copy
        );
    }

    #[test]
    fn auto_resolves_to_direct_by_default() {
        // Auto = policy decides: no rule matches → Direct (plain-terminal
        // behavior, backed by the job layer's clipboard insurance).
        assert_eq!(
            resolve_injection_mode(Some("Claude Code — bash"), InjectionMode::Auto),
            InjectionMode::Direct
        );
        assert_eq!(
            resolve_injection_mode(Some("Visual Studio Code"), InjectionMode::Auto),
            InjectionMode::Direct
        );
        // No foreground info (Wayland, detection failure) → same default.
        assert_eq!(
            resolve_injection_mode(None, InjectionMode::Auto),
            InjectionMode::Direct
        );
        // Empty title (edge case) → same default.
        assert_eq!(
            resolve_injection_mode(Some(""), InjectionMode::Auto),
            InjectionMode::Direct
        );
    }

    #[test]
    fn explicit_global_is_respected_when_no_rule_matches() {
        assert_eq!(
            resolve_injection_mode(Some("Claude Code — bash"), InjectionMode::Direct),
            InjectionMode::Direct
        );
        assert_eq!(
            resolve_injection_mode(Some("Claude Code — bash"), InjectionMode::Copy),
            InjectionMode::Copy
        );
        assert_eq!(
            resolve_injection_mode(None, InjectionMode::Copy),
            InjectionMode::Copy
        );
    }

    #[test]
    fn case_insensitive_match() {
        assert_eq!(
            resolve_injection_mode(Some("ORCA"), InjectionMode::Direct),
            InjectionMode::Copy
        );
        assert_eq!(
            resolve_injection_mode(Some("OrCa"), InjectionMode::Auto),
            InjectionMode::Copy
        );
    }
}
