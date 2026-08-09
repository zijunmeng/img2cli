//! Per-host injection policy.
//!
//! The global `injection_mode` (config) is one value applied to every target
//! window. But hosts differ in what synthetic input they accept: Orca's agent
//! terminal rejects ALL synthetic input — Direct (Enigo Unicode) included,
//! verified 2026-08-09 even as administrator — so only Copy + manual Ctrl+V
//! works there. This module overrides the global mode for known-constrained
//! hosts so the user doesn't have to flip settings every time they switch
//! windows.
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
    // synthetic input incl. Direct+admin. Copy is the only reliable mode, so
    // force it regardless of the user's global setting.
    ("orca", InjectionMode::Copy),
];

/// Resolve the effective injection mode for the focused window.
///
/// Returns the forced mode from the first matching host rule, or `global_mode`
/// unchanged if no rule matches (or the title is missing). Pure + testable.
pub fn resolve_injection_mode(
    foreground_title: Option<&str>,
    global_mode: InjectionMode,
) -> InjectionMode {
    let title = match foreground_title {
        Some(t) => t,
        None => return global_mode,
    };
    let lower = title.to_lowercase();
    for (needle, mode) in HOST_RULES {
        // Empty needle never matches — mirrors ManualRuleResolver's guard
        // against `contains("")` matching everything.
        if !needle.is_empty() && lower.contains(*needle) {
            return *mode;
        }
    }
    global_mode
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orca_title_forces_copy() {
        // Orca rejects all synthetic input (Direct+admin verified to fail) —
        // Copy is the only working mode, so force it regardless of global.
        assert_eq!(
            resolve_injection_mode(Some("Orca — Task: fix bug"), InjectionMode::Direct),
            InjectionMode::Copy
        );
        assert_eq!(
            resolve_injection_mode(Some("my orca session"), InjectionMode::Auto),
            InjectionMode::Copy
        );
    }

    #[test]
    fn non_matching_title_keeps_global() {
        // Plain terminals / VS Code / browsers: respect the user's global choice.
        assert_eq!(
            resolve_injection_mode(Some("Claude Code — bash"), InjectionMode::Direct),
            InjectionMode::Direct
        );
        assert_eq!(
            resolve_injection_mode(Some("Visual Studio Code"), InjectionMode::Auto),
            InjectionMode::Auto
        );
    }

    #[test]
    fn none_title_keeps_global() {
        // No foreground info (Wayland, detection failure) → don't override.
        assert_eq!(
            resolve_injection_mode(None, InjectionMode::Swap),
            InjectionMode::Swap
        );
    }

    #[test]
    fn case_insensitive_match() {
        assert_eq!(
            resolve_injection_mode(Some("ORCA"), InjectionMode::Direct),
            InjectionMode::Copy
        );
        assert_eq!(
            resolve_injection_mode(Some("OrCa"), InjectionMode::Direct),
            InjectionMode::Copy
        );
    }

    #[test]
    fn empty_title_keeps_global() {
        // Empty title (edge case) → no rule matches → global preserved.
        assert_eq!(
            resolve_injection_mode(Some(""), InjectionMode::Direct),
            InjectionMode::Direct
        );
    }
}
