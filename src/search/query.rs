use crate::ranking;
use crate::search::tokenization::{split_camel_case, is_english_stop_word};
use itertools::Itertools;
use std::collections::HashSet;

/// Preprocesses a query into original and stemmed term pairs
/// When exact is true, splits only on whitespace and skips stemming/stopword removal
/// When exact is false, uses stemming/stopword logic but splits primarily on whitespace
pub fn preprocess_query(query: &str, exact: bool) -> Vec<(String, String)> {
    // Convert to lowercase first
    let lowercase_query = query.to_lowercase();

    if exact {
        // For exact matching, just split on whitespace and return as-is without stemming
        lowercase_query
            .split_whitespace()
            .filter(|word| !word.is_empty())
            .map(|word| {
                let original = word.to_string();
                (original.clone(), original)
            })
            .collect()
    } else {
        // For non-exact matching, apply stemming and stopword removal
        let stemmer = ranking::get_stemmer();

        // Split by whitespace first
        lowercase_query
            .split_whitespace()
            .flat_map(|word| {
                // Split each word on camel case boundaries
                split_camel_case(word)
            })
            .filter(|word| !word.is_empty() && !is_english_stop_word(word))
            .map(|word| {
                let original = word.to_string();
                let stemmed = stemmer.stem(&word).to_string();
                (original, stemmed)
            })
            .collect()
    }
}

/// Escapes special regex characters in a string
pub fn regex_escape(s: &str) -> String {
    let special_chars = [
        '.', '^', '$', '*', '+', '?', '(', ')', '[', ']', '{', '}', '|', '\\',
    ];
    let mut result = String::with_capacity(s.len() * 2);

    for c in s.chars() {
        if special_chars.contains(&c) {
            result.push('\\');
        }
        result.push(c);
    }

    result
}

/// Creates regex patterns that match either original or stemmed terms
/// Generates grouped patterns for better matching in various contexts
///
/// Returns a vector of tuples (pattern, HashSet<usize>), where the HashSet<usize>
/// contains indices of terms the pattern corresponds to.
/// For example, for term_pairs = [("ip", "ip"), ("whitelisting", "whitelist")]:
/// - ("(\bip|ip\b)", {0}) - combined pattern for "ip" with both boundaries
/// - ("(\b(whitelisting|whitelist)|(whitelisting|whitelist)\b)", {1}) - combined pattern for both forms with both boundaries
/// - ("(ipwhitelisting|ipwhitelist|whitelistingip|whitelistip)", {0, 1}) - all concatenated combinations
pub fn create_term_patterns(term_pairs: &[(String, String)]) -> Vec<(String, HashSet<usize>)> {
    let mut patterns: Vec<(String, HashSet<usize>)> = Vec::new();

    // Generate individual term patterns with combined start and end boundaries
    for (term_idx, (original, stemmed)) in term_pairs.iter().enumerate() {
        let base_pattern = if original == stemmed {
            regex_escape(original)
        } else {
            format!("({}|{})", regex_escape(original), regex_escape(stemmed))
        };

        // Create a HashSet with just this term's index
        let term_indices = HashSet::from([term_idx]);

        // Add combined boundary pattern (start OR end boundary)
        patterns.push((
            format!("(\\b{}|{}\\b)", base_pattern, base_pattern),
            term_indices.clone(),
        ));
    }

    // Generate concatenated combinations for multi-term queries
    if term_pairs.len() > 1 {
        // Collect all terms (both original and stemmed)
        let terms: Vec<(String, usize)> = term_pairs
            .iter()
            .enumerate()
            .flat_map(|(term_idx, (o, s))| vec![(o.clone(), term_idx), (s.clone(), term_idx)])
            .collect();

        // Group permutations by their term indices
        let mut concatenated_patterns: std::collections::HashMap<Vec<usize>, Vec<String>> =
            std::collections::HashMap::new();

        // Generate permutations of terms (2 at a time)
        for perm in terms.iter().permutations(2).unique() {
            // Extract the term indices for this permutation
            let term_indices: Vec<usize> = perm.iter().map(|(_, idx)| *idx).collect();

            // Skip if we're just getting different forms of the same term
            if term_indices.iter().unique().count() < 2 {
                continue;
            }

            // Create the concatenated pattern
            let concatenated = perm
                .iter()
                .map(|(term, _)| regex_escape(term))
                .collect::<String>();

            // Add to the group
            concatenated_patterns
                .entry(term_indices)
                .or_default()
                .push(concatenated);
        }

        // Add each group as a single pattern with alternatives
        for (term_indices, pattern_group) in concatenated_patterns {
            if pattern_group.len() == 1 {
                patterns.push((pattern_group[0].clone(), term_indices.into_iter().collect()));
            } else {
                // Combine patterns with OR
                let combined_pattern = format!("({})", pattern_group.join("|"));
                patterns.push((combined_pattern, term_indices.into_iter().collect()));
            }
        }
    }

    patterns
}

#[cfg(test)]
mod tests {
    include!("query_tests.rs");

    #[test]
    fn test_grouped_patterns() {
        // Test with "ip" and "whitelisting"
        let term_pairs = vec![
            ("ip".to_string(), "ip".to_string()),
            ("whitelisting".to_string(), "whitelist".to_string()),
        ];

        let patterns = create_term_patterns(&term_pairs);

        // Print the patterns for inspection
        println!("Generated patterns:");
        for (pattern, indices) in &patterns {
            println!("Pattern: {:?}, Indices: {:?}", pattern, indices);
        }

        // Verify we have the expected number of patterns
        // 1 pattern for each term (with combined boundaries) + 2 patterns for combinations
        // (one for each order of terms)
        assert_eq!(patterns.len(), 4);

        // Verify the first pattern is for "ip" with both boundaries
        let ip_pattern = patterns
            .iter()
            .find(|(_, indices)| indices.len() == 1 && indices.contains(&0));
        assert!(ip_pattern.is_some());
        let (ip_pattern, _) = ip_pattern.unwrap();
        assert!(ip_pattern.contains("\\bip|ip\\b"));

        // Verify the second pattern is for "whitelisting|whitelist" with both boundaries
        let whitelist_pattern = patterns
            .iter()
            .find(|(_, indices)| indices.len() == 1 && indices.contains(&1));
        assert!(whitelist_pattern.is_some());
        let (whitelist_pattern, _) = whitelist_pattern.unwrap();
        assert!(whitelist_pattern.contains("(whitelisting|whitelist)"));

        // Verify there are combination patterns
        let combo_patterns: Vec<_> = patterns
            .iter()
            .filter(|(_, indices)| indices.len() == 2)
            .collect();
        assert_eq!(combo_patterns.len(), 2);

        // Check that one combination has "ipwhitelisting|ipwhitelist"
        let has_ip_first = combo_patterns.iter().any(|(pattern, _)| {
            pattern.contains("ipwhitelisting") && pattern.contains("ipwhitelist")
        });
        assert!(has_ip_first);

        // Check that one combination has "whitelistingip|whitelistip"
        let has_whitelist_first = combo_patterns.iter().any(|(pattern, _)| {
            pattern.contains("whitelistingip") && pattern.contains("whitelistip")
        });
        assert!(has_whitelist_first);
    }
}
