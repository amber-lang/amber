//! Construct coverage test: verifies every language construct has at least one test fixture.
//!
//! This test collects all constructs from:
//! 1. Keyword registry (iter_keywords()) - keywords with their kinds
//! 2. EBNF grammar rules (statement_local, expression alternatives)
//!
//! It then checks each construct against test fixtures and generates coverage reports.

use crate::modules::keywords::{iter_keywords, KeywordKind};
use crate::utils::grammar_ebnf::generate_grammar_ebnf;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

/// A language construct with its kind and coverage information
#[derive(Debug, Clone)]
struct Construct {
    name: String,
    kind: ConstructKind,
}

/// The kind of construct - mirrors KeywordKind but for unified representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConstructKind {
    Stmt,
    BuiltinStmt,
    BuiltinExpr,
    BinaryOp,
    GrammarRule,
}

impl From<KeywordKind> for ConstructKind {
    fn from(kind: KeywordKind) -> Self {
        match kind {
            KeywordKind::Stmt => ConstructKind::Stmt,
            KeywordKind::BuiltinStmt => ConstructKind::BuiltinStmt,
            KeywordKind::BuiltinExpr => ConstructKind::BuiltinExpr,
            KeywordKind::BinaryOp => ConstructKind::BinaryOp,
        }
    }
}

/// Coverage data for a construct
#[derive(Debug, Clone)]
struct ConstructCoverage {
    construct: String,
    kind: ConstructKind,
    fixture_count: usize,
    fixture_paths: Vec<String>,
    source_rules: Vec<String>,
}

/// Hand-written syntax pattern table mapping construct names to regex patterns
/// This is the only hand-maintained surface - extend here for new constructs
const SYNTAX_PATTERNS: &[(&str, &str)] = &[
    // Keywords - mechanical \b{kw}\b patterns
    ("if", r"\bif\b"),
    ("while", r"\bwhile\b"),
    ("for", r"\bfor\b"),
    ("loop", r"\bloop\b"),
    ("break", r"\bbreak\b"),
    ("continue", r"\bcontinue\b"),
    ("return", r"\breturn\b"),
    ("fail", r"\bfail\b"),
    ("let", r"\blet\b"),
    ("const", r"\bconst\b"),
    ("fun", r"\bfun\b"),
    ("main", r"\bmain\b"),
    ("import", r"\bimport\b"),
    ("test", r"\btest\b"),
    ("as", r"\bas\b"),
    ("is", r"\bis\b"),
    ("status", r"\bstatus\b"),
    ("ref", r"\bref\b"),
    ("pub", r"\bpub\b"),
    ("and", r"\band\b"),
    ("or", r"\bor\b"),
    ("not", r"\bnot\b"),
    ("then", r"\bthen\b"),
    ("else", r"\belse\b"),
    ("in", r"\bin\b"),
    ("from", r"\bfrom\b"),
    ("failed", r"\bfailed\b"),
    ("succeeded", r"\bsucceeded\b"),
    ("exited", r"\bexited\b"),
    ("silent", r"\bsilent\b"),
    ("suppress", r"\bsuppress\b"),
    ("trust", r"\btrust\b"),
    ("sudo", r"\bsudo\b"),
    // Builtins - statement
    ("echo", r"\becho\s*\("),
    ("cd", r"\bcd\s*\("),
    ("clear", r"\bclear\s*\("),
    ("wait", r"\bwait\s*\("),
    ("mkdir", r"\bmkdir\s*\("),
    ("rm", r"\brm\s*\("),
    ("cp", r"\bcp\s*\("),
    ("mv", r"\bmv\s*\("),
    ("cat", r"\bcat\s*\("),
    ("grep", r"\bgrep\s*\("),
    ("awk", r"\bawk\s*\("),
    ("sed", r"\bsed\s*\("),
    ("find", r"\bfind\s*\("),
    ("ls", r"\bls\s*\("),
    ("pwd", r"\bpwd\s*\("),
    ("date", r"\bdate\s*\("),
    // Builtins - expression
    ("len", r"\blen\s*\("),
    ("lines", r"\blines\s*\("),
    ("shellname", r"\bshellname\s*\("),
    ("shellversion", r"\bshellversion\s*\("),
    ("pid", r"\bpid\s*\("),
    ("exists", r"\bexists\s*\("),
    ("isdir", r"\bisdir\s*\("),
    ("isfile", r"\bisfile\s*\("),
    // Punctuation / grammar rules
    ("lt", r"<[^=]"),
    ("gt", r">[^=]"),
    ("le", r"<="),
    ("ge", r">="),
    ("shorthand_add", r"\+="),
    ("shorthand_sub", r"-="),
    ("shorthand_mul", r"\*="),
    ("shorthand_div", r"/="),
    ("shorthand_modulo", r"%="),
    ("range", r"\.\."),
    ("range_inclusive", r"\.\.\="),
    ("ternary", r"then\s+.+?\s+else"),
    ("parentheses", r"\([^)]*\)"),
    ("array", r"\[[^\]]*\]"),
    ("expression_index", r"\[[^\]]+\]"),
    ("command", r"\$\$"),
    ("function_call", r"\w+\s*\("),
    ("binary_operation", r"[\+\-\*\/%]"),
    ("unary_operation", r"-\s*\w"),
    ("cast", r"\bas\s+\w+"),
    ("comment_doc", r"///"),
    ("comment", r"//"),
    // Registry keyword names (from AutoKeyword) mapped to the same syntax —
    // the table above keys these by EBNF rule name, the registry by keyword string
    ("+=", r"\+="),
    ("-=", r"-="),
    ("*=", r"\*="),
    ("/=", r"/="),
    ("%=", r"%="),
    ("///", r"///"),
    ("//", r"//"),
];

/// Pre-compiled regex patterns for performance (computed once per test run)
fn get_compiled_patterns() -> &'static Vec<(String, Regex)> {
    use std::sync::OnceLock;
    static COMPILED: OnceLock<Vec<(String, Regex)>> = OnceLock::new();
    COMPILED.get_or_init(|| {
        SYNTAX_PATTERNS
            .iter()
            .filter_map(|(name, pattern)| {
                Regex::new(pattern)
                    .ok()
                    .map(|re| (name.to_string(), re))
            })
            .collect()
    })
}

/// Collect all constructs from keyword registry and EBNF grammar
fn collect_constructs() -> Vec<Construct> {
    let mut constructs = Vec::new();
    let mut seen = HashSet::new();

    // Add keywords from registry
    for reg in iter_keywords() {
        if seen.insert(reg.keyword.to_string()) {
            constructs.push(Construct {
                name: reg.keyword.to_string(),
                kind: ConstructKind::from(reg.kind),
            });
        }
    }

    // Parse EBNF grammar for rule names
    let grammar = generate_grammar_ebnf();
    let rule_regex = Regex::new(r"(\w+)\s*=").unwrap();

    // Focus on statement_local and expression rules
    let statement_local_match = Regex::new(r"statement_local\s*=\s*([^;]+);").unwrap();
    let expression_match = Regex::new(r"expression\s*=\s*([^;]+);").unwrap();

    // Extract alternatives from statement_local
    if let Some(caps) = statement_local_match.captures(&grammar) {
        if let Some(m) = caps.get(1) {
            let alternatives: Vec<&str> = m.as_str().split('|').map(|s| s.trim()).collect();
            for alt in alternatives {
                // Extract rule name (first identifier)
                if let Some(rule_captures) = rule_regex.captures(alt) {
                    if let Some(rule_name) = rule_captures.get(1) {
                        let name = rule_name.as_str();
                        // Skip terminal references (uppercase) and keywords
                        if name.chars().next().map_or(true, |c| c.is_lowercase())
                            && !name.starts_with("KEYWORD_")
                            && !name.starts_with("builtin_")
                            && seen.insert(name.to_string())
                        {
                            constructs.push(Construct {
                                name: name.to_string(),
                                kind: ConstructKind::GrammarRule,
                            });
                        }
                    }
                }
            }
        }
    }

    // Extract alternatives from expression
    if let Some(caps) = expression_match.captures(&grammar) {
        if let Some(m) = caps.get(1) {
            let alternatives: Vec<&str> = m.as_str().split('|').map(|s| s.trim()).collect();
            for alt in alternatives {
                if let Some(rule_captures) = rule_regex.captures(alt) {
                    if let Some(rule_name) = rule_captures.get(1) {
                        let name = rule_name.as_str();
                        if name.chars().next().map_or(true, |c| c.is_lowercase())
                            && !name.starts_with("KEYWORD_")
                            && !name.starts_with("builtin_")
                            && seen.insert(name.to_string())
                        {
                            constructs.push(Construct {
                                name: name.to_string(),
                                kind: ConstructKind::GrammarRule,
                            });
                        }
                    }
                }
            }
        }
    }

    constructs
}

/// Collect all .ab fixture files recursively
fn collect_fixtures() -> Vec<(String, String)> {
    let mut fixtures = Vec::new();
    let test_dir = Path::new("src/tests");

    fn walk_dir(dir: &Path, fixtures: &mut Vec<(String, String)>) {
        if !dir.exists() {
            return;
        }

        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk_dir(&path, fixtures);
            } else if path.extension().is_some_and(|e| e == "ab") {
                if let Ok(content) = fs::read_to_string(&path) {
                    let path_str = path.to_string_lossy().to_string();
                    fixtures.push((path_str, content));
                }
            }
        }
    }

    walk_dir(test_dir, &mut fixtures);
    fixtures
}

/// Check if a construct is covered by fixture filename or content
fn is_construct_covered(construct: &str, fixture_path: &str, fixture_content: &str) -> bool {
    // Filename signals coverage only on an explicit stem token (e.g. "if" in
    // depth_if_1.ab); whole-path substring matching let unrelated paths like
    // "diff" or "erroring" cover constructs they never exercise.
    let stem = std::path::Path::new(fixture_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    if stem.split(['_', '-']).any(|token| token == construct) {
        return true;
    }

    // Check content matches construct regex (use pre-compiled patterns)
    let compiled = get_compiled_patterns();
    if let Some((_, re)) = compiled.iter().find(|(name, _)| name == construct) {
        if re.is_match(fixture_content) {
            return true;
        }
    }

    false
}

/// Compute coverage for all constructs
fn compute_coverage(constructs: &[Construct], fixtures: &[(String, String)]) -> Vec<ConstructCoverage> {
    let mut coverage = Vec::new();

    for construct in constructs {
        let mut fixture_paths = Vec::new();

        for (path, content) in fixtures {
            if is_construct_covered(&construct.name, path, content) {
                fixture_paths.push(path.clone());
            }
        }

        let source_rules = if SYNTAX_PATTERNS.iter().any(|(n, _)| *n == construct.name) {
            vec![format!("syntax_pattern: {}", construct.name)]
        } else {
            vec![format!("keyword_registry: {}", construct.name)]
        };

        coverage.push(ConstructCoverage {
            construct: construct.name.clone(),
            kind: construct.kind,
            fixture_count: fixture_paths.len(),
            fixture_paths,
            source_rules,
        });
    }

    coverage
}

/// Generate markdown report
fn generate_markdown_report(coverage: &[ConstructCoverage]) -> String {
    let mut report = String::from("# Construct Coverage Report\n\n");

    // Sort by fixture count ascending
    let mut sorted = coverage.to_vec();
    sorted.sort_by_key(|a| a.fixture_count);

    // Split into sections
    let zero_fixtures: Vec<_> = sorted.iter().filter(|c| c.fixture_count == 0).collect();
    let one_fixture: Vec<_> = sorted.iter().filter(|c| c.fixture_count == 1).collect();
    let multi_fixture: Vec<_> = sorted.iter().filter(|c| c.fixture_count >= 2).collect();

    // 0 fixtures section
    report.push_str("## 0 Fixtures (Uncovered)\n\n");
    if zero_fixtures.is_empty() {
        report.push_str("*None*\n\n");
    } else {
        report.push_str("| Construct | Kind |\n");
        report.push_str("|-----------|------|\n");
        for c in &zero_fixtures {
            report.push_str(&format!(
                "| {} | {:?} |\n",
                c.construct, c.kind
            ));
        }
        report.push('\n');
    }

    // 1 fixture section
    report.push_str("## 1 Fixture (Thin Coverage)\n\n");
    if one_fixture.is_empty() {
        report.push_str("*None*\n\n");
    } else {
        report.push_str("| Construct | Kind | Fixture |\n");
        report.push_str("|-----------|------|---------|\n");
        for c in &one_fixture {
            let fixture = c.fixture_paths.first().map(|s| s.as_str()).unwrap_or("-");
            let fixture_short = fixture
                .split('/')
                .next_back()
                .unwrap_or(fixture)
                .chars()
                .take(20)
                .collect::<String>();
            report.push_str(&format!(
                "| {} | {:?} | {} |\n",
                c.construct, c.kind, fixture_short
            ));
        }
        report.push('\n');
    }

    // >=2 fixtures section (top 10 thinnest from this group)
    report.push_str("## ≥2 Fixtures (Well Covered - Top 10 Thinnest)\n\n");
    if multi_fixture.is_empty() {
        report.push_str("*None*\n\n");
    } else {
        report.push_str("| Construct | Kind | Fixture Count |\n");
        report.push_str("|-----------|------|---------------|\n");
        for c in multi_fixture.iter().take(10) {
            report.push_str(&format!(
                "| {} | {:?} | {} |\n",
                c.construct, c.kind, c.fixture_count
            ));
        }
        report.push('\n');
    }

    report
}

/// Generate JSON report
fn generate_json_report(coverage: &[ConstructCoverage]) -> String {
    let mut json = String::from("[\n");
    for (i, c) in coverage.iter().enumerate() {
        if i > 0 {
            json.push_str(",\n");
        }
        json.push_str(&format!(
            "  {{\n    \"construct\": \"{}\",\n    \"kind\": \"{:?}\",\n    \"fixture_count\": {},\n    \"fixture_paths\": {:?},\n    \"source_rules\": {:?}\n  }}",
            c.construct, c.kind, c.fixture_count, c.fixture_paths, c.source_rules
        ));
    }
    json.push_str("\n]\n");
    json
}

/// Load baseline from committed file
fn load_baseline() -> HashMap<String, usize> {
    let baseline_path = Path::new("src/tests/construct_coverage_baseline.json");
    let mut baseline = HashMap::new();

    if !baseline_path.exists() {
        return baseline;
    }

    let content = match fs::read_to_string(baseline_path) {
        Ok(c) => c,
        Err(_) => return baseline,
    };

    // Simple JSON parsing for baseline
    // Look for patterns like: "construct": "name" and "fixture_count": N
    let mut current_construct = String::new();
    let mut in_object = false;

    for line in content.lines() {
        let trimmed = line.trim();
        
        // Start of object
        if trimmed == "{" {
            in_object = true;
            current_construct.clear();
        }
        
        // Extract construct name
        if in_object && trimmed.starts_with("\"construct\":") {
            // Format: "construct": "clear",
            let after_colon = trimmed.split(':').nth(1).unwrap_or("");
            if let Some(start) = after_colon.find('"') {
                let rest = &after_colon[start + 1..];
                if let Some(end) = rest.find('"') {
                    current_construct = rest[..end].to_string();
                }
            }
        }
        
        // Extract fixture count and save
        if in_object && trimmed.starts_with("\"fixture_count\":") {
            if let Some(count_str) = trimmed.split(':').nth(1) {
                if let Some(count) = count_str.trim().split(',').next().and_then(|s| s.parse::<usize>().ok()) {
                    if !current_construct.is_empty() {
                        baseline.insert(current_construct.clone(), count);
                    }
                }
            }
        }
        
        // End of object
        if trimmed == "}" {
            in_object = false;
        }
    }

    baseline
}

#[test]
fn construct_coverage() {
    // Collect constructs and fixtures
    let constructs = collect_constructs();
    let fixtures = collect_fixtures();

    println!("Collected {} constructs", constructs.len());
    println!("Collected {} fixtures", fixtures.len());

    // Compute coverage
    let coverage = compute_coverage(&constructs, &fixtures);

    // Load baseline
    let baseline = load_baseline();

    // Check coverage against baseline and detect issues
    let mut uncovered_gaps = Vec::new();

    let construct_names: HashSet<_> = constructs.iter().map(|c| c.name.clone()).collect();

    for cov in &coverage {
        let has_coverage = cov.fixture_count > 0;
        let baseline_entry = baseline.get(&cov.construct);

        match (has_coverage, baseline_entry) {
            (true, Some(&0)) => {
                // Previously uncovered, now covered - great!
                println!(
                    "✓ Construct '{}' now covered (was a documented gap)",
                    cov.construct
                );
            }
            (false, Some(&0)) => {
                // Documented gap - warn but pass
                uncovered_gaps.push(cov.construct.clone());
                println!(
                    "⚠ Construct '{}' has 0 fixtures (documented gap in baseline)",
                    cov.construct
                );
            }
            (false, None) => {
                // No coverage and not a documented gap - collect for reporting
                uncovered_gaps.push(cov.construct.clone());
            }
            (false, Some(&baseline_count)) if baseline_count > 0 => {
                // Regression: had coverage, now lost it
                panic!(
                    "REGRESSION: Construct '{}' had {} fixtures in baseline, now has 0",
                    cov.construct, baseline_count
                );
            }
            (true, _) => {
                // Covered - good
            }
            _ => {}
        }

        // Check for new constructs not in baseline
        if !baseline.contains_key(&cov.construct) && cov.fixture_count == 0 {
            // Already added to uncovered_gaps above
        }
    }

    // Check for stale baseline entries (in baseline but no longer in constructs)
    for name in baseline.keys() {
        if !construct_names.contains(name) {
            println!(
                "⚠ Baseline entry '{}' is stale (construct no longer exists)",
                name
            );
        }
    }

    // Generate reports
    let markdown = generate_markdown_report(&coverage);
    let json = generate_json_report(&coverage);

    // Write reports to target directory
    let target_dir = Path::new("target/construct_coverage");
    if !target_dir.exists() {
        fs::create_dir_all(target_dir).expect("Failed to create target directory");
    }

    let md_path = target_dir.join("construct_coverage.md");
    let mut md_file = File::create(&md_path).expect("Failed to create markdown file");
    md_file
        .write_all(markdown.as_bytes())
        .expect("Failed to write markdown");

    let json_path = target_dir.join("construct_coverage.json");
    let mut json_file = File::create(&json_path).expect("Failed to create JSON file");
    json_file
        .write_all(json.as_bytes())
        .expect("Failed to write JSON");

    println!(
        "✓ Reports generated: {}/construct_coverage.md, {}/construct_coverage.json",
        target_dir.display(),
        target_dir.display()
    );

    // Summary
    let uncovered_count = coverage.iter().filter(|c| c.fixture_count == 0).count();
    let covered_count = coverage.len() - uncovered_count;
    println!(
        "\n=== Summary ===\nTotal constructs: {}\nCovered: {}\nUncovered: {}",
        coverage.len(),
        covered_count,
        uncovered_count
    );

    if !uncovered_gaps.is_empty() {
        println!("\nUncovered constructs (need fixtures or documented as gaps):");
        for gap in &uncovered_gaps {
            println!("  - {}", gap);
        }
    }

    // Filter to only non-gaps (uncovered but NOT in baseline as 0)
    let non_gap_uncovered: Vec<_> = uncovered_gaps
        .iter()
        .filter(|g| baseline.get(*g).map_or(true, |&c| c != 0))
        .cloned()
        .collect();

    if !non_gap_uncovered.is_empty() {
        panic!(
            "{} constructs have 0 fixtures and are not documented gaps: {:?}",
            non_gap_uncovered.len(),
            non_gap_uncovered
        );
    } else if !uncovered_gaps.is_empty() && baseline.is_empty() {
        println!("\nNote: No baseline file exists yet. Create src/tests/construct_coverage_baseline.json");
        println!("with these constructs marked as 0 fixtures to document them as allowed gaps.");
    }
}

/// Report test: prints construct→count table for visibility
#[test]
fn construct_coverage_report() {
    let constructs = collect_constructs();
    let fixtures = collect_fixtures();
    let coverage = compute_coverage(&constructs, &fixtures);

    // Sort by fixture count ascending
    let mut sorted = coverage;
    sorted.sort_by_key(|a| a.fixture_count);

    println!("\n=== Construct Coverage Report ===");
    println!("Total constructs: {}", sorted.len());
    println!("\nConstruct -> Fixture Count (sorted ascending):\n");

    for c in &sorted {
        println!("  {:20} {:4}  ({:?})", c.construct, c.fixture_count, c.kind);
    }

    // Summary stats
    let zero_count = sorted.iter().filter(|c| c.fixture_count == 0).count();
    let one_count = sorted.iter().filter(|c| c.fixture_count == 1).count();
    let two_count = sorted.iter().filter(|c| c.fixture_count == 2).count();
    let three_count = sorted.iter().filter(|c| c.fixture_count == 3).count();
    let four_plus = sorted.iter().filter(|c| c.fixture_count >= 4).count();

    println!("\n=== Summary ===");
    println!("  0 fixtures: {}", zero_count);
    println!("  1 fixture:  {}", one_count);
    println!("  2 fixtures: {}", two_count);
    println!("  3 fixtures: {}", three_count);
    println!("  ≥4 fixtures: {}", four_plus);
}

/// Ground-truth test: parses fixtures with construct tracing enabled and verifies
/// that the trace matches the regex-based coverage detection.
/// This is a hybrid gate: fails only when BOTH the regex match AND the trace agree a construct is missing.
#[test]
fn construct_coverage_ground_truth() {
    use crate::compiler::{AmberCompiler, CompilerOptions};
    use std::collections::HashMap;

    // Collect fixtures
    let fixtures = collect_fixtures();
    println!("\n=== Ground Truth Test ===");
    println!("Parsing {} fixtures with construct tracing enabled\n", fixtures.len());

    // Load baseline
    let baseline = load_baseline();

    // Build construct→fixtures attribution from real parses
    let mut construct_to_fixtures: HashMap<String, Vec<String>> = HashMap::new();
    let mut parse_errors = Vec::new();

    for (path, content) in &fixtures {
        // Parse with trace enabled
        let compiler = AmberCompiler::new(
            content.clone(),
            Some(path.clone()),
            CompilerOptions::default(),
        );

        match compiler.tokenize() {
            Ok(tokens) => {
                match compiler.parse(tokens) {
                    Ok((_block, _meta)) => {
                        // Get trace data
                        if let Some(trace) = crate::utils::construct_trace::with_trace(|t| t.clone()) {
                            for construct in trace.constructs {
                                construct_to_fixtures
                                    .entry(construct.to_string())
                                    .or_default()
                                    .push(path.clone());
                            }
                        }
                    }
                    Err(e) => {
                        parse_errors.push((path.clone(), e.message.unwrap_or_default()));
                    }
                }
            }
            Err(e) => {
                parse_errors.push((path.clone(), e.message.unwrap_or_default()));
            }
        }
    }

    // Print ground truth attribution
    println!("Construct → Fixtures Attribution (from real parses):");
    println!("---------------------------------------------------");
    let mut sorted_constructs: Vec<_> = construct_to_fixtures.iter().collect();
    sorted_constructs.sort_by(|a, b| a.0.cmp(b.0));
    for (construct, fixtures) in &sorted_constructs {
        let fixture_count = fixtures.len();
        println!("  {:20} → {} fixture(s)", construct, fixture_count);
    }

    // Get all constructs from the coverage test
    let all_constructs = collect_constructs();
    let construct_names: HashSet<_> = all_constructs.iter().map(|c| c.name.clone()).collect();

    // Find constructs that have NO trace coverage
    let trace_uncovered: Vec<_> = construct_names
        .iter()
        .filter(|name| !construct_to_fixtures.contains_key(*name))
        .collect();

    // For each trace-uncovered construct, check if regex ALSO says it's uncovered
    // This is the hybrid gate: fail only if BOTH agree AND it's not in the baseline
    let mut hybrid_uncovered = Vec::new();
    let fixtures_content: Vec<_> = fixtures.iter().map(|(_, c)| c.as_str()).collect();
    let fixtures_paths: Vec<_> = fixtures.iter().map(|(p, _)| p.as_str()).collect();

    for construct in &trace_uncovered {
        // Check regex coverage
        let regex_covered = fixtures_content
            .iter()
            .zip(fixtures_paths.iter())
            .any(|(content, path)| is_construct_covered(construct, path, content));

        if !regex_covered {
            // Check if this is a documented baseline gap
            let is_baseline_gap = baseline.get(*construct).is_some_and(|&c| c == 0);
            if !is_baseline_gap {
                // BOTH trace and regex agree this construct is uncovered and not exempted
                hybrid_uncovered.push(*construct);
            }
        }
    }

    // Print summary
    println!("\n=== Hybrid Gate Summary ===");
    println!("Constructs with NO trace coverage: {}", trace_uncovered.len());
    println!("Constructs with BOTH trace AND regex uncovered: {}", hybrid_uncovered.len());

    if !hybrid_uncovered.is_empty() {
        println!("\nHybrid uncovered constructs (need fixtures):");
        for construct in &hybrid_uncovered {
            println!("  - {}", construct);
        }
        panic!(
            "{} constructs have NO trace coverage AND NO regex coverage",
            hybrid_uncovered.len()
        );
    } else {
        println!("\n✓ All constructs have either trace coverage OR regex coverage (or both)");
    }

    // Report parse errors if any
    if !parse_errors.is_empty() {
        println!("\n⚠ Parse errors ({}):", parse_errors.len());
        for (path, error) in &parse_errors {
            println!("  {}: {}", path, error);
        }
    }
}
