//! getEndDate uses the same owned program for direct and suspended scheduling.
use super::super::model::{error, patterns};
use super::program::{CalendarReply, CalendarRequest, Step, Warning};
use super::{lookup_references, spend, trimmed, Store, Time};
use crate::compat::ErrorCode;
enum Phase<T, D> {
    Begin {
        start: D,
        raw: String,
    },
    References(Vec<T>),
    Parsed {
        start: D,
        text: String,
        template: String,
    },
    Date,
    Added(f64),
    Warned(f64),
    Done,
}
pub(in crate::compat::diagrams::gantt) struct End<T, D> {
    phase: Phase<T, D>,
}
impl<T: Clone, D: Clone> End<T, D> {
    pub(in crate::compat::diagrams::gantt) fn new(start: D, raw: String) -> Self {
        Self {
            phase: Phase::Begin { start, raw },
        }
    }
    pub(in crate::compat::diagrams::gantt) fn advance<S: Store<Task = T, Date = D>>(
        &mut self,
        s: &mut S,
        mut reply: Option<CalendarReply>,
    ) -> Result<Step<Option<D>>, S::Error> {
        loop {
            match std::mem::replace(&mut self.phase, Phase::Done) {
                Phase::Begin { start, raw } => {
                    s.method("getEndDate")?;
                    let text = trimmed(&raw);
                    if let Some(c) = patterns()[3].captures(text) {
                        let lookup = lookup_references(s, &c[1])?;
                        self.phase = Phase::References(lookup.found);
                        if !lookup.unknown.is_empty() {
                            return Ok(Step::Warning(Warning::References {
                                keyword: "until",
                                missing: lookup.unknown.into_iter().map(str::to_owned).collect(),
                            }));
                        }
                        continue;
                    }
                    let template = s.date_format()?;
                    let template = trimmed(&template);
                    s.method("parse")?;
                    spend(
                        s,
                        (template.encode_utf16().count() + 1) as f64
                            * (text.encode_utf16().count() + 1) as f64,
                    )?;
                    let now = s.now()?;
                    let request = CalendarRequest::Parse {
                        text: text.to_owned(),
                        template: template.to_owned(),
                        now,
                    };
                    self.phase = Phase::Parsed {
                        start,
                        text: text.to_owned(),
                        template: template.to_owned(),
                    };
                    return Ok(Step::Resource(request));
                }
                Phase::References(found) => {
                    let mut earliest = None;
                    for task in found {
                        let earlier = if let Some(prior) = &earliest {
                            let current = s.time(&task, Time::Start)?;
                            let before = s.time(prior, Time::Start)?;
                            match (current, before) {
                                (Some(a), Some(b)) => s.milliseconds(&a)? < s.milliseconds(&b)?,
                                _ => false,
                            }
                        } else {
                            true
                        };
                        if earlier {
                            earliest = Some(task);
                        }
                    }
                    if let Some(task) = earliest {
                        return s.time(&task, Time::Start).map(Step::Complete);
                    }
                    s.method("today")?;
                    let now = s.now()?;
                    self.phase = Phase::Date;
                    return Ok(Step::Resource(CalendarRequest::Today(now)));
                }
                Phase::Parsed {
                    start,
                    text,
                    template,
                } => {
                    if let Some(ms) = reply.take().expect("calendar reply").number() {
                        let parsed = s.date(ms)?;
                        if s.inclusive()? {
                            let date = s.milliseconds(&parsed)?;
                            self.phase = Phase::Date;
                            return Ok(Step::Resource(CalendarRequest::AddDays {
                                date,
                                amount: 1.,
                            }));
                        }
                        return Ok(Step::Complete(Some(parsed)));
                    }
                    let milliseconds = s.milliseconds(&start)?;
                    let Some(c) = patterns()[4].captures(&text) else {
                        self.phase = Phase::Warned(milliseconds);
                        return Ok(Step::Warning(Warning::Duration { text, template }));
                    };
                    let amount = c[1].parse().unwrap_or(f64::NAN);
                    let unit = c[2].to_owned();
                    self.phase = Phase::Added(milliseconds);
                    return Ok(Step::Resource(CalendarRequest::Add {
                        date: milliseconds,
                        amount,
                        unit,
                    }));
                }
                Phase::Added(original) => {
                    let end = reply
                        .take()
                        .expect("calendar reply")
                        .number()
                        .expect("add result");
                    return s
                        .date(if end.is_finite() { end } else { original })
                        .map(|v| Step::Complete(Some(v)));
                }
                Phase::Warned(milliseconds) => {
                    return s.date(milliseconds).map(|v| Step::Complete(Some(v)))
                }
                Phase::Date => {
                    return reply
                        .take()
                        .expect("calendar reply")
                        .number()
                        .map(|ms| s.date(ms))
                        .transpose()
                        .map(Step::Complete)
                }
                Phase::Done => {
                    return Err(error(
                        "Gantt end-date operation already finished",
                        ErrorCode::Semantic,
                    )
                    .into())
                }
            }
        }
    }
}
