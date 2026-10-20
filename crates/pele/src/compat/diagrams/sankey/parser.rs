//! Pele sankey/{lexer,parser}.ts RFC4180 ASCII scanner, 458322a5 (MIT, kepano).
use super::indexed::{IndexedSankey, SourceSymbols};
use super::model::*;
use crate::compat::{source::space, ParseContext, PeleError};
use std::{borrow::Cow, collections::HashMap};
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum T {
    END,
    STUCK,
    SANKEY,
    NEWLINE,
    COMMA,
    DQUOTE,
    NON_ESCAPED_TEXT,
    ESCAPED_TEXT,
    EOF,
}
const NAMES: [&str; 9] = [
    "$end",
    "STUCK",
    "SANKEY",
    "NEWLINE",
    "COMMA",
    "DQUOTE",
    "NON_ESCAPED_TEXT",
    "ESCAPED_TEXT",
    "EOF",
];
struct Token {
    kind: T,
    start: usize,
    end: usize,
}
fn data(c: u8) -> bool {
    (32..=126).contains(&c) && c != 34 && c != 44
}
fn keyword(src: &str, p: usize, w: &str) -> bool {
    src.get(p..p + w.len())
        .is_some_and(|s| s.eq_ignore_ascii_case(w))
        && !src
            .as_bytes()
            .get(p + w.len())
            .is_some_and(|c| c.is_ascii_alphanumeric() || *c == b'_')
}
fn tokenize(src: &str) -> Vec<Token> {
    let b = src.as_bytes();
    let n = b.len();
    let (mut p, mut state) = (0, 0);
    let mut stack = vec![];
    let mut out = vec![];
    while p < n {
        let start = p;
        let c = b[p];
        let kind;
        if state == 2 {
            if c == 34 && b.get(p + 1) != Some(&34) {
                state = stack.pop().unwrap_or(state);
                p += 1;
                kind = T::DQUOTE;
            } else {
                while p < n {
                    let d = b[p];
                    if data(d) || matches!(d, 44 | 13 | 10) {
                        p += 1
                    } else if d == 34 && b.get(p + 1) == Some(&34) {
                        p += 2
                    } else {
                        break;
                    }
                }
                kind = T::ESCAPED_TEXT;
            }
        } else {
            let len = if state == 0 && (c | 32) == 115 {
                if keyword(src, p, "sankey-beta") {
                    11
                } else if keyword(src, p, "sankey") {
                    6
                } else {
                    0
                }
            } else {
                0
            };
            if len > 0 {
                stack.push(state);
                state = 1;
                p += len;
                kind = T::SANKEY;
            } else if c == 10 || (c == 13 && b.get(p + 1) == Some(&10)) {
                p += if c == 10 { 1 } else { 2 };
                kind = T::NEWLINE;
            } else if c == 44 {
                p += 1;
                kind = T::COMMA;
            } else if c == 34 {
                stack.push(state);
                state = 2;
                p += 1;
                kind = T::DQUOTE;
            } else {
                while p < n && data(b[p]) {
                    p += 1
                }
                kind = T::NON_ESCAPED_TEXT;
            }
        }
        out.push(Token {
            kind,
            start,
            end: p,
        });
        if p == start {
            out.push(Token {
                kind: T::STUCK,
                start,
                end: p,
            });
            return out;
        }
    }
    out.push(Token {
        kind: if state == 2 { T::ESCAPED_TEXT } else { T::EOF },
        start: n,
        end: n,
    });
    out.push(Token {
        kind: T::END,
        start: n,
        end: n,
    });
    out
}
fn prepare(src: &str) -> String {
    let src = src.trim_matches(|c| c != '\n' && c != '\r' && space(c));
    let mut out = String::with_capacity(src.len());
    let mut br = false;
    for c in src.chars() {
        if c == '\r' || c == '\n' {
            if !br {
                out.push('\n')
            }
            br = true
        } else {
            out.push(c);
            br = false
        }
    }
    out.trim_matches(space).into()
}
// JS parseFloat consumes the longest decimal prefix, including signed Infinity.
fn amount(s: &str) -> f64 {
    let s = s.trim_matches(space);
    let b = s.as_bytes();
    let mut p = usize::from(matches!(b.first(), Some(b'+') | Some(b'-')));
    if s.get(p..).is_some_and(|s| s.starts_with("Infinity")) {
        return if b.first() == Some(&b'-') {
            -f64::INFINITY
        } else {
            f64::INFINITY
        };
    }
    let start = p;
    while b.get(p).is_some_and(u8::is_ascii_digit) {
        p += 1
    }
    let mut digits = p - start;
    if b.get(p) == Some(&b'.') {
        p += 1;
        let q = p;
        while b.get(p).is_some_and(u8::is_ascii_digit) {
            p += 1
        }
        digits += p - q;
    }
    if digits == 0 {
        return f64::NAN;
    }
    let end = p;
    if matches!(b.get(p), Some(b'e') | Some(b'E')) {
        p += 1;
        if matches!(b.get(p), Some(b'+') | Some(b'-')) {
            p += 1
        }
        let q = p;
        while b.get(p).is_some_and(u8::is_ascii_digit) {
            p += 1
        }
        if p == q {
            p = end
        }
    }
    s[..p].parse().unwrap_or(f64::NAN)
}
struct Reader<'a> {
    src: &'a str,
    tokens: Vec<Token>,
    i: usize,
}
impl<'a> Reader<'a> {
    fn t(&self) -> T {
        self.tokens[self.i].kind
    }
    fn fail(&self, expected: &str) -> PeleError {
        let t = &self.tokens[self.i];
        let got = if t.kind == T::STUCK {
            self.tokens[self.i - 1].kind
        } else {
            t.kind
        };
        PeleError::syntax(
            "sankey",
            self.src,
            t.start,
            &format!("Expecting {expected}, got '{}'", NAMES[got as usize]),
            false,
        )
    }
    fn expect(&mut self, t: T, also: &str) -> Result<(), PeleError> {
        if self.t() != t {
            return Err(self.fail(&format!("'{}'{also}", NAMES[t as usize])));
        }
        self.i += 1;
        Ok(())
    }
    fn take(&mut self) -> &'a str {
        let t = &self.tokens[self.i];
        self.i += 1;
        &self.src[t.start..t.end]
    }
    fn field(&mut self) -> Result<&'a str, PeleError> {
        if self.t() == T::NON_ESCAPED_TEXT {
            return Ok(self.take());
        }
        if self.t() != T::DQUOTE {
            return Err(self.fail("'DQUOTE', 'NON_ESCAPED_TEXT'"));
        }
        self.i += 1;
        if self.t() != T::ESCAPED_TEXT {
            return Err(self.fail("'ESCAPED_TEXT'"));
        }
        let text = self.take();
        self.expect(T::DQUOTE, "")?;
        Ok(text)
    }
    fn name(&mut self) -> Result<Cow<'a, str>, PeleError> {
        let text = self.field()?.trim_matches(space);
        Ok(if text.contains("\"\"") {
            Cow::Owned(text.replace("\"\"", "\""))
        } else {
            Cow::Borrowed(text)
        })
    }
}
trait Target: Sized {
    type Output;
    fn new(title: Option<&str>) -> Self;
    fn node(&mut self, id: Cow<'_, str>) -> usize;
    fn link(&mut self, source: usize, target: usize, value: f64);
    fn finish(self) -> Self::Output;
}
struct LegacyTarget {
    model: SankeyModel,
    lookup: HashMap<String, usize>,
}
impl Target for LegacyTarget {
    type Output = SankeyModel;
    fn new(title: Option<&str>) -> Self {
        Self {
            model: SankeyModel::new(title),
            lookup: HashMap::new(),
        }
    }
    fn node(&mut self, id: Cow<'_, str>) -> usize {
        if let Some(index) = self.lookup.get(id.as_ref()) {
            return *index;
        }
        let index = self.lookup.len();
        let id = id.into_owned();
        self.model.nodes.push(SankeyNode {
            id: id.clone(),
            index,
        });
        self.lookup.insert(id, index);
        index
    }
    fn link(&mut self, source: usize, target: usize, value: f64) {
        self.model.links.push(SankeyLink {
            source: self.model.nodes[source].clone(),
            target: self.model.nodes[target].clone(),
            value,
        });
    }
    fn finish(self) -> Self::Output {
        self.model
    }
}
impl Target for SourceSymbols {
    type Output = IndexedSankey;
    fn new(title: Option<&str>) -> Self {
        SourceSymbols::new(title)
    }
    fn node(&mut self, id: Cow<'_, str>) -> usize {
        SourceSymbols::node(self, id)
    }
    fn link(&mut self, source: usize, target: usize, value: f64) {
        SourceSymbols::link(self, source, target, value);
    }
    fn finish(self) -> Self::Output {
        SourceSymbols::finish(self)
    }
}
pub(super) fn parse(c: ParseContext<'_>) -> Result<SankeyModel, PeleError> {
    parse_into::<LegacyTarget>(c)
}
pub(super) fn parse_indexed(c: ParseContext<'_>) -> Result<IndexedSankey, PeleError> {
    parse_into::<SourceSymbols>(c)
}
fn parse_into<M: Target>(c: ParseContext<'_>) -> Result<M::Output, PeleError> {
    let src = prepare(c.source);
    let mut r = Reader {
        tokens: tokenize(&src),
        src: &src,
        i: 0,
    };
    let mut db = M::new(c.title);
    r.expect(T::SANKEY, "")?;
    r.expect(T::NEWLINE, "")?;
    loop {
        let from = r.name()?;
        r.expect(T::COMMA, ", 'NEWLINE', 'EOF'")?;
        let to = r.name()?;
        r.expect(T::COMMA, ", 'NEWLINE', 'EOF'")?;
        let value = amount(r.field()?);
        let mut ids = [0; 2];
        for (k, id) in [from, to].into_iter().enumerate() {
            ids[k] = db.node(id);
        }
        db.link(ids[0], ids[1], value);
        if r.t() != T::NEWLINE {
            break;
        }
        r.i += 1;
    }
    if r.t() == T::EOF {
        r.i += 1
    }
    if r.t() != T::END {
        return Err(r.fail("'NEWLINE', 'EOF'"));
    }
    Ok(db.finish())
}
