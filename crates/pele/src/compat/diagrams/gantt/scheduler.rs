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
pub(super) fn schedule<S: Store>(
    s: &mut S,
    now: f64,
    cal: &mut Calendar<'_>,
) -> Result<(), S::Error> {
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
    let mut all = compile_tasks(s, cal)?;
    for _ in 0..10 {
        if all {
            break;
        }
        all = compile_tasks(s, cal)?;
    }
    Ok(())
}
pub(super) fn invalid<S: Store>(
    s: &mut S,
    date: &S::Date,
    cal: &mut Calendar<'_>,
) -> Result<Option<bool>, S::Error> {
    s.method("isInvalidDate")?;
    let milliseconds = s.milliseconds(date)?;
    let day = cal.fields(milliseconds).map(|f| f.weekday as usize);
    cal.check().map_err(S::Error::from)?;
    let weekday = s.excluded_day(day)?;
    if !s.dated_tokens()? {
        return Ok(weekday);
    }
    let template = s.date_format()?;
    let template = trimmed(&template);
    let date_only = if milliseconds.is_finite() {
        cal.iso(milliseconds)
    } else {
        "0NaN-NaN-NaN".into()
    };
    cal.check().map_err(S::Error::from)?;
    spend(s, 200.)?;
    let formatted = if template != "YYYY-MM-DD" {
        spend(s, 400. + template.encode_utf16().count() as f64 * 8.)?;
        let formatted = cal.format(milliseconds, template);
        cal.check().map_err(S::Error::from)?;
        formatted
    } else {
        Some(date_only.clone())
    };
    let mut listed = |include| -> Result<bool, S::Error> {
        if s.contains(include, &date_only)? {
            return Ok(true);
        }
        if let Some(f) = &formatted {
            s.contains(include, f)
        } else {
            Ok(false)
        }
    };
    if listed(true)? {
        return Ok(Some(false));
    }
    if weekday == Some(true) {
        return Ok(Some(true));
    }
    Ok(Some(listed(false)?))
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
pub(super) fn start<S: Store>(
    s: &mut S,
    raw: &str,
    cal: &mut Calendar<'_>,
) -> Result<Option<S::Date>, S::Error> {
    s.method("getStartDate")?;
    let text = trimmed(raw);
    let template = s.date_format()?;
    let template = trimmed(&template);
    if matches!(template, "x" | "X") && patterns()[6].is_match(text) {
        let ms = super::dates::clip(text.parse().unwrap_or(f64::NAN));
        return checked(s, ms, text).map(Some);
    }
    if let Some(c) = patterns()[2].captures(text) {
        let mut latest: Option<S::Task> = None;
        for task in references(s, "after", &c[1])? {
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
                latest = Some(task)
            }
        }
        return if let Some(task) = latest {
            s.time(&task, Time::End)
        } else {
            today(s, cal).map(Some)
        };
    }
    if let Some(date) = parse(s, text, template, cal)? {
        return Ok(Some(date));
    }
    let date = cal
        .resources
        .parse_date(text)
        .map_err(S::Error::from)?
        .unwrap_or(f64::NAN);
    let fields = if date.is_finite() {
        cal.fields(date)
    } else {
        None
    };
    cal.check().map_err(S::Error::from)?;
    if !date.is_finite() || fields.is_none_or(|f| !(-10000..=10000).contains(&f.civil.year)) {
        return Err(error(format!("Invalid date:{text}"), ErrorCode::Semantic).into());
    }
    s.date(date).map(Some)
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
    s.method("getEndDate")?;
    let text = trimmed(raw);
    if let Some(c) = patterns()[3].captures(text) {
        let mut earliest: Option<S::Task> = None;
        for task in references(s, "until", &c[1])? {
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
                earliest = Some(task)
            }
        }
        return if let Some(task) = earliest {
            s.time(&task, Time::Start)
        } else {
            today(s, cal).map(Some)
        };
    }
    let template = s.date_format()?;
    let template = trimmed(&template);
    if let Some(parsed) = parse(s, text, template, cal)? {
        return if s.inclusive()? {
            let ms = s.milliseconds(&parsed)?;
            let end = cal.add_days(ms, 1.);
            cal.check().map_err(S::Error::from)?;
            s.date(end).map(Some)
        } else {
            Ok(Some(parsed))
        };
    }
    let milliseconds = s.milliseconds(start)?;
    let Some(c) = patterns()[4].captures(text) else {
        s.warn(format!("Gantt: \"{text}\" is neither a date in the \"{template}\" format nor a duration such as \"3d\", so the task gets no duration."))?;
        return s.date(milliseconds).map(Some);
    };
    let value = c[1].parse().unwrap_or(f64::NAN);
    let end = cal.add(milliseconds, value, &c[2]);
    cal.check().map_err(S::Error::from)?;
    s.date(if end.is_finite() { end } else { milliseconds })
        .map(Some)
}
pub(super) fn compile_tasks<S: Store>(s: &mut S, cal: &mut Calendar<'_>) -> Result<bool, S::Error> {
    s.method("compileTasks")?;
    let mut all = true;
    for task in s.tasks()? {
        compile_task(s, &task, cal)?;
        all = all && s.processed(&task)?;
    }
    Ok(all)
}
pub(super) fn compile_task<S: Store>(
    s: &mut S,
    task: &S::Task,
    cal: &mut Calendar<'_>,
) -> Result<(), S::Error> {
    s.method("compileTask")?;
    if let Some(raw) = s.raw_start(task)? {
        if let Some(date) = start(s, &raw, cal)? {
            s.set_time(task, Time::Start, Some(date))?;
        }
    } else {
        let date = if let Some(previous) = s.previous(task)? {
            s.time(&previous, Time::End)?
        } else {
            Some(today(s, cal)?)
        };
        s.set_time(task, Time::Start, date)?;
    }
    let Some(start) = s.time(task, Time::Start)?.filter(|d| s.date_truth(d)) else {
        return Ok(());
    };
    let raw = s.raw_end(task)?;
    let date = end(s, &start, &raw, cal)?;
    let truth = date.as_ref().is_some_and(|d| s.date_truth(d));
    s.set_time(task, Time::End, date)?;
    if !truth {
        return Ok(());
    }
    s.set_flag(task, Flag::Processed, true)?;
    let raw = s.raw_end(task)?;
    let manual = parse(s, &raw, "YYYY-MM-DD", cal)?.is_some();
    s.set_flag(task, Flag::ManualEnd, manual)?;
    check_dates(s, task, cal)
}
pub(super) fn check_dates<S: Store>(
    s: &mut S,
    task: &S::Task,
    cal: &mut Calendar<'_>,
) -> Result<(), S::Error> {
    s.method("checkTaskDates")?;
    // This accessor reads the actual manualEndTime without changing processed.
    if s.token_length(false)? == 0. || s.manual(task)? {
        return Ok(());
    }
    let start = s.time(task, Time::Start)?;
    let milliseconds = if let Some(start) = start {
        s.milliseconds(&start)?
    } else {
        f64::NAN
    };
    let mut cursor = cal.add_days(milliseconds, 1.);
    cal.check().map_err(S::Error::from)?;
    let end = s.time(task, Time::End)?;
    let mut end = if let Some(end) = end {
        s.milliseconds(&end)?
    } else {
        f64::NAN
    };
    let max = cal.add_days(end, 10000.);
    cal.check().map_err(S::Error::from)?;
    let mut bad = false;
    let mut render = None;
    while cursor <= end {
        spend(s, 400.)?;
        if !bad {
            render = Some(end)
        }
        let date = s.date(cursor)?;
        bad = invalid(s, &date, cal)? == Some(true);
        if bad {
            end = cal.add_days(end, 1.);
            cal.check().map_err(S::Error::from)?;
            if end > max {
                return Err(error("Failed to find a valid date that was not excluded by `excludes` after 10,000 iterations.",ErrorCode::Semantic).into());
            }
        }
        cursor = cal.add_days(cursor, 1.);
        cal.check().map_err(S::Error::from)?;
    }
    let end = s.date(end)?;
    s.set_time(task, Time::End, Some(end))?;
    let render = render.map(|ms| s.date(ms)).transpose()?;
    s.set_time(task, Time::RenderEnd, render)
}
