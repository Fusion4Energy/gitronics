//! Reading and interpreting MCNP-family source files: full models parsed by
//! migjorn, and the plain data-card sidecar files (materials, transforms,
//! tallies, source) that are concatenated into the assembled model verbatim.

use crate::error::GitronicsError;
use crate::types::FileName;
use migjorn::{Model, Severity};
use std::fs;
use std::path::Path;

/// Reads an MCNP model file and parses it with migjorn, returning an error if
/// the parser reports any error-severity diagnostic (warnings are tolerated).
pub fn parse_model_file(path: &Path, file_name: &FileName) -> Result<Model, GitronicsError> {
    let text = fs::read_to_string(path).map_err(|source| GitronicsError::io_path(path, source))?;
    let model = Model::parse(&text);
    if let Some(diag) = model
        .diagnostics()
        .iter()
        .find(|d| matches!(d.severity, Severity::Error))
    {
        return Err(GitronicsError::FailedToLoadMCNPFile {
            file_name: file_name.clone(),
            error: diag.message.clone(),
        });
    }
    Ok(model)
}

/// Reads a Gitronics data-card file and returns its data-card text.
///
/// Following the data-card file convention, the first line is treated as a
/// title and dropped unless it is an MCNP comment (in which case it is kept as a
/// header), and content stops at the first blank line — anything after it is
/// ignored.
pub fn read_data_cards_text(path: &Path, file_name: &FileName) -> Result<String, GitronicsError> {
    let content =
        fs::read_to_string(path).map_err(|source| GitronicsError::io_path(path, source))?;
    let mut kept: Vec<&str> = Vec::new();
    for (i, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            break; // MCNP section separator: stop at the first blank line
        }
        if i == 0 && !is_mcnp_comment(line) {
            continue; // drop a non-comment title line
        }
        kept.push(line);
    }
    if kept.is_empty() {
        return Err(GitronicsError::FailedToLoadDataCardsFile {
            file_name: file_name.clone(),
            error: "no data cards found before the first blank line".to_string(),
        });
    }
    Ok(kept.join("\n"))
}

/// Sort key mirroring a data card's id: cards whose mnemonic ends in a number
/// (`M1`, `F4`, `TR1`) order by that number and come first; mnemonics without a
/// trailing number (`SDEF`, `MODE`) fall back to alphabetical order and come
/// last — matching the old `DataCardId::{Int, String}` ordering.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum DataCardKey {
    Int(u32),
    Str(String),
}

/// The sort key of a data-card file block, derived from the id of its first
/// data card. Full-line comments (and blank lines) are skipped so a comment
/// header does not decide the order.
fn first_data_card_key(chunk: &str) -> DataCardKey {
    for line in chunk.lines() {
        if is_mcnp_comment(line) {
            continue;
        }
        // First whitespace token is the mnemonic, minus any leading `*`.
        let mnemonic = line
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim_start_matches('*');
        // Split the trailing id number off the mnemonic letters (`TR1` -> 1,
        // `F4:N` -> 4).
        let digits: String = mnemonic
            .chars()
            .skip_while(|c| !c.is_ascii_digit())
            .take_while(|c| c.is_ascii_digit())
            .collect();
        return match digits.parse::<u32>() {
            Ok(id) => DataCardKey::Int(id),
            Err(_) => DataCardKey::Str(mnemonic.to_ascii_uppercase()),
        };
    }
    DataCardKey::Str(String::new())
}

/// Order data-card file blocks by the id of each block's first data card,
/// keeping the cards inside a block in their original order (a stable sort over
/// whole blocks). Restores the deterministic output the pre-migjorn-0.2 build
/// produced via `order_data_cards_groups_by_id`.
pub fn sort_data_card_chunks(chunks: &mut [String]) {
    chunks.sort_by_key(|chunk| first_data_card_key(chunk));
}

/// True if `line` is an MCNP full-line comment (first non-blank character is
/// `c`/`C`, followed by whitespace or end of line) or blank.
fn is_mcnp_comment(line: &str) -> bool {
    let t = line.trim_start();
    let mut chars = t.chars();
    match chars.next() {
        None => true,
        Some('c') | Some('C') => chars.next().is_none_or(|c| c.is_whitespace()),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_data_card_chunks_by_first_card_id() {
        // Blocks arrive in arbitrary configuration order.
        let mut chunks = vec![
            "m10 1001 1\nm2 8016 1".to_string(), // first card M10 -> 10
            "C header\nm1 6000 1".to_string(),   // comment skipped, M1 -> 1
            "sdef pos=0 0 0".to_string(),        // no id -> sorts last
            "m3 26000 1".to_string(),            // M3 -> 3
        ];
        sort_data_card_chunks(&mut chunks);
        assert_eq!(
            chunks,
            vec![
                "C header\nm1 6000 1".to_string(),
                "m3 26000 1".to_string(),
                "m10 1001 1\nm2 8016 1".to_string(),
                "sdef pos=0 0 0".to_string(),
            ]
        );
    }

    #[test]
    fn test_sort_data_card_chunks_is_stable_within_block() {
        // Cards inside a block are never reordered, even if not id-sorted.
        let mut chunks = vec!["m5 1001 1\nm2 8016 1".to_string()];
        sort_data_card_chunks(&mut chunks);
        assert_eq!(chunks, vec!["m5 1001 1\nm2 8016 1".to_string()]);
    }

    #[test]
    fn first_data_card_key_reads_the_trailing_id_of_the_mnemonic() {
        assert_eq!(first_data_card_key("m1 1001 1"), DataCardKey::Int(1));
        assert_eq!(first_data_card_key("M100 1001 1"), DataCardKey::Int(100));
        assert_eq!(first_data_card_key("*TR1 0 0 0"), DataCardKey::Int(1));
        assert_eq!(first_data_card_key("F4:N 1"), DataCardKey::Int(4));
        assert_eq!(first_data_card_key("FMESH14:n"), DataCardKey::Int(14));
    }

    #[test]
    fn first_data_card_key_falls_back_to_the_uppercased_mnemonic() {
        assert_eq!(
            first_data_card_key("sdef pos=0 0 0"),
            DataCardKey::Str("SDEF".to_string())
        );
        assert_eq!(
            first_data_card_key("MODE N P"),
            DataCardKey::Str("MODE".to_string())
        );
    }

    #[test]
    fn first_data_card_key_skips_comments_and_blanks() {
        assert_eq!(
            first_data_card_key("C a header\nc another\n\nm7 1001 1"),
            DataCardKey::Int(7)
        );
        // Nothing but comments has no id at all.
        assert_eq!(
            first_data_card_key("C only a comment"),
            DataCardKey::Str(String::new())
        );
        assert_eq!(first_data_card_key(""), DataCardKey::Str(String::new()));
    }

    #[test]
    fn numbered_cards_sort_before_unnumbered_ones() {
        assert!(DataCardKey::Int(9999) < DataCardKey::Str("AAAA".to_string()));
    }
}
