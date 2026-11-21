use std::collections::BTreeSet;

use konekode_complete::{Engine, Where, catalog, is_include_line, match_pairs, pairs};

#[test]
fn only_a_real_include_line_counts() {
    assert!(is_include_line("#include <bits/stdc++.h>"));
    assert!(is_include_line("  #include<vector>"));
    assert!(!is_include_line("#includes <x>"));
    assert!(!is_include_line("#include_x"));
    assert!(!is_include_line("#include9"));
    assert!(!is_include_line("int x;"));
}

#[test]
fn the_line_pair_is_the_include_line_pair_for_that_opener() {
    let all = pairs();
    let unlocked: BTreeSet<String> = all.iter().map(|pair| pair.id.clone()).collect();
    let engine = Engine::new(catalog());
    for pair in &all {
        let found = engine.line_pair(&unlocked, &pair.open);
        if pair.place == Where::IncludeLine {
            assert_eq!(found.as_ref(), Some(pair));
        } else {
            assert!(
                found.is_none_or(
                    |other| other.open != pair.open || other.place == Where::IncludeLine
                )
            );
        }
    }
    let includes: Vec<_> = all
        .iter()
        .filter(|pair| pair.place == Where::IncludeLine)
        .collect();
    assert_ne!(includes.len(), 0);
    for pair in all.iter().filter(|pair| pair.place != Where::IncludeLine) {
        if includes.iter().all(|line| line.open != pair.open) {
            assert_eq!(engine.line_pair(&unlocked, &pair.open), None);
        }
    }
}

#[test]
fn an_include_closer_must_be_on_the_same_line() {
    let all = pairs();
    let Some(line) = all.iter().find(|pair| pair.place == Where::IncludeLine) else {
        return;
    };
    let same = format!("#include {}x{}", line.open, line.close);
    assert!(match_pairs(&same, &all).contains(&(9, 11)));
    let split = format!("#include {}\n{}", line.open, line.close);
    assert!(
        !match_pairs(&split, &all)
            .iter()
            .any(|(opener, _)| *opener == 9)
    );
    let below = format!("int a;\n#include {}x{}", line.open, line.close);
    assert!(match_pairs(&below, &all).contains(&(16, 18)));
}
