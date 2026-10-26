//! RED — Finding A6: contract name drift (spec.md references contract files that differ from contracts/ names).
//! TDD: fails until all referenced contracts exist with matching basename.

#[test]
fn contract_references_exist_and_match() {
    let spec = include_str!("../../specs/005-adopt-design-sona/spec.md");
    // Extract referenced contract filenames (heuristic: "contracts/" + basename).
    let refs: Vec<&str> = spec.split("contracts/").skip(1)
        .filter_map(|s| s.split(|c: char| c == ' ' || c == ')' || c == '\n').next())
        .collect();
    // Check contracts directory contents.
    let dir = std::fs::read_dir("specs/005-adopt-design-sona/contracts").expect("contracts dir");
    let names: std::collections::HashSet<String> = dir.filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".md"))
        .collect();
    // For every reference, basename must exist.
    for r in refs.iter().take(10) {
        let basename = r.split('/').last().unwrap_or(r);
        if basename.is_empty() { continue; }
        assert!(
            names.contains(basename) || names.contains(&(basename.to_string() + ".md")),
            "Contract reference '{}' not found in contracts/ (drift)", basename
        );
    }
}
