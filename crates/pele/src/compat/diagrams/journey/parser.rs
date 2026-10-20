//! Translated ordered Jison-compatible scanner/parser, Pele journey and common/scan.ts (MIT).
use super::{JourneyModel, JourneyTask};
use crate::compat::{
    source::{space, word},
    PeleError,
};
#[derive(Clone, Copy, PartialEq)]
enum K {
    End,
    Eof,
    Invalid,
    Newline,
    Journey,
    Title,
    AccTitle,
    AccTitleValue,
    AccDescr,
    AccDescrValue,
    AccDescrMultiline,
    Section,
    TaskName,
    TaskData,
    Colon,
}
const NAMES: [&str; 15] = [
    "$end",
    "EOF",
    "INVALID",
    "NEWLINE",
    "journey",
    "title",
    "acc_title",
    "acc_title_value",
    "acc_descr",
    "acc_descr_value",
    "acc_descr_multiline_value",
    "section",
    "taskName",
    "taskData",
    ":",
];
struct Token {
    k: K,
    from: usize,
    to: usize,
}
fn ch(s: &str, p: usize) -> Option<char> {
    s.get(p..)?.chars().next()
}
fn keyword(s: &str, p: usize, w: &str) -> bool {
    s.get(p..p + w.len())
        .is_some_and(|v| v.eq_ignore_ascii_case(w))
}
fn line_end(s: &str, p: usize) -> usize {
    s[p..].find('\n').map_or(s.len(), |i| p + i)
}
fn skip_space(s: &str, mut p: usize) -> usize {
    while ch(s, p).is_some_and(space) {
        p += ch(s, p).unwrap().len_utf8()
    }
    p
}
fn trivia(s: &str, mut p: usize) -> usize {
    while p < s.len() {
        let c = ch(s, p).unwrap();
        let next = p + c.len_utf8();
        if c == '%' && ch(s, next) != Some('{') {
            p = line_end(s, p)
        } else if c != '}' && s[next..].starts_with("%%") {
            p = line_end(s, next + 2)
        } else if c == '\n' {
            return p;
        } else if space(c) {
            p = skip_space(s, p)
        } else if c == '#' {
            p = line_end(s, p)
        } else {
            return p;
        }
    }
    p
}
fn text_end(s: &str, p: usize, colon: bool) -> usize {
    s[p..]
        .char_indices()
        .find(|(_, c)| matches!(c, '#' | '\n' | ';') || (colon && *c == ':'))
        .map_or(s.len(), |(i, _)| p + i)
}
fn emit(out: &mut Vec<Token>, k: K, from: usize, to: usize) {
    out.push(Token { k, from, to })
}
fn tokens(s: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let mut p = 0;
    while p < s.len() {
        p = trivia(s, p);
        if p >= s.len() {
            break;
        }
        let c = ch(s, p).unwrap();
        if c == '\n' {
            let mut e = p + 1;
            while ch(s, e) == Some('\n') {
                e += 1
            }
            emit(&mut out, K::Newline, p, e);
            p = e;
            continue;
        }
        if keyword(s, p, "journey") && !ch(s, p + 7).is_some_and(word) {
            emit(&mut out, K::Journey, p, p + 7);
            p += 7;
            continue;
        }
        if keyword(s, p, "title") && ch(s, p + 5).is_some_and(space) {
            let after = p + 5 + ch(s, p + 5).unwrap().len_utf8();
            let e = text_end(s, after, false);
            if e > after {
                emit(&mut out, K::Title, p, e);
                p = e;
                continue;
            }
        }
        let descr = keyword(s, p, "accdescr");
        if descr || keyword(s, p, "acctitle") {
            let mut q = skip_space(s, p + 8);
            if ch(s, q) == Some(':') {
                q = skip_space(s, q + 1);
                let e = line_end(s, q);
                emit(
                    &mut out,
                    if descr { K::AccDescr } else { K::AccTitle },
                    p,
                    q,
                );
                emit(
                    &mut out,
                    if descr {
                        K::AccDescrValue
                    } else {
                        K::AccTitleValue
                    },
                    q,
                    e,
                );
                p = e;
                continue;
            }
            if descr && ch(s, q) == Some('{') {
                q = skip_space(s, q + 1);
                while ch(s, q) != Some('}') {
                    let e = s[q..].find('}').map_or(s.len(), |i| q + i);
                    emit(&mut out, K::AccDescrMultiline, q, e);
                    if q == s.len() {
                        break;
                    }
                    q = e
                }
                p = if q == s.len() { q } else { q + 1 };
                continue;
            }
        }
        if keyword(s, p, "section") && ch(s, p + 7).is_some_and(space) {
            let after = p + 7 + ch(s, p + 7).unwrap().len_utf8();
            let e = text_end(s, after, true);
            if e > after {
                emit(&mut out, K::Section, p, e);
                p = e;
                continue;
            }
        }
        let (k, e) = if c == ':' {
            let e = text_end(s, p + 1, false);
            if e > p + 1 {
                (K::TaskData, e)
            } else {
                (K::Colon, p + 1)
            }
        } else if c == ';' {
            (K::Invalid, p + 1)
        } else {
            (K::TaskName, text_end(s, p + c.len_utf8(), true))
        };
        emit(&mut out, k, p, e);
        p = e;
    }
    if out.last().is_none_or(|t| t.from < s.len()) {
        emit(&mut out, K::Eof, s.len(), s.len())
    }
    emit(&mut out, K::End, s.len(), s.len());
    out
}
pub(super) fn js_number(raw: &str) -> f64 {
    let raw = raw.trim_matches(space);
    if raw.is_empty() {
        return 0.;
    }
    if raw == "Infinity" || raw == "+Infinity" {
        return f64::INFINITY;
    }
    if raw == "-Infinity" {
        return f64::NEG_INFINITY;
    }
    for (prefix, radix) in [
        ("0x", 16),
        ("0X", 16),
        ("0b", 2),
        ("0B", 2),
        ("0o", 8),
        ("0O", 8),
    ] {
        if let Some(digits) = raw.strip_prefix(prefix) {
            if digits.is_empty() {
                return f64::NAN;
            }
            // JavaScript Number rounds the complete binary integer once, to nearest-even.
            // Accumulating floating point digits would round at every step for long literals.
            let mut bit_count = 0usize;
            let mut leading = 0u64;
            let mut guard = false;
            let mut sticky = false;
            let digit_bits = match radix {
                2 => 1,
                8 => 3,
                _ => 4,
            };
            for c in digits.chars() {
                let Some(digit) = c.to_digit(radix) else {
                    return f64::NAN;
                };
                for shift in (0..digit_bits).rev() {
                    let bit = digit >> shift & 1;
                    if bit_count == 0 && bit == 0 {
                        continue;
                    }
                    bit_count += 1;
                    if bit_count <= 53 {
                        leading = leading * 2 + bit as u64;
                    } else if bit_count == 54 {
                        guard = bit != 0;
                    } else {
                        sticky |= bit != 0;
                    }
                }
            }
            if bit_count > 1024 {
                return f64::INFINITY;
            }
            if bit_count <= 53 {
                return leading as f64;
            }
            if guard && (sticky || leading & 1 != 0) {
                leading += 1;
            }
            return leading as f64 * 2f64.powi((bit_count - 53) as i32);
        }
    }
    static DECIMAL: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    if !DECIMAL
        .get_or_init(|| {
            regex::Regex::new(r"^[+-]?(?:[0-9]+\.?[0-9]*|\.[0-9]+)(?:[eE][+-]?[0-9]+)?$").unwrap()
        })
        .is_match(raw)
    {
        return f64::NAN;
    }
    raw.parse().unwrap_or(f64::NAN)
}
fn description(raw: &str) -> String {
    let mut out = String::new();
    let mut after_newline = false;
    for c in raw.trim_matches(space).chars() {
        if after_newline && space(c) {
            continue;
        }
        out.push(c);
        after_newline = c == '\n'
    }
    out
}
pub(super) fn parse(src: &str, title: Option<&str>) -> Result<JourneyModel, PeleError> {
    let ts = tokens(src);
    let fail = |i: usize, expected: &str| {
        let t = &ts[i];
        PeleError::syntax(
            "journey",
            src,
            t.from,
            &format!(
                "Expecting {expected}, got '{}'",
                if t.k == K::End {
                    "1"
                } else {
                    NAMES[t.k as usize]
                }
            ),
            false,
        )
    };
    if ts[0].k != K::Journey {
        return Err(fail(0, "'journey'"));
    }
    let mut m = JourneyModel::empty(title);
    let mut section = String::new();
    let mut i = 1;
    loop {
        let t = &ts[i];
        let text = &src[t.from..t.to];
        match t.k{K::Newline=>{},K::Title=>{let at=text.char_indices().nth(6).map_or(text.len(),|v|v.0);m.title=Some(text[at..].into())},K::AccTitle=>{i+=1;m.acc_title=Some(src[ts[i].from..ts[i].to].trim_matches(space).into())},K::AccDescr=>{i+=1;m.acc_descr=Some(description(&src[ts[i].from..ts[i].to]))},K::AccDescrMultiline=>m.acc_descr=Some(description(text)),K::Section=>{let at=text.char_indices().nth(8).map_or(text.len(),|v|v.0);section=text[at..].into();m.sections.push(section.clone());
                m.current_section = section.clone()},K::TaskName=>{i+=1;if ts[i].k!=K::TaskData{return Err(fail(i,"'taskData'"))}let pieces:Vec<_>=src[ts[i].from+1..ts[i].to].split(':').collect();m.tasks.push(JourneyTask{task:text.into(),score:js_number(pieces[0]),people:if pieces.len()==1{vec![]}else{pieces[1].split(',').map(|s|s.trim_matches(space).into()).collect()},section:section.clone()})},K::Eof=>return Ok(m),_=>return Err(fail(i,"'EOF', 'SPACE', 'NEWLINE', 'title', 'acc_title', 'acc_descr', 'acc_descr_multiline_value', 'section', 'taskName'"))}
        i += 1
    }
}
