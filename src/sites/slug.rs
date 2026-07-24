const ELIDED: [char; 3] = ['\'', '\u{2019}', '\u{02bc}'];

pub fn hyphenated(input: &str) -> String {
    let mut out = String::new();
    let mut last_was_sep = true;
    for ch in input.to_lowercase().chars() {
        if ELIDED.contains(&ch) {
            continue;
        }
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            last_was_sep = false;
        } else if !last_was_sep {
            out.push('-');
            last_was_sep = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

pub fn compact(input: &str) -> String {
    input
        .to_lowercase()
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .collect()
}

pub fn title_matches(page_title: &str, expected_title: &str) -> bool {
    let expected_slug = hyphenated(expected_title);
    !expected_slug.is_empty() && hyphenated(page_title).contains(&expected_slug)
}

pub fn fold_turkish(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            'ç' | 'Ç' => 'c',
            'ğ' | 'Ğ' => 'g',
            'ı' | 'İ' => 'i',
            'ö' | 'Ö' => 'o',
            'ş' | 'Ş' => 's',
            'ü' | 'Ü' => 'u',
            other => other,
        })
        .collect()
}

pub fn strip_leading_the(input: &str) -> &str {
    let trimmed = input.trim();
    if trimmed.to_lowercase().starts_with("the ") {
        return trimmed[4..].trim_start();
    }
    trimmed
}
