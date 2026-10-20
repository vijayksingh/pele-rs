//! Pele timeline/{lexer,parser,db}.ts and common/scan.ts, 458322a5 (MIT, kepano).
use super::{TimelineModel, TimelinePeriod};
use crate::compat::{
    source::{space, word},
    ErrorCode, PeleError,
};
#[derive(Clone, Copy, PartialEq)]
enum T {
    End,
    Eof,
    Invalid,
    Newline,
    Timeline,
    Lr,
    Td,
    Title,
    AccTitle,
    AccTitleValue,
    AccDescr,
    AccDescrValue,
    AccMultiline,
    Section,
    Period,
    Event,
}
const NAMES: [&str; 16] = [
    "$end",
    "EOF",
    "INVALID",
    "NEWLINE",
    "timeline",
    "timeline_lr",
    "timeline_td",
    "title",
    "acc_title",
    "acc_title_value",
    "acc_descr",
    "acc_descr_value",
    "acc_descr_multiline_value",
    "section",
    "period",
    "event",
];
struct Token {
    kind: T,
    start: usize,
    end: usize,
}
fn ch(s: &str, p: usize) -> Option<char> {
    s.get(p..)?.chars().next()
}
fn next(s: &str, p: usize) -> usize {
    p + ch(s, p).map_or(0, char::len_utf8)
}
fn keyword(s: &str, p: usize, k: &str) -> bool {
    s.get(p..p + k.len())
        .is_some_and(|v| v.eq_ignore_ascii_case(k))
}
fn line_end(s: &str, p: usize) -> usize {
    s[p..].find('\n').map_or(s.len(), |i| p + i)
}
fn skip_space(s: &str, mut p: usize) -> usize {
    while ch(s, p).is_some_and(space) {
        p = next(s, p)
    }
    p
}
fn trivia(s: &str, mut p: usize) -> usize {
    while p < s.len() {
        let c = ch(s, p).unwrap();
        if c == '%' && ch(s, p + 1) != Some('{') {
            p = line_end(s, p)
        } else if c != '}' && c.len_utf16() == 1 && s[next(s, p)..].starts_with("%%") {
            p = line_end(s, next(s, p) + 2)
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
fn tokenize(s: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let mut p = 0;
    while p < s.len() {
        p = trivia(s, p);
        if p >= s.len() {
            break;
        }
        let start = p;
        let c = ch(s, p).unwrap();
        let (kind, end) = if c == '\n' {
            let mut e = p + 1;
            while ch(s, e) == Some('\n') {
                e += 1
            }
            (T::Newline, e)
        } else if keyword(s, p, "timeline") {
            let mut e = p + 8;
            while matches!(ch(s, e), Some(' ' | '\t')) {
                e += 1
            }
            let lr = keyword(s, e, "lr");
            if e > p + 8 && (lr || keyword(s, e, "td")) && !ch(s, e + 2).is_some_and(word) {
                (if lr { T::Lr } else { T::Td }, e + 2)
            } else if !ch(s, p + 8).is_some_and(word) {
                (T::Timeline, p + 8)
            } else {
                period(s, p)
            }
        } else if keyword(s, p, "title")
            && ch(s, p + 5).is_some_and(space)
            && next(s, p + 5) < s.len()
            && ch(s, next(s, p + 5)) != Some('\n')
        {
            (T::Title, line_end(s, next(s, p + 5)))
        } else {
            let descr = keyword(s, p, "accdescr");
            if descr || keyword(s, p, "acctitle") {
                let mut q = skip_space(s, p + 8);
                if ch(s, q) == Some(':') {
                    q = skip_space(s, q + 1);
                    let end = line_end(s, q);
                    out.push(Token {
                        kind: if descr { T::AccDescr } else { T::AccTitle },
                        start: p,
                        end: q,
                    });
                    out.push(Token {
                        kind: if descr {
                            T::AccDescrValue
                        } else {
                            T::AccTitleValue
                        },
                        start: q,
                        end,
                    });
                    p = end;
                    continue;
                } else if descr && ch(s, q) == Some('{') {
                    q = skip_space(s, q + 1);
                    while ch(s, q) != Some('}') {
                        let end = s[q..].find('}').map_or(s.len(), |i| q + i);
                        out.push(Token {
                            kind: T::AccMultiline,
                            start: q,
                            end,
                        });
                        if q == s.len() {
                            break;
                        }
                        q = end
                    }
                    p = if q < s.len() { q + 1 } else { q };
                    continue;
                }
            }
            if keyword(s, p, "section") && ch(s, p + 7).is_some_and(space) {
                let mut e = next(s, p + 7);
                while e < s.len() && !matches!(ch(s, e), Some(':' | '\n')) {
                    e = next(s, e)
                }
                if e > next(s, p + 7) {
                    (T::Section, e)
                } else {
                    period_or_event(s, p)
                }
            } else {
                period_or_event(s, p)
            }
        };
        out.push(Token { kind, start, end });
        p = end
    }
    if out.last().is_none_or(|t| t.start < s.len()) {
        out.push(Token {
            kind: T::Eof,
            start: s.len(),
            end: s.len(),
        })
    }
    out.push(Token {
        kind: T::End,
        start: s.len(),
        end: s.len(),
    });
    out
}
fn period(s: &str, p: usize) -> (T, usize) {
    let mut e = next(s, p);
    while e < s.len() && !matches!(ch(s, e), Some('#' | ':' | '\n')) {
        e = next(s, e)
    }
    (T::Period, e)
}
fn period_or_event(s: &str, p: usize) -> (T, usize) {
    if ch(s, p) != Some(':') {
        return period(s, p);
    }
    if ch(s, p + 1).is_some_and(space) {
        let mut e = next(s, p + 1);
        while e < s.len() {
            if ch(s, e) == Some('\n') || (ch(s, e) == Some(':') && ch(s, e + 1).is_some_and(space))
            {
                break;
            }
            e = next(s, e)
        }
        if e > next(s, p + 1) {
            return (T::Event, e);
        }
    }
    (T::Invalid, p + 1)
}
pub(super) fn parse(s: &str, title: Option<&str>) -> Result<TimelineModel, PeleError> {
    let tokens = tokenize(s);
    let fail = |i: usize, expected: &str| {
        let t = &tokens[i];
        let got = if t.kind == T::End {
            "1"
        } else {
            NAMES[t.kind as usize]
        };
        PeleError::syntax(
            "timeline",
            s,
            t.start,
            &format!("Expecting {expected}, got '{got}'"),
            false,
        )
    };
    let mut m = TimelineModel::empty(title);
    match tokens[0].kind {
        T::Timeline | T::Lr => {}
        T::Td => m.direction = "TD".into(),
        _ => return Err(fail(0, "'timeline', 'timeline_lr', 'timeline_td'")),
    }
    let mut i = 1;
    while i < tokens.len() {
        let t = &tokens[i];
        let text = &s[t.start..t.end];
        match t.kind {
            T::Newline => {},
            T::Title => m.title = Some(text[next(text, 5)..].into()),
            T::AccTitle | T::AccDescr => {
                let kind = t.kind;
                i += 1;
                let value = &tokens[i];
                let text = s[value.start..value.end].trim_matches(space);
                if kind == T::AccTitle { m.acc_title = Some(text.into()); }
                else { m.acc_descr = Some(description(text)); }
            },
            T::AccMultiline => m.acc_descr = Some(description(text.trim_matches(space))),
            T::Section => {
                m.current_section = text[next(text, 7)..].into();
                m.sections.push(m.current_section.clone());
            },
            T::Period => m.periods.push(TimelinePeriod {
                text: text.into(), events: Vec::new(),
                section: m.current_section.clone(),
                section_index: m.sections.len() as i64 - 1,
            }),
            T::Event => {
                let Some(period) = m.periods.last_mut() else {
                    let mut error = PeleError::new("An event needs a time period before it.", ErrorCode::Semantic);
                    error.diagram_type = Some("timeline".into());
                    return Err(error);
                };
                period.events.push(text[next(text, 1)..].into());
            },
            T::Eof => return Ok(m),
            _ => return Err(fail(i, "'EOF', 'SPACE', 'NEWLINE', 'title', 'acc_title', 'acc_descr', 'acc_descr_multiline_value', 'section', 'period', 'event'")),
        }
        i += 1
    }
    Ok(m)
}
fn description(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        out.push(c);
        if c == '\n' {
            while chars.peek().is_some_and(|c| space(*c)) {
                chars.next();
            }
        }
    }
    out
}
