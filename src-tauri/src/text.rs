use crate::models::Item;
use unicode_normalization::UnicodeNormalization;

/// Removes HTML tags and decodes the most common entities.
pub fn strip_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    for c in input.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    decode_entities(&out)
}

fn decode_entities(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(idx) = rest.find('&') {
        out.push_str(&rest[..idx]);
        let after = &rest[idx..];
        let Some(end) = after.find(';') else {
            out.push('&');
            rest = &after[1..];
            continue;
        };
        let entity = &after[1..end];
        let decoded = match entity {
            "nbsp" => Some(' '),
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            _ if entity.starts_with("#x") || entity.starts_with("#X") => {
                u32::from_str_radix(&entity[2..], 16)
                    .ok()
                    .and_then(char::from_u32)
            }
            _ if entity.starts_with('#') => entity[1..].parse::<u32>().ok().and_then(char::from_u32),
            _ => None,
        };
        if let Some(c) = decoded {
            out.push(c);
            rest = &after[end + 1..];
        } else {
            out.push('&');
            rest = &after[1..];
        }
    }
    out.push_str(rest);
    out
}

fn strip_diacritics(input: &str) -> String {
    input
        .nfd()
        .filter(|c| !unicode_normalization::char::is_combining_mark(*c))
        .collect()
}

/// Display-ready text: no HTML, NFKC, collapsed spaces.
pub fn clean_display(input: &str) -> String {
    let stripped = strip_html(input);
    let normalized: String = stripped.nfkc().collect();
    let mut out = String::with_capacity(normalized.len());
    let mut last_space = false;
    for c in normalized.chars() {
        let c = if matches!(c, '\u{3000}' | '\n' | '\r' | '\t') {
            ' '
        } else {
            c
        };
        if c == ' ' {
            if !last_space {
                out.push(' ');
            }
            last_space = true;
        } else {
            out.push(c);
            last_space = false;
        }
    }
    out.trim().to_string()
}

/// Key to deduplicate items (no spaces or punctuation).
pub fn normalize_key(input: &str) -> String {
    let cleaned = clean_display(input);
    let folded = strip_diacritics(&cleaned).to_lowercase();
    folded
        .chars()
        .filter(|c| c.is_alphanumeric() || has_japanese_char(*c))
        .collect()
}

const IGNORED_ANSWER_CHARS: &[char] = &[
    '。', '、', '.', ',', ';', '；', ':', '：', '!', '?', '！', '？', '"', '\'', '“', '”', '‘', '’',
    '*', '〜', '~', '·', '-',
];

/// Normalizes an answer to compare it with the accepted variants.
pub fn normalize_answer(input: &str) -> String {
    let cleaned = clean_display(input);
    let folded = strip_diacritics(&cleaned).to_lowercase();
    folded
        .chars()
        .filter(|c| !c.is_whitespace() && !IGNORED_ANSWER_CHARS.contains(c))
        .collect()
}

pub fn has_japanese_char(c: char) -> bool {
    let u = c as u32;
    (0x3040..=0x30FF).contains(&u)
        || (0x3400..=0x4DBF).contains(&u)
        || (0x4E00..=0x9FFF).contains(&u)
        || (0xF900..=0xFAFF).contains(&u)
        || (0xFF66..=0xFF9D).contains(&u)
        || matches!(u, 0x3005 | 0x3006 | 0x30FC)
}

pub fn has_japanese(input: &str) -> bool {
    input.chars().any(has_japanese_char)
}

pub fn remove_parentheticals(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut depth = 0i32;
    for c in input.chars() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out
}

fn parenthetical_contents(input: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut current = String::new();
    for c in input.chars() {
        match c {
            '(' => {
                if depth > 0 {
                    current.push(c);
                }
                depth += 1;
            }
            ')' => {
                if depth > 1 {
                    current.push(c);
                }
                depth -= 1;
                if depth == 0 {
                    out.push(std::mem::take(&mut current));
                }
            }
            _ if depth > 0 => current.push(c),
            _ => {}
        }
    }
    out
}

fn push_variant(out: &mut Vec<String>, raw: &str) {
    let mut value = raw.trim().to_string();
    value = value.replace(['〜', '~', '*'], "");
    value = value
        .trim_matches(|c: char| {
            matches!(c, '。' | '.' | ',' | '、' | ';' | '；' | ':' | '：' | '(' | ')' | '[' | ']')
                || c.is_whitespace()
        })
        .to_string();
    value = value.trim().to_string();
    if value.is_empty() || !has_japanese(&value) {
        return;
    }
    if !out.iter().any(|v| v == &value) {
        out.push(value);
    }
}

/// All acceptable Japanese forms (kanji and readings).
pub fn japanese_variants(item: &Item) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(kanji) = item.kanji.as_deref() {
        add_japanese_variants(&mut out, kanji);
    }
    add_japanese_variants(&mut out, &item.reading);
    out
}

fn add_japanese_variants(out: &mut Vec<String>, raw: &str) {
    let cleaned = clean_display(raw);
    if cleaned.is_empty() {
        return;
    }

    let without = remove_parentheticals(&cleaned);
    push_variant(out, &without);
    push_variant(out, &cleaned);

    for inner in parenthetical_contents(&cleaned) {
        for part in inner.split('/') {
            push_variant(out, part);
        }
    }

    for token in cleaned.split(|c: char| c == '/' || c == ';' || c.is_whitespace()) {
        let token = token.trim_matches(|c: char| {
            matches!(c, '。' | '.' | ',' | '、' | ';' | '；' | ':' | '：')
        });
        if token.is_empty() {
            continue;
        }
        let base = remove_parentheticals(token);
        push_variant(out, &base);
        for inner in parenthetical_contents(token) {
            push_variant(out, &inner);
            if !base.is_empty() {
                let combined = format!("{base}{inner}");
                push_variant(out, &combined);
            }
        }
    }
}

fn push_meaning(out: &mut Vec<String>, raw: &str) {
    let value = clean_display(raw);
    let value = value.trim_matches(|c: char| {
        matches!(c, '。' | '.' | ',' | '、' | ';' | '；' | ':' | '：')
    });
    if value.is_empty() || value == "..." || value == "…" {
        return;
    }
    if !out.iter().any(|v| v == value) {
        out.push(value.to_string());
    }

    let without_placeholder = value.replace("...", "").replace('…', "");
    let without_placeholder = without_placeholder
        .trim_matches(|c: char| {
            matches!(c, '。' | '.' | ',' | '、' | ';' | '；' | ':' | '：' | '-' | '—')
                || c.is_whitespace()
        })
        .to_string();
    if !without_placeholder.is_empty() && !out.iter().any(|v| v == &without_placeholder) {
        out.push(without_placeholder);
    }
}

/// Acceptable meanings in JP → ES.
pub fn meaning_variants(item: &Item) -> Vec<String> {
    let mut out = Vec::new();
    let cleaned = clean_display(&item.meaning);

    push_meaning(&mut out, &cleaned);
    for part in cleaned.split(';') {
        push_meaning(&mut out, part);
        for sub in part.split(" / ") {
            push_meaning(&mut out, sub);
        }
    }

    let without = remove_parentheticals(&cleaned);
    push_meaning(&mut out, &without);
    for part in without.split(';') {
        for sub in part.split(" / ") {
            push_meaning(&mut out, sub);
        }
    }

    for part in cleaned.split(';') {
        let part = remove_parentheticals(part);
        for sub in part.split(" / ") {
            push_meaning(&mut out, sub);
        }
    }

    out
}

fn kana_to_hiragana(c: char) -> char {
    let code = c as u32;
    if (0x30A1..=0x30F6).contains(&code) {
        char::from_u32(code - 0x60).unwrap_or(c)
    } else {
        c
    }
}

fn single_romaji(c: char) -> Option<&'static str> {
    Some(match c {
        'あ' => "a",
        'い' => "i",
        'う' => "u",
        'え' => "e",
        'お' => "o",
        'か' => "ka",
        'き' => "ki",
        'く' => "ku",
        'け' => "ke",
        'こ' => "ko",
        'さ' => "sa",
        'し' => "shi",
        'す' => "su",
        'せ' => "se",
        'そ' => "so",
        'た' => "ta",
        'ち' => "chi",
        'つ' => "tsu",
        'て' => "te",
        'と' => "to",
        'な' => "na",
        'に' => "ni",
        'ぬ' => "nu",
        'ね' => "ne",
        'の' => "no",
        'は' => "ha",
        'ひ' => "hi",
        'ふ' => "fu",
        'へ' => "he",
        'ほ' => "ho",
        'ま' => "ma",
        'み' => "mi",
        'む' => "mu",
        'め' => "me",
        'も' => "mo",
        'や' => "ya",
        'ゆ' => "yu",
        'よ' => "yo",
        'ら' => "ra",
        'り' => "ri",
        'る' => "ru",
        'れ' => "re",
        'ろ' => "ro",
        'わ' => "wa",
        'を' => "o",
        'ん' => "n",
        'が' => "ga",
        'ぎ' => "gi",
        'ぐ' => "gu",
        'げ' => "ge",
        'ご' => "go",
        'ざ' => "za",
        'じ' => "ji",
        'ず' => "zu",
        'ぜ' => "ze",
        'ぞ' => "zo",
        'だ' => "da",
        'ぢ' => "ji",
        'づ' => "zu",
        'で' => "de",
        'ど' => "do",
        'ば' => "ba",
        'び' => "bi",
        'ぶ' => "bu",
        'べ' => "be",
        'ぼ' => "bo",
        'ぱ' => "pa",
        'ぴ' => "pi",
        'ぷ' => "pu",
        'ぺ' => "pe",
        'ぽ' => "po",
        'ゔ' => "vu",
        'ゐ' => "wi",
        'ゑ' => "we",
        _ => return None,
    })
}

const DIGRAPHS: &[(&str, &str)] = &[
    ("きゃ", "kya"),
    ("きゅ", "kyu"),
    ("きょ", "kyo"),
    ("しゃ", "sha"),
    ("しゅ", "shu"),
    ("しょ", "sho"),
    ("ちゃ", "cha"),
    ("ちゅ", "chu"),
    ("ちょ", "cho"),
    ("にゃ", "nya"),
    ("にゅ", "nyu"),
    ("にょ", "nyo"),
    ("ひゃ", "hya"),
    ("ひゅ", "hyu"),
    ("ひょ", "hyo"),
    ("みゃ", "mya"),
    ("みゅ", "myu"),
    ("みょ", "myo"),
    ("りゃ", "rya"),
    ("りゅ", "ryu"),
    ("りょ", "ryo"),
    ("ぎゃ", "gya"),
    ("ぎゅ", "gyu"),
    ("ぎょ", "gyo"),
    ("じゃ", "ja"),
    ("じゅ", "ju"),
    ("じょ", "jo"),
    ("びゃ", "bya"),
    ("びゅ", "byu"),
    ("びょ", "byo"),
    ("ぴゃ", "pya"),
    ("ぴゅ", "pyu"),
    ("ぴょ", "pyo"),
    ("ふぁ", "fa"),
    ("ふぃ", "fi"),
    ("ふぇ", "fe"),
    ("ふぉ", "fo"),
    ("てぃ", "ti"),
    ("でぃ", "di"),
    ("しぇ", "she"),
    ("じぇ", "je"),
    ("ちぇ", "che"),
    ("いぇ", "ye"),
    ("きぇ", "kye"),
    ("ぎぇ", "gye"),
    ("にぇ", "nye"),
    ("ひぇ", "hye"),
    ("びぇ", "bye"),
    ("ぴぇ", "pye"),
    ("みぇ", "mye"),
    ("りぇ", "rye"),
    ("うぃ", "wi"),
    ("うぇ", "we"),
    ("うぉ", "wo"),
    ("ゔぁ", "va"),
    ("ゔぃ", "vi"),
    ("ゔぇ", "ve"),
    ("ゔぉ", "vo"),
];

/// Converts kana to romaji (approximate Hepburn). Kanji are left untouched.
pub fn to_romaji(input: &str) -> String {
    let normalized: String = input.nfkc().collect();
    let chars: Vec<char> = normalized.chars().map(kana_to_hiragana).collect();
    let mut out = String::with_capacity(chars.len() * 2);
    let mut i = 0;
    let mut double_next = false;

    while i < chars.len() {
        let c = chars[i];

        if c == 'っ' {
            double_next = true;
            i += 1;
            continue;
        }

        if c == 'ー' {
            if let Some(vowel) = out
                .chars()
                .rev()
                .find(|ch| matches!(ch, 'a' | 'i' | 'u' | 'e' | 'o'))
            {
                out.push(vowel);
            }
            i += 1;
            continue;
        }

        if c == 'ん' {
            out.push('n');
            i += 1;
            continue;
        }

        let digraph = chars.get(i + 1).and_then(|next| {
            let key: String = [c, *next].iter().collect();
            DIGRAPHS
                .iter()
                .find(|(kana, _)| *kana == key.as_str())
                .map(|(_, romaji)| *romaji)
        });

        let (romaji, consumed) = match digraph {
            Some(romaji) => (romaji, 2),
            None => match single_romaji(c) {
                Some(romaji) => (romaji, 1),
                None => {
                    out.push(c);
                    i += 1;
                    continue;
                }
            },
        };

        if double_next {
            if romaji.starts_with("ch") {
                out.push('t');
            } else if let Some(first) = romaji.chars().next() {
                out.push(first);
            }
            double_next = false;
        }

        out.push_str(romaji);
        i += consumed;
    }

    out
}

/// Acceptable romaji variants: Hepburn, collapsed long vowels, kunrei and nasal.
pub fn romaji_variants(input: &str) -> Vec<String> {
    let base = to_romaji(input);
    if base.is_empty() {
        return Vec::new();
    }

    let mut out = vec![base.clone()];

    let mut collapsed = base.clone();
    for (from, to) in [
        ("ou", "o"),
        ("oo", "o"),
        ("aa", "a"),
        ("uu", "u"),
        ("ee", "e"),
        ("ei", "e"),
    ] {
        collapsed = collapsed.replace(from, to);
    }
    if !out.contains(&collapsed) {
        out.push(collapsed.clone());
    }

    let mut kunrei = collapsed.clone();
    for (from, to) in [
        ("shi", "si"),
        ("chi", "ti"),
        ("tsu", "tu"),
        ("fu", "hu"),
        ("ji", "zi"),
        ("sha", "sya"),
        ("shu", "syu"),
        ("sho", "syo"),
        ("cha", "tya"),
        ("chu", "tyu"),
        ("cho", "tyo"),
        ("ja", "zya"),
        ("ju", "zyu"),
        ("jo", "zyo"),
    ] {
        kunrei = kunrei.replace(from, to);
    }
    if !out.contains(&kunrei) {
        out.push(kunrei);
    }

    let mut nasal = collapsed.clone();
    for (from, to) in [("nb", "mb"), ("np", "mp"), ("nm", "mm")] {
        nasal = nasal.replace(from, to);
    }
    if !out.contains(&nasal) {
        out.push(nasal);
    }

    if base.ends_with("ha") {
        let wa = format!("{}wa", &base[..base.len() - 2]);
        if !out.contains(&wa) {
            out.push(wa);
        }
    }

    out
}

pub fn check_es_to_jp(answer: &str, item: &Item) -> bool {
    let answer = normalize_answer(answer);
    if answer.is_empty() {
        return false;
    }

    let variants = japanese_variants(item);
    if variants
        .iter()
        .any(|variant| normalize_answer(variant) == answer)
    {
        return true;
    }

    // If it is not romaji, there is nothing else to try.
    if !answer.chars().all(|c| c.is_ascii_alphabetic()) {
        return false;
    }

    variants.iter().any(|variant| {
        romaji_variants(variant)
            .iter()
            .any(|romaji| normalize_answer(romaji) == answer)
    })
}

pub fn check_jp_to_es(answer: &str, item: &Item) -> bool {
    let answer = normalize_answer(answer);
    if answer.is_empty() {
        return false;
    }
    meaning_variants(item)
        .iter()
        .any(|variant| normalize_answer(variant) == answer)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(kanji: Option<&str>, reading: &str, meaning: &str) -> Item {
        Item {
            id: 1,
            kanji: kanji.map(str::to_string),
            reading: reading.to_string(),
            meaning: meaning.to_string(),
            kanji_gloss: None,
            lesson: String::new(),
            category: String::new(),
            is_pattern: false,
            irregular: false,
            has_audio: false,
        }
    }

    #[test]
    fn strips_html_and_entities() {
        assert_eq!(
            strip_html("ir (<i>destino</i> に)"),
            "ir (destino に)"
        );
        assert_eq!(strip_html("veinte&nbsp;años"), "veinte años");
        assert_eq!(strip_html("a &amp; b"), "a & b");
    }

    #[test]
    fn normalizes_answers() {
        assert_eq!(normalize_answer("  Buenos días. "), "buenosdias");
        assert_eq!(normalize_answer("おはよう。"), "おはよう");
        assert_eq!(normalize_answer("Tシャツ"), "tシャツ");
    }

    #[test]
    fn accepts_readings_and_kanji() {
        let item = item(Some("四"), "よん／し／（よ）", "4");
        assert!(check_es_to_jp("四", &item));
        assert!(check_es_to_jp("よん", &item));
        assert!(check_es_to_jp("し", &item));
        assert!(check_es_to_jp("よ", &item));
        assert!(!check_es_to_jp("ご", &item));
    }

    #[test]
    fn accepts_optional_parentheticals() {
        let item = item(None, "おやすみ（なさい）。", "Buenas noches.");
        assert!(check_es_to_jp("おやすみ", &item));
        assert!(check_es_to_jp("おやすみなさい", &item));
        assert!(check_es_to_jp("おやすみなさい。", &item));
    }

    #[test]
    fn accepts_multiple_meanings() {
        let item = item(Some("聞く"), "きく", "escuchar; oír (〜を); preguntar (<i>persona</i> に)");
        assert!(check_jp_to_es("escuchar", &item));
        assert!(check_jp_to_es("oír", &item));
        assert!(check_jp_to_es("oir", &item));
        assert!(check_jp_to_es("preguntar", &item));
        assert!(!check_jp_to_es("hablar", &item));
    }

    #[test]
    fn accepts_meaning_without_placeholders() {
        let item = item(None, "〜です。", "Soy ...");
        assert!(check_jp_to_es("Soy", &item));
    }

    #[test]
    fn handles_multiple_kanji_forms() {
        let item = item(Some("暑い (weather) 熱い (thing)"), "あつい", "caluroso; caliente");
        assert!(check_es_to_jp("暑い", &item));
        assert!(check_es_to_jp("熱い", &item));
        assert!(check_es_to_jp("あつい", &item));
    }

    #[test]
    fn romanizes_kana() {
        assert_eq!(to_romaji("がっこう"), "gakkou");
        assert_eq!(to_romaji("しゃしん"), "shashin");
        assert_eq!(to_romaji("コーヒー"), "koohii");
        assert_eq!(to_romaji("こんにちは"), "konnichiha");
        assert!(romaji_variants("こんにちは").contains(&"konnichiwa".to_string()));
    }

    #[test]
    fn accepts_romaji_answers() {
        let item = item(Some("おはよう"), "おはよう", "Buenos días.");
        assert!(check_es_to_jp("ohayou", &item));
        assert!(check_es_to_jp("ohayo", &item));
        assert!(check_es_to_jp("Ohayō", &item));
        assert!(!check_es_to_jp("ohayouu", &item));
    }

    #[test]
    fn accepts_romaji_long_vowels_and_small_tsu() {
        let item = item(Some("学校"), "がっこう", "escuela");
        assert!(check_es_to_jp("gakkou", &item));
        assert!(check_es_to_jp("gakko", &item));
        assert!(check_es_to_jp("gakkō", &item));
    }

    #[test]
    fn accepts_romaji_ha_as_wa() {
        let item = item(None, "こんにちは", "Buenas tardes.");
        assert!(check_es_to_jp("konnichiha", &item));
        assert!(check_es_to_jp("konnichiwa", &item));
    }
}
