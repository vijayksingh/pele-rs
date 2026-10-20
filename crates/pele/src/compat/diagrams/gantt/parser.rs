//! Pele gantt/parser.ts, 458322a5 (MIT, kepano).
use super::{
    lexer::{tokenize, Token},
    model::GanttModel,
    tokens::{NAMES, T},
};
use crate::compat::{source::space, PeleError};
const LINE:&str="'EOF', 'SPACE', 'NL', 'weekday_monday', 'weekday_tuesday', 'weekday_wednesday', 'weekday_thursday', 'weekday_friday', 'weekday_saturday', 'weekday_sunday', 'weekend_friday', 'weekend_saturday', 'dateFormat', 'inclusiveEndDates', 'topAxis', 'axisFormat', 'tickInterval', 'excludes', 'includes', 'todayMarker', 'title', 'acc_title', 'acc_descr', 'acc_descr_multiline_value', 'section', 'taskTxt', 'click'";
fn starts(t: T) -> bool {
    matches!(
        t,
        T::EOF
            | T::NL
            | T::dateFormat
            | T::inclusiveEndDates
            | T::topAxis
            | T::axisFormat
            | T::tickInterval
            | T::excludes
            | T::includes
            | T::todayMarker
            | T::title
            | T::acc_title
            | T::acc_descr
            | T::acc_descr_multiline_value
            | T::section
            | T::taskTxt
            | T::click
            | T::weekday_monday
            | T::weekday_tuesday
            | T::weekday_wednesday
            | T::weekday_thursday
            | T::weekday_friday
            | T::weekday_saturday
            | T::weekday_sunday
            | T::weekend_friday
            | T::weekend_saturday
    )
}
pub(super) trait Target {
    fn header(&mut self) -> &mut GanttModel;
    fn add_task(&mut self, text: String, data: String) -> Result<(), PeleError>;
    fn class(&mut self, ids: &str) -> Result<(), PeleError>;
    fn link(&mut self, ids: &str, url: &str) -> Result<(), PeleError>;
}
impl Target for GanttModel {
    fn header(&mut self) -> &mut GanttModel {
        self
    }
    fn add_task(&mut self, text: String, data: String) -> Result<(), PeleError> {
        GanttModel::add_task(self, text, data)
    }
    fn class(&mut self, ids: &str) -> Result<(), PeleError> {
        GanttModel::class(self, ids);
        Ok(())
    }
    fn link(&mut self, ids: &str, url: &str) -> Result<(), PeleError> {
        GanttModel::link(self, ids, url)
    }
}
pub fn parse(source: &str, db: &mut GanttModel) -> Result<(), PeleError> {
    parse_into(source, db)
}
struct Reader<'a> {
    source: &'a str,
    tokens: Vec<Token>,
    i: usize,
}
impl Reader<'_> {
    fn t(&self) -> T {
        self.tokens.get(self.i).map_or(T::END, |t| t.kind)
    }
    fn take(&mut self) -> String {
        let s = self.tokens[self.i].text.clone();
        self.i += 1;
        s
    }
    fn fail(&self, expected: &str) -> PeleError {
        let t = self
            .tokens
            .get(self.i)
            .unwrap_or_else(|| self.tokens.last().unwrap());
        PeleError::syntax(
            "gantt",
            self.source,
            t.start,
            &format!(
                "Expecting {expected}, got '{}'",
                if t.kind == T::END {
                    "1"
                } else {
                    NAMES[t.kind as usize]
                }
            ),
            false,
        )
    }
    fn end(&self, also: &str) -> Result<(), PeleError> {
        if !starts(self.t()) {
            Err(self.fail(&format!("{LINE}{also}")))
        } else {
            Ok(())
        }
    }
    fn expect(&mut self, t: T) -> Result<String, PeleError> {
        if self.t() != t {
            return Err(self.fail(&format!("'{}'", NAMES[t as usize])));
        }
        Ok(self.take())
    }
}
pub(super) fn parse_into(source: &str, db: &mut impl Target) -> Result<(), PeleError> {
    let mut r = Reader {
        source,
        tokens: tokenize(source),
        i: 0,
    };
    r.expect(T::gantt)?;
    loop {
        let t = r.t();
        if !starts(t) {
            return Err(r.fail(LINE));
        }
        let text = r.take();
        match t {
            T::EOF => {
                if r.t() == T::END {
                    return Ok(());
                }
                r.end("")?
            }
            T::NL => {}
            T::acc_title | T::acc_descr => {
                let value = r.expect(if t == T::acc_title {
                    T::acc_title_value
                } else {
                    T::acc_descr_value
                })?;
                r.end("")?;
                if t == T::acc_title {
                    db.header().acc_title = Some(value.trim_matches(space).into())
                } else {
                    db.header().acc_descr = Some(description(value.trim_matches(space)))
                }
            }
            T::acc_descr_multiline_value => {
                r.end("")?;
                db.header().acc_descr = Some(description(text.trim_matches(space)))
            }
            T::taskTxt => {
                let data = r.expect(T::taskData)?;
                r.end("")?;
                db.add_task(text, data)?
            }
            T::click => {
                let mut name = false;
                let mut args = false;
                let mut href = None;
                if r.t() == T::callbackname {
                    name = true;
                    r.i += 1;
                    if r.t() == T::callbackargs {
                        args = true;
                        r.i += 1
                    }
                    if r.t() == T::href {
                        href = Some(r.take());
                        r.end("")?
                    } else {
                        r.end(if args {
                            ", 'href'"
                        } else {
                            ", 'callbackargs', 'href'"
                        })?
                    }
                } else if r.t() == T::href {
                    href = Some(r.take());
                    if r.t() == T::callbackname {
                        name = true;
                        r.i += 1;
                        if r.t() == T::callbackargs {
                            r.i += 1;
                            r.end("")?
                        } else {
                            r.end(", 'callbackargs'")?
                        }
                    } else {
                        r.end(", 'callbackname'")?
                    }
                } else {
                    return Err(r.fail("'callbackname', 'href'"));
                }
                if name {
                    db.class(&text)?
                }
                if let Some(href) = href {
                    db.link(&text, &href)?
                }
            }
            _ => {
                r.end("")?;
                match t {
                    T::dateFormat => db.header().date_format = cut(&text, 11),
                    T::inclusiveEndDates => db.header().inclusive_end_dates = true,
                    T::topAxis => db.header().top_axis = true,
                    T::axisFormat => db.header().axis_format = cut(&text, 11),
                    T::tickInterval => db.header().tick_interval = Some(cut(&text, 13)),
                    T::excludes => GanttModel::tokens(&mut db.header().excludes, &cut(&text, 9)),
                    T::includes => GanttModel::tokens(&mut db.header().includes, &cut(&text, 9)),
                    T::todayMarker => db.header().today_marker = cut(&text, 12),
                    T::weekend_friday => db.header().weekend = "friday".into(),
                    T::weekend_saturday => db.header().weekend = "saturday".into(),
                    T::title => db.header().title = Some(cut(&text, 6)),
                    T::section => db.header().add_section(cut(&text, 8)),
                    _ => {
                        db.header().weekday = Some(
                            [
                                "monday",
                                "tuesday",
                                "wednesday",
                                "thursday",
                                "friday",
                                "saturday",
                                "sunday",
                            ][t as usize - T::weekday_monday as usize]
                                .into(),
                        )
                    }
                }
            }
        }
    }
}
fn cut(s: &str, n: usize) -> String {
    s.chars().skip(n).collect()
}
fn description(s: &str) -> String {
    let mut out = String::new();
    let mut skip = false;
    for c in s.chars() {
        if skip && space(c) {
            continue;
        }
        skip = c == '\n';
        out.push(c)
    }
    out
}
