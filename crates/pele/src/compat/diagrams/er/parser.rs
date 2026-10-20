//! Direct Pele er/parser.ts grammar and database calls, 458322a5 (MIT, kepano).
use super::{
    lexer::{tokenize, Token},
    model::*,
    tokens::{NAMES, T},
};
use crate::compat::{source::space, ParseContext, PeleError};
const NAME: &str = "'UNICODE_TEXT', 'NUM', 'ENTITY_NAME', 'DECIMAL_NUM', 'ENTITY_ONE'";
const CARDS: &str = "'ZERO_OR_ONE', 'ZERO_OR_MORE', 'ONE_OR_MORE', 'ONLY_ONE', 'MD_PARENT'";
const LINE:&str="'EOF', 'NEWLINE', 'title', 'acc_title', 'acc_descr', 'acc_descr_multiline_value', 'SUBGRAPH', 'direction_tb', 'direction_bt', 'direction_rl', 'direction_lr', 'CLASSDEF', 'CLASS', 'STYLE', 'UNICODE_TEXT', 'NUM', 'ENTITY_NAME', 'DECIMAL_NUM', 'ENTITY_ONE'";
fn card(t: T) -> Option<Cardinality> {
    Some(match t {
        T::ZERO_OR_ONE => Cardinality::ZERO_OR_ONE,
        T::ZERO_OR_MORE => Cardinality::ZERO_OR_MORE,
        T::ONE_OR_MORE => Cardinality::ONE_OR_MORE,
        T::ONLY_ONE => Cardinality::ONLY_ONE,
        T::MD_PARENT => Cardinality::MD_PARENT,
        _ => return None,
    })
}
fn is_name(t: T) -> bool {
    matches!(
        t,
        T::UNICODE_TEXT | T::ENTITY_NAME | T::NUM | T::DECIMAL_NUM | T::ENTITY_ONE
    )
}
fn style(t: T) -> bool {
    matches!(t, T::STYLE_TEXT | T::NUM | T::COLON | T::BRKT)
}
struct Frame {
    id: String,
    title: String,
    list: Vec<DocItem>,
}
// One grammar, two private construction targets. Public/source typed parsing
// keeps the original ErModel actions; direct publication may request fallback.
pub(super) trait Target {
    fn header(&mut self) -> &mut ErModel;
    fn entity(
        &mut self,
        name: &str,
        alias: Option<&str>,
        attributes: Vec<ErAttribute>,
        order: Vec<bool>,
    ) -> Result<(), PeleError>;
    fn relationship(
        &mut self,
        a: &str,
        role: String,
        b: &str,
        spec: RelSpec,
    ) -> Result<(), PeleError>;
    fn subgraph(
        &mut self,
        id: String,
        title: String,
        list: Vec<DocItem>,
    ) -> Result<String, PeleError>;
    fn add_class(&mut self, ids: &[String], styles: &[String]) -> Result<(), PeleError>;
    fn styles(&mut self, ids: &[String], styles: &[String]) -> Result<(), PeleError>;
    fn set_class(&mut self, ids: &[String], names: &[String]) -> Result<(), PeleError>;
}
impl Target for ErModel {
    fn header(&mut self) -> &mut ErModel {
        self
    }
    fn entity(
        &mut self,
        name: &str,
        alias: Option<&str>,
        attributes: Vec<ErAttribute>,
        order: Vec<bool>,
    ) -> Result<(), PeleError> {
        if let Some(alias) = alias {
            self.add_entity(name, alias);
        }
        self.append_parsed_attributes(name, attributes, order);
        Ok(())
    }
    fn relationship(
        &mut self,
        a: &str,
        role: String,
        b: &str,
        spec: RelSpec,
    ) -> Result<(), PeleError> {
        self.add_entity(a, "");
        self.add_entity(b, "");
        ErModel::relationship(self, a, role, b, spec);
        Ok(())
    }
    fn subgraph(
        &mut self,
        id: String,
        title: String,
        list: Vec<DocItem>,
    ) -> Result<String, PeleError> {
        Ok(ErModel::subgraph(self, id, title, list))
    }
    fn add_class(&mut self, ids: &[String], styles: &[String]) -> Result<(), PeleError> {
        ErModel::add_class(self, ids, styles);
        Ok(())
    }
    fn styles(&mut self, ids: &[String], styles: &[String]) -> Result<(), PeleError> {
        ErModel::styles(self, ids, styles);
        Ok(())
    }
    fn set_class(&mut self, ids: &[String], names: &[String]) -> Result<(), PeleError> {
        ErModel::set_class(self, ids, names);
        Ok(())
    }
}
struct Reader<'a, 'b, D> {
    src: &'a str,
    tokens: Vec<Token<'a>>,
    i: usize,
    db: &'b mut D,
}
impl<D: Target> Reader<'_, '_, D> {
    fn t(&self) -> T {
        self.tokens.get(self.i).map_or(T::END, |t| t.kind)
    }
    fn take(&mut self) -> String {
        let s = self.tokens[self.i].text.to_owned();
        self.i += 1;
        s
    }
    fn fail(&self, expected: &str) -> PeleError {
        let t = &self.tokens[self.i.min(self.tokens.len() - 1)];
        if t.kind == T::ERROR {
            PeleError::syntax("er", self.src, t.start, "", true)
        } else {
            PeleError::syntax(
                "er",
                self.src,
                t.start,
                &format!(
                    "Expecting {expected}, got '{}'",
                    if t.kind == T::CHAR {
                        t.text
                    } else {
                        NAMES[t.kind as usize]
                    }
                ),
                false,
            )
        }
    }
    fn expect(&mut self, t: T) -> Result<String, PeleError> {
        if self.t() != t {
            return Err(self.fail(&format!("'{}'", NAMES[t as usize])));
        }
        Ok(self.take())
    }
    fn separator(&mut self) -> Result<(), PeleError> {
        if !matches!(self.t(), T::NEWLINE | T::SEMI | T::EOF) {
            return Err(self.fail("'EOF', 'NEWLINE', 'SEMI'"));
        }
        self.i += 1;
        Ok(())
    }
    fn name(&mut self) -> Result<String, PeleError> {
        let t = self.t();
        if !is_name(t) {
            return Err(self.fail(NAME));
        }
        let s = self.take();
        Ok(if t == T::ENTITY_NAME {
            s[1..s.len() - 1].into()
        } else {
            s
        })
    }
    fn ids(&mut self) -> Result<Vec<String>, PeleError> {
        let mut v = vec![];
        loop {
            if !matches!(self.t(), T::UNICODE_TEXT | T::STYLE_TEXT) {
                return Err(self.fail("'UNICODE_TEXT', 'STYLE_TEXT'"));
            }
            v.push(self.take());
            if self.t() != T::COMMA {
                return Ok(v);
            }
            self.i += 1
        }
    }
    fn styles(&mut self) -> Result<Vec<String>, PeleError> {
        let mut v = vec![];
        loop {
            if !style(self.t()) {
                return Err(self.fail("'STYLE_TEXT', 'COLON', 'NUM', 'BRKT'"));
            }
            let mut s = self.take();
            while style(self.t()) {
                s.push_str(&self.take())
            }
            v.push(s);
            if self.t() != T::COMMA {
                return Ok(v);
            }
            self.i += 1
        }
    }
    fn rel(&mut self) -> Result<RelSpec, PeleError> {
        let card_b = card(self.t()).unwrap();
        self.i += 1;
        let rel_type = match self.t() {
            T::IDENTIFYING => Identification::IDENTIFYING,
            T::NON_IDENTIFYING => Identification::NON_IDENTIFYING,
            _ => return Err(self.fail("'NON_IDENTIFYING', 'IDENTIFYING'")),
        };
        self.i += 1;
        let Some(card_a) = card(self.t()) else {
            return Err(self.fail(CARDS));
        };
        self.i += 1;
        Ok(RelSpec {
            card_a,
            rel_type,
            card_b,
        })
    }
    fn role(&mut self) -> Result<String, PeleError> {
        let t = self.t();
        if !matches!(t, T::WORD | T::ENTITY_NAME | T::UNICODE_TEXT) {
            return Err(self.fail("'UNICODE_TEXT', 'ENTITY_NAME', 'WORD'"));
        }
        let s = self.take();
        Ok(if t == T::UNICODE_TEXT {
            s
        } else {
            s[1..s.len() - 1].into()
        })
    }
    fn char(&self, c: &str) -> bool {
        self.t() == T::CHAR && self.tokens[self.i].text == c
    }
    fn block(&mut self) -> Result<(Vec<ErAttribute>, Vec<bool>), PeleError> {
        if self.t() != T::BLOCK_START {
            return Ok((vec![], vec![]));
        }
        self.i += 1;
        if self.t() == T::BLOCK_STOP {
            self.i += 1;
            return Ok((vec![], vec![]));
        }
        let mut attrs = vec![];
        let mut order = vec![];
        loop {
            let mut ty = self.expect(T::ATTRIBUTE_WORD)?;
            if self.char("?") {
                ty.push_str(&self.take())
            }
            if self.t() != T::ATTRIBUTE_WORD {
                return Err(self.fail(if ty.ends_with('?') {
                    "'ATTRIBUTE_WORD'"
                } else {
                    "'ATTRIBUTE_WORD', '?'"
                }));
            }
            let name = self.take();
            let mut keys = vec![];
            if self.t() == T::ATTRIBUTE_KEY {
                keys.push(self.take());
                while self.char(",") {
                    self.i += 1;
                    keys.push(self.expect(T::ATTRIBUTE_KEY)?);
                }
            }
            let comment_first = keys.is_empty() && self.t() == T::COMMENT;
            let comment = if self.t() == T::COMMENT {
                let s = self.take();
                s[1..s.len() - 1].into()
            } else {
                String::new()
            };
            order.push(comment_first);
            attrs.push(ErAttribute {
                r#type: ty,
                name,
                keys,
                comment,
            });
            if self.t() == T::BLOCK_STOP {
                break;
            }
        }
        self.i += 1;
        Ok((attrs, order))
    }
    fn entity(&mut self) -> Result<Vec<String>, PeleError> {
        let a = self.name()?;
        if self.t() == T::SQS {
            self.i += 1;
            let alias = self.name()?;
            self.expect(T::SQE)?;
            let classes = if self.t() == T::STYLE_SEPARATOR {
                self.i += 1;
                self.ids()?
            } else {
                vec![]
            };
            let (attrs, order) = self.block()?;
            self.db.entity(&a, Some(&alias), attrs, order)?;
            self.db.set_class(std::slice::from_ref(&a), &classes)?;
            return Ok(vec![a]);
        }
        let classes = if self.t() == T::STYLE_SEPARATOR {
            self.i += 1;
            self.ids()?
        } else {
            vec![]
        };
        if card(self.t()).is_some() {
            let spec = self.rel()?;
            let b = self.name()?;
            let classes_b = if self.t() == T::STYLE_SEPARATOR {
                self.i += 1;
                self.ids()?
            } else {
                vec![]
            };
            self.expect(T::COLON)?;
            let label = self.role()?;
            self.db.relationship(&a, label, &b, spec)?;
            self.db.set_class(std::slice::from_ref(&a), &classes)?;
            self.db.set_class(std::slice::from_ref(&b), &classes_b)?;
            Ok(vec![a, b])
        } else {
            let (attrs, order) = self.block()?;
            self.db.entity(&a, None, attrs, order)?;
            self.db.set_class(std::slice::from_ref(&a), &classes)?;
            Ok(vec![a])
        }
    }
    fn parse(mut self) -> Result<(), PeleError> {
        self.expect(T::ER_DIAGRAM)?;
        let mut frames = vec![Frame {
            id: String::new(),
            title: String::new(),
            list: vec![],
        }];
        loop {
            let nested = frames.len() > 1;
            match self.t() {
                T::NEWLINE => self.i += 1,
                T::EOF => {
                    self.i += 1;
                    if nested {
                        return Err(self.fail(&format!("'END', {LINE}")));
                    }
                    return Ok(());
                }
                T::END_KW => {
                    if !nested {
                        return Err(self.fail(LINE));
                    }
                    self.i += 1;
                    let f = frames.pop().unwrap();
                    let id = self.db.subgraph(f.id, f.title, f.list)?;
                    if frames.len() > 1 {
                        frames.last_mut().unwrap().list.push(DocItem::Text(id))
                    }
                }
                T::SUBGRAPH => {
                    self.i += 1;
                    let id = self.name()?;
                    let mut title = id.clone();
                    if self.t() == T::SQS {
                        self.i += 1;
                        title = self.name()?;
                        while is_name(self.t()) {
                            title.push(' ');
                            title.push_str(&self.name()?)
                        }
                        self.expect(T::SQE)?;
                    }
                    self.separator()?;
                    frames.push(Frame {
                        id,
                        title,
                        list: vec![],
                    })
                }
                T::acc_title | T::acc_descr | T::acc_descr_multiline_value => {
                    let t = self.t();
                    let value = if t == T::acc_descr_multiline_value {
                        self.take()
                    } else {
                        self.i += 1;
                        self.expect(if t == T::acc_title {
                            T::acc_title_value
                        } else {
                            T::acc_descr_value
                        })?
                    };
                    let value = value.trim_matches(space).to_owned();
                    if t == T::acc_title {
                        self.db.header().acc_title = Some(value.clone())
                    } else {
                        let mut normalized = String::new();
                        let mut chars = value.chars().peekable();
                        while let Some(c) = chars.next() {
                            normalized.push(c);
                            if c == '\n' {
                                while chars.peek().is_some_and(|c| space(*c)) {
                                    chars.next();
                                }
                            }
                        }
                        self.db.header().acc_descr = Some(normalized)
                    }
                    if nested {
                        frames.last_mut().unwrap().list.push(DocItem::Text(value))
                    }
                }
                T::direction_tb | T::direction_bt | T::direction_rl | T::direction_lr => {
                    let d = match self.t() {
                        T::direction_bt => "BT",
                        T::direction_rl => "RL",
                        T::direction_lr => "LR",
                        _ => "TB",
                    }
                    .to_owned();
                    self.i += 1;
                    if nested {
                        frames.last_mut().unwrap().list.push(DocItem::Dir(d))
                    } else {
                        self.db.header().direction = d
                    }
                }
                T::CLASSDEF | T::STYLE => {
                    let t = self.t();
                    let keyword = self.take();
                    let ids = self.ids()?;
                    let styles = self.styles()?;
                    self.separator()?;
                    if t == T::CLASSDEF {
                        self.db.add_class(&ids, &styles)?
                    } else {
                        self.db.styles(&ids, &styles)?
                    }
                    if nested {
                        frames.last_mut().unwrap().list.push(DocItem::Text(keyword))
                    }
                }
                T::CLASS => {
                    let keyword = self.take();
                    let ids = self.ids()?;
                    let names = self.ids()?;
                    self.db.set_class(&ids, &names)?;
                    if nested {
                        frames.last_mut().unwrap().list.push(DocItem::Text(keyword))
                    }
                }
                t => {
                    if !is_name(t) {
                        return Err(self.fail(&if nested {
                            format!("'END', {LINE}")
                        } else {
                            LINE.into()
                        }));
                    }
                    let names = self.entity()?;
                    if nested {
                        frames
                            .last_mut()
                            .unwrap()
                            .list
                            .extend(names.into_iter().map(DocItem::Text))
                    }
                }
            }
        }
    }
}
pub(super) fn parse(context: ParseContext<'_>) -> Result<ErModel, PeleError> {
    let mut db = ErModel::new(context.title);
    parse_into(context.source, &mut db)?;
    Ok(db)
}
pub(super) fn parse_into(source: &str, db: &mut impl Target) -> Result<(), PeleError> {
    Reader {
        src: source,
        tokens: tokenize(source),
        i: 0,
        db,
    }
    .parse()
}
