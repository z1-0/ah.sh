use anyhow::Result;
use chrono::{DateTime, Utc};
use comfy_table::{Attribute, Cell, Color, Table, presets::UTF8_FULL};
use console::{Term, style};
use std::collections::HashMap;
use std::io::Write;

use crate::{provider::ProviderType, session::Session, session::types::HISTORY_LIMIT};

/// Language grouping by first letter range
struct LanguageGroup {
    range: String,
    languages: Vec<String>,
}

/// Print warning message
pub fn print_warning<S: ToString>(msg: S) {
    println!("{}: {}", style("WARNING").yellow(), msg.to_string());
}

/// Print success message
pub fn print_success<S: ToString>(msg: S) {
    println!("{}", style(msg.to_string()).green());
}

/// Print error message
pub fn print_error<S: ToString>(msg: S) {
    eprintln!("{}", style(msg.to_string()).red());
}

/// Print info message
pub fn print_info<S: ToString>(msg: S) {
    println!("{}", msg.to_string());
}

pub fn print_bold<S: ToString>(msg: S) {
    println!("{}", style(msg.to_string()).bold());
}

/// Check if output should use colors (not piped)
pub fn is_terminal() -> bool {
    Term::stderr().is_term()
}

/// Ask for user confirmation, returns true if confirmed
pub fn ask_confirmation(prompt: &str) -> bool {
    print!("{}", prompt);
    if std::io::stdout().flush().is_err() {
        return false;
    }
    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_err() {
        return false;
    }
    matches!(input.trim().to_ascii_lowercase().as_str(), "y" | "yes")
}

/// Print session found info (multi-line)
pub fn print_session_found(id: &str, provider: &str, languages: &[String]) {
    print_success("Session found");
    println!("  ID:       {}", id);
    println!("  Provider: {}", provider);
    println!("  Languages: {}", languages.join(", "));
    println!();
}

/// Print no existing session info (multi-line)
pub fn print_no_session(provider: &str, languages: &[String]) {
    print_info("No existing session");
    println!("  Provider: {}", provider);
    println!("  Languages: {}", languages.join(", "));
    println!();
}

/// Sessions table with default headers
pub fn print_sessions_list(sessions: &[Session]) {
    let default_headers = ["Index", "ID", "Provider", "Languages"];
    // Build table data
    let mut rows: Vec<Vec<String>> = Vec::with_capacity(sessions.len());

    for (i, s) in sessions.iter().enumerate() {
        let langs = s.languages.join(", ");
        rows.push(vec![
            (i + 1).to_string(),
            s.id.clone(),
            s.provider.to_string(),
            langs,
        ]);
    }
    print_sessions_table(&default_headers, &rows);
}

/// Sessions table printer using comfy-table
fn print_sessions_table(headers: &[&str], rows: &[Vec<String>]) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(comfy_table::ContentArrangement::Dynamic);

    // Set header with blue bold cells
    let header_cells: Vec<Cell> = headers
        .iter()
        .map(|h| Cell::new(*h).add_attribute(Attribute::Bold).fg(Color::Blue))
        .collect();
    table.set_header(header_cells);

    // Add rows
    for row in rows {
        let row_cells: Vec<Cell> = row
            .iter()
            .enumerate()
            .map(|(i, cell)| {
                if i == 1 {
                    // Provider column - green
                    Cell::new(cell.as_str()).fg(Color::Green)
                } else {
                    Cell::new(cell.as_str())
                }
            })
            .collect();
        table.add_row(row_cells);
    }

    println!("{table}");
}

pub fn print_provider_list(providers: &[ProviderType]) -> Result<()> {
    // Build provider info: (name, languages_count)
    let mut provider_info: Vec<(String, usize)> = Vec::with_capacity(providers.len());

    for p in providers {
        let langs = p.to_provider()?.get_supported_languages();
        provider_info.push((p.to_string(), langs.len()));
    }
    print_provider_table(&provider_info);
    Ok(())
}

/// Provider list table using comfy-table
/// Provider info: (name, languages_count)
fn print_provider_table(providers: &[(String, usize)]) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(comfy_table::ContentArrangement::Dynamic);

    // Header
    table.set_header(vec![
        Cell::new("Index")
            .add_attribute(Attribute::Bold)
            .fg(Color::Blue),
        Cell::new("Provider")
            .add_attribute(Attribute::Bold)
            .fg(Color::Blue),
        Cell::new("Languages")
            .add_attribute(Attribute::Bold)
            .fg(Color::Blue),
    ]);

    // Rows
    for (i, (name, lang_count)) in providers.iter().enumerate() {
        table.add_row(vec![
            Cell::new(i + 1),
            Cell::new(name.as_str()).fg(Color::Green),
            Cell::new(*lang_count),
        ]);
    }

    println!("{table}");
}

pub fn print_provider_show(providers: &[ProviderType]) -> Result<()> {
    for (i, provider_type) in providers.iter().enumerate() {
        if i > 0 {
            println!();
        }
        write_provider_languages(*provider_type)?;
    }

    Ok(())
}

/// Print session history prompt for current directory
pub fn print_session_history(sessions: &[Session], history_timestamps: &[DateTime<Utc>]) {
    println!();
    println!("╭─ Session History ──────────────────────────────────────────╮");
    for (i, session) in sessions.iter().enumerate().take(HISTORY_LIMIT) {
        let langs = session.languages.join(", ");
        let timestamp = history_timestamps
            .get(i)
            .map(|ts| ts.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        println!("│ │");
        println!("│ #{} {} │", i + 1, session.id);
        println!("│ {} ({}) │", langs, session.provider);
        println!("│ Last used: {} │", timestamp);
    }
    println!("│ │");
    println!("╰────────────────────────────────────────────────────────────────╯");
    print!(" Enter session number to restore, or press Enter to skip: ");
    let _ = std::io::stdout().flush();
}

fn write_provider_languages(pt: ProviderType) -> Result<()> {
    let provider = pt.to_provider()?;
    let supported_languages = provider.get_supported_languages();
    let language_to_aliases = provider.get_language_to_aliases();

    // Build aliases list
    let mut aliases: Vec<(String, String)> = Vec::new();
    for language in supported_languages {
        if let Some(lang_aliases) = language_to_aliases.get(language) {
            for alias in lang_aliases {
                aliases.push((language.clone(), alias.clone()));
            }
        }
    }

    print_language_groups(&pt.to_string(), supported_languages, &aliases);

    Ok(())
}

/// Group languages by alphabet ranges (A-E, F-J, K-O, P-T, U-Z)
fn group_languages_by_alphabet(languages: &[String]) -> Vec<LanguageGroup> {
    let mut groups: Vec<LanguageGroup> = Vec::new();
    let mut current_range = String::new();
    let mut current_languages: Vec<String> = Vec::new();
    let mut last_range_idx: Option<usize> = None;

    // Define ranges as (start_char, end_char, range_label)
    let ranges: [(char, char, &str); 5] = [
        ('A', 'E', "A-E"),
        ('F', 'J', "F-J"),
        ('K', 'O', "K-O"),
        ('P', 'T', "P-T"),
        ('U', 'Z', "U-Z"),
    ];

    for lang in languages {
        let first_char = lang.chars().next().unwrap_or('A').to_ascii_uppercase();

        // Find which range this language belongs to
        let range_idx = ranges
            .iter()
            .position(|(s, e, _)| first_char >= *s && first_char <= *e);

        match range_idx {
            Some(idx) => {
                if Some(idx) == last_range_idx {
                    // Same range, add to current group
                    current_languages.push(lang.clone());
                } else {
                    // New range, save current and start new
                    if !current_languages.is_empty() {
                        groups.push(LanguageGroup {
                            range: current_range.clone(),
                            languages: current_languages.clone(),
                        });
                    }
                    current_range = ranges[idx].2.to_string();
                    current_languages = vec![lang.clone()];
                    last_range_idx = Some(idx);
                }
            }
            None => {
                // Outside A-Z, put in first group if empty
                if current_languages.is_empty() && !groups.is_empty() {
                    // Add to last group
                    groups.last_mut().unwrap().languages.push(lang.clone());
                } else if current_languages.is_empty() {
                    current_range = "A-E".to_string();
                    current_languages.push(lang.clone());
                    last_range_idx = Some(0);
                } else {
                    groups.push(LanguageGroup {
                        range: current_range.clone(),
                        languages: current_languages.clone(),
                    });
                    current_range = "A-E".to_string();
                    current_languages = vec![lang.clone()];
                    last_range_idx = Some(0);
                }
            }
        }
    }

    // Add last group
    if !current_languages.is_empty() {
        groups.push(LanguageGroup {
            range: current_range,
            languages: current_languages,
        });
    }

    groups
}

/// Print language groups with inline aliases
fn print_language_groups(provider: &str, languages: &[String], aliases: &[(String, String)]) {
    // Build language -> aliases map
    let mut lang_to_aliases: HashMap<String, Vec<String>> = HashMap::new();
    for (lang, alias) in aliases {
        lang_to_aliases
            .entry(lang.clone())
            .or_default()
            .push(alias.clone());
    }

    let groups = group_languages_by_alphabet(languages);

    // Print header
    println!(
        "{} ─────────────────────────────────────────────────",
        style(format!("Provider: {}", provider)).blue().bold()
    );
    print_bold(format!("{} languages:", languages.len()));
    println!();

    // Print groups with blank line between them
    for (i, group) in groups.iter().enumerate() {
        if i > 0 {
            println!();
        }

        // Print group range header
        print_bold(&group.range);

        // Build line with inline aliases
        let mut line_parts: Vec<String> = Vec::new();
        for lang in &group.languages {
            if let Some(alias_list) = lang_to_aliases.get(lang) {
                line_parts.push(format!("{} ({})", lang, alias_list.join(", ")));
            } else {
                line_parts.push(lang.clone());
            }
        }

        println!("{}", line_parts.join(", "));
    }
}
