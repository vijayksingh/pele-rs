//! Shared native Gantt scheduling algorithm from Pele gantt/db.ts, 458322a5
//! (MIT, kepano). Storage owns task/Date identity; scheduling never copies a DB.
use super::{
    dates::{Calendar, WEEKDAYS},
    model::{error, patterns},
};
use crate::compat::{ErrorCode, PeleError};
#[derive(Clone, Copy)]
pub(super) enum Time {
    Start,
    End,
    RenderEnd,
}
#[derive(Clone, Copy)]
pub(super) enum Flag {
    Processed,
    ManualEnd,
}
/// Domain storage seam, deliberately limited to Gantt's scheduling state.
/// Public storage writes immediately, including writes preceding an error.
pub(super) trait Store {
    type Task: Clone;
    type TaskIterator;
    type Date: Clone;
    type Token: Clone;
    type Error: From<PeleError>;
    fn method(&mut self, _name: &str) -> Result<(), Self::Error> {
        Ok(())
    }
    fn now(&mut self) -> Result<f64, Self::Error>;
    fn set_now(&mut self, now: f64) -> Result<(), Self::Error>;
    fn work(&mut self) -> Result<f64, Self::Error>;
    fn add_work(&mut self, units: f64) -> Result<f64, Self::Error> {
        let work = self.work()? + units;
        self.set_work(work)?;
        Ok(work)
    }
    fn set_work(&mut self, work: f64) -> Result<(), Self::Error>;
    fn tokens(&mut self, include: bool) -> Result<Vec<Self::Token>, Self::Error>;
    fn token_is(&self, token: &Self::Token, text: &str) -> bool;
    fn token_length(&mut self, include: bool) -> Result<f64, Self::Error>;
    fn set_tokens(&mut self, include: bool, tokens: Vec<Self::Token>) -> Result<(), Self::Error>;
    fn contains(&mut self, include: bool, text: &str) -> Result<bool, Self::Error>;
    fn weekend_start(&mut self) -> Result<Option<usize>, Self::Error>;
    fn set_excluded_days(&mut self, days: Vec<bool>) -> Result<(), Self::Error>;
    fn excluded_day(&mut self, day: Option<usize>) -> Result<Option<bool>, Self::Error>;
    fn dated_tokens(&mut self) -> Result<bool, Self::Error>;
    fn set_dated_tokens(&mut self, value: bool) -> Result<(), Self::Error>;
    fn date_format(&mut self) -> Result<String, Self::Error>;
    fn inclusive(&mut self) -> Result<bool, Self::Error>;
    fn tasks(&mut self) -> Result<Vec<Self::Task>, Self::Error>;
    fn task_iterator(&mut self) -> Result<Self::TaskIterator, Self::Error>;
    fn next_task(
        &mut self,
        iterator: &mut Self::TaskIterator,
    ) -> Result<Option<Self::Task>, Self::Error>;
    fn previous(&mut self, task: &Self::Task) -> Result<Option<Self::Task>, Self::Error>;
    fn lookup(&mut self, id: &str) -> Result<Option<Self::Task>, Self::Error>;
    fn raw_start(&mut self, task: &Self::Task) -> Result<Option<String>, Self::Error>;
    fn raw_end(&mut self, task: &Self::Task) -> Result<String, Self::Error>;
    fn time(&mut self, task: &Self::Task, field: Time) -> Result<Option<Self::Date>, Self::Error>;
    fn set_time(
        &mut self,
        task: &Self::Task,
        field: Time,
        value: Option<Self::Date>,
    ) -> Result<(), Self::Error>;
    fn date(&mut self, milliseconds: f64) -> Result<Self::Date, Self::Error>;
    fn milliseconds(&mut self, date: &Self::Date) -> Result<f64, Self::Error>;
    fn date_truth(&self, date: &Self::Date) -> bool;
    fn manual(&mut self, task: &Self::Task) -> Result<bool, Self::Error>;
    fn processed(&mut self, task: &Self::Task) -> Result<bool, Self::Error>;
    fn set_flag(&mut self, task: &Self::Task, flag: Flag, value: bool) -> Result<(), Self::Error>;
    fn warn(&mut self, message: String) -> Result<(), Self::Error>;
}
fn trimmed(s: &str) -> &str {
    s.trim_matches(crate::compat::source::space)
}
pub(super) fn spend<S: Store>(s: &mut S, units: f64) -> Result<(), S::Error> {
    s.method("spend")?;
    let work = s.add_work(units)?;
    if work > 4e8 {
        return Err(error(
            "The gantt chart takes too much work to schedule.",
            ErrorCode::Limit,
        )
        .into());
    }
    Ok(())
}
pub(super) fn parse<S: Store>(
    s: &mut S,
    text: &str,
    template: &str,
    cal: &mut Calendar<'_>,
) -> Result<Option<S::Date>, S::Error> {
    s.method("parse")?;
    spend(
        s,
        (template.encode_utf16().count() + 1) as f64 * (text.encode_utf16().count() + 1) as f64,
    )?;
    let now = s.now()?;
    let parsed = cal.parse_strict(text, template, now);
    cal.check().map_err(S::Error::from)?;
    parsed.map(|ms| s.date(ms)).transpose()
}
pub(super) fn today<S: Store>(s: &mut S, cal: &mut Calendar<'_>) -> Result<S::Date, S::Error> {
    s.method("today")?;
    let now = s.now()?;
    let value = cal.today(now);
    cal.check().map_err(S::Error::from)?;
    s.date(value)
}
fn prepare_schedule<S: Store>(s: &mut S, now: f64) -> Result<(), S::Error> {
    s.method("getTasks")?;
    s.set_now(now)?;
    s.set_work(0.)?;
    let includes = s.tokens(true)?;
    s.set_tokens(true, includes)?;
    let excludes = s.tokens(false)?;
    s.set_tokens(false, excludes)?;
    let start = s.weekend_start()?;
    let weekends = s.contains(false, "weekends")?;
    let mut days = vec![];
    for (day, name) in WEEKDAYS.iter().enumerate() {
        let iso = if day == 0 { 7 } else { day };
        let weekend = weekends && start.is_some_and(|start| iso == start || iso == start + 1);
        days.push(weekend || s.contains(false, &name.to_lowercase())?);
    }
    s.set_excluded_days(days)?;
    let mut dated = s.token_length(true)? > 0.;
    if !dated {
        dated = !patterns()[0].is_match(trimmed(&s.date_format()?));
    }
    s.set_dated_tokens(dated)?;
    for token in s.tokens(false)? {
        if !s.token_is(&token, "weekends")
            && !WEEKDAYS
                .iter()
                .any(|d| s.token_is(&token, &d.to_lowercase()))
        {
            s.set_dated_tokens(true)?;
        }
    }
    for task in s.tasks()? {
        // Chained JS assignments execute right-hand writes first.
        s.set_time(&task, Time::End, None)?;
        s.set_time(&task, Time::Start, None)?;
        s.set_time(&task, Time::RenderEnd, None)?;
        s.set_flag(&task, Flag::ManualEnd, false)?;
        s.set_flag(&task, Flag::Processed, false)?;
    }
    Ok(())
}
pub(super) fn schedule<S: Store>(
    s: &mut S,
    now: f64,
    cal: &mut Calendar<'_>,
) -> Result<(), S::Error> {
    let mut program = Schedule::new(now);
    let mut reply = None;
    loop {
        match program.advance(s, reply.take())? {
            Step::Resource(op) => reply = Some(op.run(cal).map_err(S::Error::from)?),
            Step::Warning(w) => s.warn(w.message())?,
            Step::Complete(v) => return Ok(v),
        }
    }
}
pub(super) fn invalid<S: Store>(
    s: &mut S,
    date: &S::Date,
    cal: &mut Calendar<'_>,
) -> Result<Option<bool>, S::Error> {
    let mut program = tasks::Invalid::new(date.clone());
    let mut reply = None;
    loop {
        match program.advance(s, reply.take())? {
            Step::Resource(op) => reply = Some(op.run(cal).map_err(S::Error::from)?),
            Step::Warning(w) => s.warn(w.message())?,
            Step::Complete(v) => return Ok(v),
        }
    }
}
pub(super) struct ReferenceLookup<'a, T> {
    pub found: Vec<T>,
    pub unknown: Vec<&'a str>,
}
// Resolve once before warning evaluation. The returned task handles remain the
// same objects even when a warning getter or callback edits the lookup table.
pub(super) fn lookup_references<'a, S: Store>(
    s: &mut S,
    list: &'a str,
) -> Result<ReferenceLookup<'a, S::Task>, S::Error> {
    s.method("references")?;
    let mut found = vec![];
    let mut unknown = vec![];
    for id in list.split(' ').filter(|id| !id.is_empty()) {
        if let Some(task) = s.lookup(id)? {
            found.push(task)
        } else {
            unknown.push(id)
        }
    }
    Ok(ReferenceLookup { found, unknown })
}
pub(super) fn references<S: Store>(
    s: &mut S,
    keyword: &str,
    list: &str,
) -> Result<Vec<S::Task>, S::Error> {
    let ReferenceLookup { found, unknown } = lookup_references(s, list)?;
    if !unknown.is_empty() {
        s.warn(format!(
            "Gantt: \"{keyword}\" refers to task ids that do not exist: {}.",
            unknown.join(", ")
        ))?;
    }
    Ok(found)
}
mod end;
mod program;
mod tasks;
pub(super) use tasks::{Schedule, Task, Tasks};
mod start;
pub(super) use end::End;
pub(super) use program::{CalendarReply, Step, Warning};
pub(super) use start::Start;
pub(super) fn start<S: Store>(
    s: &mut S,
    raw: &str,
    cal: &mut Calendar<'_>,
) -> Result<Option<S::Date>, S::Error> {
    let mut program = Start::new(raw.to_owned());
    let mut reply = None;
    loop {
        match program.advance(s, reply.take())? {
            Step::Resource(operation) => {
                reply = Some(operation.run(cal).map_err(S::Error::from)?);
            }
            Step::Warning(warning) => s.warn(warning.message())?,
            Step::Complete(value) => return Ok(value),
        }
    }
}
pub(super) fn checked<S: Store>(
    s: &mut S,
    milliseconds: f64,
    text: &str,
) -> Result<S::Date, S::Error> {
    s.method("checked")?;
    if !super::dates::clip(milliseconds).is_finite() {
        return Err(error(format!("Invalid date:{text}"), ErrorCode::Semantic).into());
    }
    s.date(milliseconds)
}
pub(super) fn end<S: Store>(
    s: &mut S,
    start: &S::Date,
    raw: &str,
    cal: &mut Calendar<'_>,
) -> Result<Option<S::Date>, S::Error> {
    let mut program = End::new(start.clone(), raw.to_owned());
    let mut reply = None;
    loop {
        match program.advance(s, reply.take())? {
            Step::Resource(operation) => reply = Some(operation.run(cal).map_err(S::Error::from)?),
            Step::Warning(warning) => s.warn(warning.message())?,
            Step::Complete(value) => return Ok(value),
        }
    }
}
pub(super) fn compile_tasks<S: Store>(s: &mut S, cal: &mut Calendar<'_>) -> Result<bool, S::Error> {
    let mut program = Tasks::new();
    let mut reply = None;
    loop {
        match program.advance(s, reply.take())? {
            Step::Resource(op) => reply = Some(op.run(cal).map_err(S::Error::from)?),
            Step::Warning(w) => s.warn(w.message())?,
            Step::Complete(v) => return Ok(v),
        }
    }
}
pub(super) fn compile_task<S: Store>(
    s: &mut S,
    task: &S::Task,
    cal: &mut Calendar<'_>,
) -> Result<(), S::Error> {
    let mut program = Task::new(task.clone());
    let mut reply = None;
    loop {
        match program.advance(s, reply.take())? {
            Step::Resource(op) => reply = Some(op.run(cal).map_err(S::Error::from)?),
            Step::Warning(w) => s.warn(w.message())?,
            Step::Complete(v) => return Ok(v),
        }
    }
}
pub(super) fn check_dates<S: Store>(
    s: &mut S,
    task: &S::Task,
    cal: &mut Calendar<'_>,
) -> Result<(), S::Error> {
    let mut program = tasks::Check::new(task.clone());
    let mut reply = None;
    loop {
        match program.advance(s, reply.take())? {
            Step::Resource(op) => reply = Some(op.run(cal).map_err(S::Error::from)?),
            Step::Warning(w) => s.warn(w.message())?,
            Step::Complete(v) => return Ok(v),
        }
    }
}
