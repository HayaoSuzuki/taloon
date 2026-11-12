//! Bounded MIT title/copyright/contact forms; never an arbitrary prefix wildcard.

#[derive(Clone, Copy)]
enum Phase {
    Notices,
    Contacts,
}

/// Called only after every mandatory MIT body word and punctuation matches.
/// Every nonempty raw line must belong to this grammar, in this order.
pub(super) fn matches(header: &str) -> bool {
    if header.len() > 5_000 {
        return false;
    }
    let mut lines = header
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .peekable();
    if lines.peek().is_some_and(|line| title(line)) {
        lines.next();
    }
    let mut phase = Phase::Notices;
    let mut notices = 0;
    let mut contacts = 0;
    for line in lines {
        match phase {
            Phase::Notices if notice(line) => {
                notices += 1;
                if notices > 32 {
                    return false;
                }
            }
            _ if notices > 0 && domain(line) => {
                contacts += 1;
                if contacts > 2 {
                    return false;
                }
                phase = Phase::Contacts;
            }
            _ => return false,
        }
    }
    true
}

fn title(line: &str) -> bool {
    let line = line.strip_prefix("# ").unwrap_or(line);
    [
        "MIT License",
        "The MIT License",
        "The MIT License (MIT)",
        "MIT License (MIT)",
    ]
    .iter()
    .any(|title| line.eq_ignore_ascii_case(title))
}

fn notice(line: &str) -> bool {
    if super::copyright_line(line) {
        return true;
    }
    let Some(value) = line.strip_prefix("Copyright ") else {
        return false;
    };
    let value = value.strip_prefix("(c) ").unwrap_or(value);
    if !value.starts_with(|c: char| c.is_ascii_digit()) {
        return holders(value);
    }
    let mut groups = 0;
    let dated_groups = value.split(',').all(|group| {
        groups += 1;
        groups <= 32 && super::copyright_line(&format!("Copyright {}", group.trim()))
    });
    if dated_groups && groups >= 2 {
        return true;
    }
    // MIT additionally permits whitespace around the dash in year ranges.
    let Some(holder_offset) =
        value.find(|c: char| !(c.is_ascii_digit() || c == '-' || c == ',' || c.is_whitespace()))
    else {
        return false;
    };
    let (years, holder) = value.split_at(holder_offset);
    years.ends_with(char::is_whitespace) && valid_years(years) && holders(holder)
}

fn valid_years(value: &str) -> bool {
    let value = value.trim().strip_suffix(',').unwrap_or(value.trim());
    let year = |s: &str| s.len() == 4 && s.bytes().all(|b| b.is_ascii_digit());
    let mut count = 0;
    value.split(',').all(|part| {
        count += 1;
        count <= 32
            && (year(part.trim())
                || part
                    .split_once('-')
                    .is_some_and(|(a, b)| year(a.trim()) && year(b.trim())))
    })
}

fn holders(value: &str) -> bool {
    if super::holder_name(value) {
        return true;
    }
    // Corporate abbreviations retain their periods within each named group.
    if let Some((first, second)) = value.split_once(" and ")
        && super::holder_name(first)
        && (super::holder_name(second)
            || matches!(
                second.strip_suffix('.').unwrap_or(second),
                "affiliates" | "contributors"
            ))
    {
        return true;
    }
    let Some((holder, alias)) = value
        .strip_suffix(") and contributors")
        .and_then(|s| s.split_once(" ("))
    else {
        return false;
    };
    alias.len() <= 64
        && alias.split_whitespace().count() == 1
        && super::holder_name(holder)
        && super::holder_name(alias)
}

fn domain(value: &str) -> bool {
    if value.len() > 253 || !value.is_ascii() {
        return false;
    }
    let Some((_, tld)) = value.rsplit_once('.') else {
        return false;
    };
    tld.len() >= 2
        && tld.bytes().all(|b| b.is_ascii_alphabetic())
        && value.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && label.as_bytes()[0].is_ascii_alphanumeric()
                && label.as_bytes()[label.len() - 1].is_ascii_alphanumeric()
                && label
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn year_only_copyright_lacks_required_holder() {
        // Arrange / Act / Assert
        assert!(!matches("Copyright 2024-2025\n"));
    }

    #[rstest]
    #[case::non_ascii("é.example")]
    #[case::too_long(&"a".repeat(254))]
    fn invalid_contact_domain_is_rejected(#[case] value: &str) {
        // Arrange / Act / Assert
        assert!(!domain(value));
    }
}
