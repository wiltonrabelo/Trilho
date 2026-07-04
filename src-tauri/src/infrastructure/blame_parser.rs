//! Parser de `git blame --line-porcelain` (RF-03).

use crate::application::GitError;
use crate::domain::BlameLine;
use chrono::{DateTime, FixedOffset, TimeZone, Utc};

pub fn parse_line_porcelain(raw: &str) -> Result<Vec<BlameLine>, GitError> {
    let mut result = Vec::new();
    let lines: Vec<String> = raw
        .lines()
        .map(|l| l.trim_end_matches('\r').to_string())
        .collect();
    let mut i = 0;

    while i < lines.len() {
        let header = lines[i].trim();
        if header.is_empty() {
            i += 1;
            continue;
        }

        let parts: Vec<&str> = header.split_whitespace().collect();
        if parts.len() < 3 || !is_sha(parts[0]) {
            i += 1;
            continue;
        }

        let sha = parts[0].to_string();
        let final_line: u32 = parts[2].parse().unwrap_or(0);
        i += 1;

        let mut author = String::new();
        let mut author_time: i64 = 0;
        let mut author_tz = 0i32;
        let mut summary = String::new();
        let mut content = String::new();

        while i < lines.len() {
            let line = &lines[i];
            if let Some(stripped) = line.strip_prefix('\t') {
                content = stripped.to_string();
                i += 1;
                break;
            }
            if line.starts_with("filename ") && i + 1 < lines.len() {
                let next = lines[i + 1].trim_start_matches('\t').to_string();
                if !next.is_empty() && !is_sha_line(&next) {
                    content = next;
                    i += 2;
                    break;
                }
            }
            if is_sha_line(line) {
                break;
            }
            if let Some(rest) = line.strip_prefix("author ") {
                author = rest.to_string();
            } else if let Some(rest) = line.strip_prefix("author-time ") {
                author_time = rest.parse().unwrap_or(0);
            } else if let Some(rest) = line.strip_prefix("author-tz ") {
                author_tz = parse_git_tz(rest);
            } else if let Some(rest) = line.strip_prefix("summary ") {
                summary = rest.to_string();
            }
            i += 1;
        }

        result.push(BlameLine {
            line: final_line,
            commit_id: sha.clone(),
            short_id: format!("{:.7}", sha),
            author,
            authored_at: timestamp_to_iso(author_time, author_tz),
            summary,
            content,
        });
    }

    Ok(result)
}

fn is_sha(s: &str) -> bool {
    s.len() >= 7 && s.chars().all(|c| c.is_ascii_hexdigit())
}

fn is_sha_line(line: &str) -> bool {
    let first = line.split_whitespace().next().unwrap_or("");
    is_sha(first) && line.split_whitespace().count() >= 3
}

fn parse_git_tz(tz: &str) -> i32 {
    let tz = tz.trim();
    if tz.len() < 5 {
        return 0;
    }
    let sign = if tz.starts_with('-') { -1 } else { 1 };
    let digits = tz.trim_start_matches(['+', '-']);
    if digits.len() < 4 {
        return 0;
    }
    let hours: i32 = digits[0..2].parse().unwrap_or(0);
    let mins: i32 = digits[2..4].parse().unwrap_or(0);
    sign * (hours * 3600 + mins * 60)
}

fn timestamp_to_iso(secs: i64, offset_secs: i32) -> String {
    let offset = FixedOffset::east_opt(offset_secs).unwrap_or(FixedOffset::east_opt(0).unwrap());
    if let Some(dt) = offset.timestamp_opt(secs, 0).single() {
        dt.to_rfc3339()
    } else {
        Utc.timestamp_opt(secs, 0)
            .single()
            .map(|d: DateTime<Utc>| d.to_rfc3339())
            .unwrap_or_else(|| Utc::now().to_rfc3339())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa 1 1 2
author Alice
author-time 1700000000
author-tz +0300
summary init
filename src/a.ts
\tline one
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa 1 2
author Alice
author-time 1700000000
author-tz +0300
summary init
filename src/a.ts
\tline two
";

    #[test]
    fn parse_duas_linhas() {
        let parsed = parse_line_porcelain(SAMPLE).expect("parse");
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].line, 1);
        assert_eq!(parsed[0].content, "line one");
        assert_eq!(parsed[1].content, "line two");
    }
}
