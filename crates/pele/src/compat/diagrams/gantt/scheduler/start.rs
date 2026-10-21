//! One getStartDate algorithm, driven synchronously or by owned public phases.
//! The program retains task identity across warnings, not a copied task value.
use super::super::model::{error, patterns};
use super::program::{CalendarReply, CalendarRequest, Step, Warning};
use super::{checked, lookup_references, spend, trimmed, Store, Time};
use crate::compat::ErrorCode;

enum Phase<T> {
    Begin(String),
    AfterReferences(Vec<T>),
    Parsed(String),
    Date,
    Done,
}
pub(in crate::compat::diagrams::gantt) struct Start<T> {
    phase: Phase<T>,
}
impl<T: Clone> Start<T> {
    pub(in crate::compat::diagrams::gantt) fn new(raw: String) -> Self {
        Self {
            phase: Phase::Begin(raw),
        }
    }
    pub(in crate::compat::diagrams::gantt) fn advance<S: Store<Task = T>>(
        &mut self,
        s: &mut S,
        mut reply: Option<CalendarReply>,
    ) -> Result<Step<Option<S::Date>>, S::Error> {
        loop {
            match std::mem::replace(&mut self.phase, Phase::Done) {
                Phase::Begin(raw) => {
                    s.method("getStartDate")?;
                    let text = trimmed(&raw);
                    let template = s.date_format()?;
                    let template = trimmed(&template);
                    if matches!(template, "x" | "X") && patterns()[6].is_match(text) {
                        let ms = super::super::dates::clip(text.parse().unwrap_or(f64::NAN));
                        return checked(s, ms, text).map(|v| Step::Complete(Some(v)));
                    }
                    if let Some(c) = patterns()[2].captures(text) {
                        let lookup = lookup_references(s, &c[1])?;
                        self.phase = Phase::AfterReferences(lookup.found);
                        if !lookup.unknown.is_empty() {
                            return Ok(Step::Warning(Warning::References {
                                keyword: "after",
                                missing: lookup.unknown.into_iter().map(str::to_owned).collect(),
                            }));
                        }
                        continue;
                    }
                    s.method("parse")?;
                    spend(
                        s,
                        (template.encode_utf16().count() + 1) as f64
                            * (text.encode_utf16().count() + 1) as f64,
                    )?;
                    let now = s.now()?;
                    self.phase = Phase::Parsed(text.to_owned());
                    return Ok(Step::Resource(CalendarRequest::Parse {
                        text: text.to_owned(),
                        template: template.to_owned(),
                        now,
                    }));
                }
                Phase::AfterReferences(found) => {
                    let mut latest = None;
                    for task in found {
                        let later = if let Some(prior) = &latest {
                            let current = s.time(&task, Time::End)?;
                            let before = s.time(prior, Time::End)?;
                            match (current, before) {
                                (Some(a), Some(b)) => s.milliseconds(&a)? > s.milliseconds(&b)?,
                                _ => false,
                            }
                        } else {
                            true
                        };
                        if later {
                            latest = Some(task);
                        }
                    }
                    if let Some(task) = latest {
                        return s.time(&task, Time::End).map(Step::Complete);
                    }
                    s.method("today")?;
                    let now = s.now()?;
                    self.phase = Phase::Date;
                    return Ok(Step::Resource(CalendarRequest::Today(now)));
                }
                Phase::Parsed(text) => {
                    if let Some(date) = reply.take().expect("native calendar reply").number() {
                        return s.date(date).map(|v| Step::Complete(Some(v)));
                    }
                    self.phase = Phase::Date;
                    return Ok(Step::Resource(CalendarRequest::Loose(text)));
                }
                Phase::Date => {
                    let value = reply.take().expect("native calendar reply").number();
                    return value.map(|ms| s.date(ms)).transpose().map(Step::Complete);
                }
                Phase::Done => {
                    return Err(error(
                        "Gantt start-date operation already finished",
                        ErrorCode::Semantic,
                    )
                    .into())
                }
            }
        }
    }
}
