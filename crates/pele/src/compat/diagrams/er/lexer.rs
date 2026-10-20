//! Direct Pele er/lexer.ts scanner, 458322a5 (MIT, kepano).
use super::tokens::T;
use crate::compat::source::space;
#[derive(Clone, Debug)]
pub(super) struct Token<'a> {
    pub kind: T,
    pub text: &'a str,
    pub start: usize,
}
#[derive(Clone, Copy, PartialEq)]
enum S {
    Initial,
    Block,
    Backtick,
    Title,
    Descr,
    Multiline,
    Style,
}
fn word(c: Option<char>) -> bool {
    c.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
}
fn text(c: char, style: bool) -> bool {
    c as u32 >= 128 || c.is_ascii_alphanumeric() || "_-*".contains(c) || (!style && c == '.')
}
fn attr(c: char, start: bool) -> bool {
    let u = c as u32;
    if u >= 0xc0 || u == 0xb5 {
        return true;
    }
    if start {
        c.is_ascii_alphabetic() || c == '_' || c == '*'
    } else {
        c.is_ascii_alphanumeric() || "_-*.[](),".contains(c)
    }
}
struct Scanner<'a> {
    src: &'a str,
    p: usize,
    state: S,
    out: Vec<Token<'a>>,
    failed_dir: usize,
    tilde_next: Option<usize>,
    whitespace_next: usize,
    tilde_line: usize,
    tilde_last: Option<usize>,
}
impl<'a> Scanner<'a> {
    fn ch(&self, at: usize) -> Option<char> {
        self.src.get(at..)?.chars().next()
    }
    fn kw(&self, w: &str, bound: bool) -> bool {
        self.src
            .get(self.p..self.p + w.len())
            .is_some_and(|s| s.eq_ignore_ascii_case(w))
            && (!bound || !word(self.ch(self.p + w.len())))
    }
    fn emit(&mut self, t: T, end: usize) {
        self.out.push(Token {
            kind: t,
            text: &self.src[self.p..end],
            start: self.p,
        });
        self.p = end
    }
    fn until(&self, start: usize, pred: impl Fn(char) -> bool) -> usize {
        let mut q = start;
        while let Some(c) = self.ch(q) {
            if !pred(c) {
                break;
            }
            q += c.len_utf8()
        }
        q
    }
    fn direction(&mut self) -> bool {
        if self.p < self.failed_dir {
            return false;
        }
        let end = self.src[self.p..]
            .find(['\n', '\r', '\u{2028}', '\u{2029}'])
            .map_or(self.src.len(), |i| i + self.p);
        let line = &self.src[self.p..end];
        let lower = line.to_ascii_lowercase();
        for (dir, t) in [
            ("TB", T::direction_tb),
            ("BT", T::direction_bt),
            ("RL", T::direction_rl),
            ("LR", T::direction_lr),
        ] {
            for (at, _) in lower.rmatch_indices("direction") {
                let mut q = self.p + at + 9;
                let ws = q;
                q = self.until(q, space);
                if q == ws {
                    continue;
                }
                if self
                    .src
                    .get(q..q + 2)
                    .is_some_and(|s| s.eq_ignore_ascii_case(dir))
                {
                    let tail = self.src[q + 2..]
                        .find('\n')
                        .map_or(self.src.len(), |i| i + q + 2);
                    self.emit(t, tail);
                    return true;
                }
            }
        }
        self.failed_dir = end;
        false
    }
    fn initial(&mut self, c: char) {
        if matches!(c, 'a' | 'A') && self.kw("acc", false) {
            for (key, state, t) in [
                ("accTitle", S::Title, T::acc_title),
                ("accDescr", S::Descr, T::acc_descr),
                ("accDescr", S::Multiline, T::acc_descr_multiline_value),
            ] {
                if !self.kw(key, false) {
                    continue;
                }
                let q = self.until(self.p + key.len(), space);
                let delimiter = if state == S::Multiline { '{' } else { ':' };
                if self.ch(q) == Some(delimiter) {
                    let end = self.until(q + 1, space);
                    self.state = state;
                    if state == S::Multiline {
                        self.p = end
                    } else {
                        self.emit(t, end)
                    }
                    return;
                }
            }
        }
        if self.direction() {
            return;
        }
        match c {
            ' ' | '\t' | '\r' => {
                self.p = self.until(self.p, |c| matches!(c, ' ' | '\t' | '\r'));
                return;
            }
            '\n' => {
                self.emit(T::NEWLINE, self.until(self.p, |c| c == '\n'));
                return;
            }
            '"' => {
                if let Some(i) = self.src[self.p + 1..].find('"') {
                    let e = self.p + 1 + i;
                    let inner = &self.src[self.p + 1..e];
                    let name = !inner.is_empty()
                        && !inner.contains(['%', '\\', '\r', '\n', '\u{b}', '\u{8}']);
                    self.emit(if name { T::ENTITY_NAME } else { T::WORD }, e + 1)
                } else {
                    self.emit(T::CHAR, self.p + 1)
                }
                return;
            }
            '{' => {
                self.state = S::Block;
                self.emit(T::BLOCK_START, self.p + 1);
                return;
            }
            '#' => {
                self.emit(T::BRKT, self.p + 1);
                return;
            }
            ',' => {
                self.emit(T::COMMA, self.p + 1);
                return;
            }
            ':' => {
                let triple = self.src[self.p..].starts_with(":::");
                self.emit(
                    if triple { T::STYLE_SEPARATOR } else { T::COLON },
                    self.p + if triple { 3 } else { 1 },
                );
                return;
            }
            '[' => {
                self.emit(T::SQS, self.p + 1);
                return;
            }
            ']' => {
                self.emit(T::SQE, self.p + 1);
                return;
            }
            _ => {}
        }
        if c.is_ascii_digit() {
            let d = self.ch(self.p + 1);
            if d == Some('+') && matches!(c, '0' | '1') {
                self.emit(
                    if c == '1' {
                        T::ONE_OR_MORE
                    } else {
                        T::ZERO_OR_MORE
                    },
                    self.p + 2,
                );
                return;
            }
            let q = self.until(self.p + 1, |c| c.is_ascii_digit());
            if self.ch(q) == Some('.') && self.ch(q + 1).is_some_and(|c| c.is_ascii_digit()) {
                self.emit(T::DECIMAL_NUM, self.until(q + 2, |c| c.is_ascii_digit()));
                return;
            }
            if c == '1' {
                if d.is_some_and(space) {
                    let r = self.until(self.p + 1 + d.unwrap().len_utf8(), space);
                    if word(self.ch(r)) || matches!(self.ch(r), Some('"' | '\'')) {
                        self.emit(T::ONLY_ONE, self.p + 1);
                        return;
                    }
                } else if matches!(d, Some('-' | '.'))
                    && matches!(self.ch(self.p + 2), Some('-' | '.'))
                {
                    self.emit(T::ONLY_ONE, self.p + 1);
                    return;
                }
                if !word(d) {
                    self.emit(T::ENTITY_ONE, self.p + 1);
                    return;
                }
            }
            self.emit(T::NUM, q);
            return;
        }
        // Lexical precedence follows keyword(), including cardinality spellings before generic names.
        for (kw, t, bound) in [
            ("erdiagram", T::ER_DIAGRAM, true),
            ("end", T::END_KW, true),
            ("style", T::STYLE, true),
            ("subgraph", T::SUBGRAPH, true),
            ("classdef", T::CLASSDEF, true),
            ("class", T::CLASS, true),
            ("one or zero", T::ZERO_OR_ONE, true),
            ("one or more", T::ONE_OR_MORE, true),
            ("one or many", T::ONE_OR_MORE, true),
            ("one", T::ONLY_ONE, true),
            ("only one", T::ONLY_ONE, true),
            ("optionally to", T::NON_IDENTIFYING, true),
            ("zero or one", T::ZERO_OR_ONE, true),
            ("zero or more", T::ZERO_OR_MORE, true),
            ("zero or many", T::ZERO_OR_MORE, true),
            ("many(0)", T::ZERO_OR_MORE, false),
            ("many(1)", T::ONE_OR_MORE, false),
            ("many", T::ZERO_OR_MORE, true),
            ("to", T::IDENTIFYING, true),
        ] {
            if self.kw(kw, bound) {
                if matches!(t, T::STYLE | T::CLASSDEF) {
                    self.state = S::Style
                }
                let end = if t == T::END_KW {
                    self.until(self.p + kw.len(), space)
                } else {
                    self.p + kw.len()
                };
                self.emit(t, end);
                return;
            }
        }
        let tail = &self.src[self.p..];
        let mut matched = None;
        for (s, t, b) in [
            ("o|", T::ZERO_OR_ONE, false),
            ("o{", T::ZERO_OR_MORE, false),
            ("|o", T::ZERO_OR_ONE, true),
            ("||", T::ONLY_ONE, false),
            ("|{", T::ONE_OR_MORE, false),
            ("}o", T::ZERO_OR_MORE, true),
            ("}|", T::ONE_OR_MORE, false),
            ("..", T::NON_IDENTIFYING, false),
            (".-", T::NON_IDENTIFYING, false),
            ("--", T::IDENTIFYING, false),
            ("-.", T::NON_IDENTIFYING, false),
        ] {
            if tail
                .get(..s.len())
                .is_some_and(|v| v.eq_ignore_ascii_case(s))
                && (!b || !word(self.ch(self.p + s.len())))
            {
                matched = Some((t, s.len()));
                break;
            }
        }
        if matches!(c, 'u' | 'U') && matches!(self.ch(self.p + 1), Some('.' | '-' | '|')) {
            matched = Some((T::MD_PARENT, 1))
        }
        if let Some((t, n)) = matched {
            self.emit(t, self.p + n);
            return;
        }
        if text(c, false) {
            self.emit(T::UNICODE_TEXT, self.until(self.p, |c| text(c, false)))
        } else {
            self.emit(T::CHAR, self.p + c.len_utf8())
        }
    }
    fn block(&mut self, c: char) {
        if space(c) {
            self.p = self.until(self.p, space);
            return;
        }
        let tail = &self.src[self.p..];
        if ["pk", "fk", "uk"]
            .iter()
            .any(|k| tail.get(..2).is_some_and(|s| s.eq_ignore_ascii_case(k)))
            && !word(self.ch(self.p + 2))
        {
            self.emit(T::ATTRIBUTE_KEY, self.p + 2);
            return;
        }
        // Cache failed generic scans. A lone tilde after a long run of invalid
        // characters must not rescan that run for every character.
        if self.tilde_next.is_some_and(|q| q < self.p) {
            self.tilde_next = self.src[self.p..].find('~').map(|q| q + self.p);
        }
        if let Some(first) = self.tilde_next {
            if self.whitespace_next < self.p {
                self.whitespace_next = self.until(self.p, |c| !space(c));
            }
            if first < self.whitespace_next {
                if self.tilde_line < self.p {
                    self.tilde_line = self.src[self.p..]
                        .find(['\n', '\r', '\u{2028}', '\u{2029}'])
                        .map_or(self.src.len(), |q| q + self.p);
                    self.tilde_last = self.src[self.p..self.tilde_line]
                        .rfind('~')
                        .map(|q| q + self.p);
                }
                if let Some(last) = self.tilde_last.filter(|last| *last > first) {
                    let end = self.until(last + 1, |c| !space(c));
                    self.emit(T::ATTRIBUTE_WORD, end);
                    return;
                }
            }
        }
        if attr(c, true) {
            self.emit(
                T::ATTRIBUTE_WORD,
                self.until(self.p + c.len_utf8(), |c| attr(c, false)),
            );
            return;
        }
        if c == '`' {
            self.state = S::Backtick;
            self.p += 1;
            return;
        }
        if c == '"' {
            if let Some(i) = self.src[self.p + 1..].find('"') {
                self.emit(T::COMMENT, self.p + i + 2);
                return;
            }
        } else if c == '}' {
            self.state = S::Initial;
            self.emit(T::BLOCK_STOP, self.p + 1);
            return;
        }
        self.emit(T::CHAR, self.p + c.len_utf8())
    }
}
pub(super) fn tokenize<'a>(src: &'a str) -> Vec<Token<'a>> {
    let mut s = Scanner {
        src,
        p: 0,
        state: S::Initial,
        out: vec![],
        failed_dir: 0,
        tilde_next: src.find('~'),
        whitespace_next: 0,
        tilde_line: 0,
        tilde_last: None,
    };
    while s.p < src.len() {
        let c = s.ch(s.p).unwrap();
        match s.state {
            S::Initial => s.initial(c),
            S::Block => s.block(c),
            S::Backtick => {
                if c == '`' {
                    s.state = S::Block;
                    s.p += 1
                } else {
                    let e = src[s.p..].find('`').map_or(src.len(), |i| i + s.p);
                    s.emit(T::ATTRIBUTE_WORD, e)
                }
            }
            S::Title | S::Descr => {
                let t = if s.state == S::Title {
                    T::acc_title_value
                } else {
                    T::acc_descr_value
                };
                let e = src[s.p..].find('\n').map_or(src.len(), |i| i + s.p);
                s.emit(t, e);
                s.state = S::Initial
            }
            S::Multiline => {
                if c == '}' {
                    s.state = S::Initial;
                    s.p += 1
                } else {
                    let e = src[s.p..].find('}').map_or(src.len(), |i| i + s.p);
                    s.emit(T::acc_descr_multiline_value, e)
                }
            }
            S::Style => {
                if c == '\n' {
                    s.state = S::Initial;
                    s.emit(T::NEWLINE, s.until(s.p, |c| c == '\n'))
                } else if space(c) {
                    s.p = s.until(s.p, space)
                } else if let Some(t) = match c {
                    ':' => Some(T::COLON),
                    ',' => Some(T::COMMA),
                    '#' => Some(T::BRKT),
                    ';' => Some(T::SEMI),
                    _ => None,
                } {
                    s.emit(t, s.p + 1)
                } else if text(c, true) {
                    s.emit(T::STYLE_TEXT, s.until(s.p, |c| text(c, true)))
                } else {
                    s.out.push(Token {
                        kind: T::ERROR,
                        text: "",
                        start: s.p,
                    });
                    return s.out;
                }
            }
        }
    }
    let last = match s.state {
        S::Initial => Some(T::EOF),
        S::Title => Some(T::acc_title_value),
        S::Descr => Some(T::acc_descr_value),
        S::Multiline => Some(T::acc_descr_multiline_value),
        _ => None,
    };
    if let Some(t) = last {
        s.emit(t, src.len())
    }
    s.emit(T::END, src.len());
    s.out
}
