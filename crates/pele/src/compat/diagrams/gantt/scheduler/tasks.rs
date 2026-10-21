//! Owned scheduling continuations. Each resume reads/writes only the current
//! Store; Calendar and warning requests never retain a borrowed Store.
use super::super::model::error;
use super::program::{CalendarReply, CalendarRequest, Step};
use super::{prepare_schedule, spend, trimmed, End, Flag, Start, Store, Time};
use crate::compat::ErrorCode;

fn parse_request<S: Store>(
    s: &mut S,
    text: String,
    template: String,
) -> Result<CalendarRequest, S::Error> {
    s.method("parse")?;
    spend(
        s,
        (template.encode_utf16().count() + 1) as f64 * (text.encode_utf16().count() + 1) as f64,
    )?;
    Ok(CalendarRequest::Parse {
        text,
        template,
        now: s.now()?,
    })
}
enum InvalidPhase<D> {
    Begin(D),
    Weekday(f64),
    Iso {
        date: f64,
        weekday: Option<bool>,
        template: String,
    },
    Format {
        weekday: Option<bool>,
        date_only: String,
    },
    Done,
}
pub(super) struct Invalid<D> {
    phase: InvalidPhase<D>,
}
impl<D: Clone> Invalid<D> {
    pub(super) fn new(date: D) -> Self {
        Self {
            phase: InvalidPhase::Begin(date),
        }
    }
    pub(super) fn advance<S: Store<Date = D>>(
        &mut self,
        s: &mut S,
        mut reply: Option<CalendarReply>,
    ) -> Result<Step<Option<bool>>, S::Error> {
        loop {
            match std::mem::replace(&mut self.phase, InvalidPhase::Done) {
                InvalidPhase::Begin(date) => {
                    s.method("isInvalidDate")?;
                    let ms = s.milliseconds(&date)?;
                    self.phase = InvalidPhase::Weekday(ms);
                    return Ok(Step::Resource(CalendarRequest::Weekday(ms)));
                }
                InvalidPhase::Weekday(ms) => {
                    let day = reply
                        .take()
                        .expect("weekday reply")
                        .number()
                        .map(|v| v as usize);
                    let weekday = s.excluded_day(day)?;
                    if !s.dated_tokens()? {
                        return Ok(Step::Complete(weekday));
                    }
                    let template = s.date_format()?;
                    let template = trimmed(&template).to_owned();
                    self.phase = InvalidPhase::Iso {
                        date: ms,
                        weekday,
                        template,
                    };
                    if ms.is_finite() {
                        return Ok(Step::Resource(CalendarRequest::Iso(ms)));
                    }
                    reply = Some(CalendarReply::Text(Some("0NaN-NaN-NaN".into())));
                }
                InvalidPhase::Iso {
                    date,
                    weekday,
                    template,
                } => {
                    let date_only = reply.take().expect("iso reply").text().expect("iso text");
                    spend(s, 200.)?;
                    if template != "YYYY-MM-DD" {
                        spend(s, 400. + template.encode_utf16().count() as f64 * 8.)?;
                        self.phase = InvalidPhase::Format { weekday, date_only };
                        return Ok(Step::Resource(CalendarRequest::Format { date, template }));
                    }
                    self.phase = InvalidPhase::Format {
                        weekday,
                        date_only: date_only.clone(),
                    };
                    reply = Some(CalendarReply::Text(Some(date_only)));
                }
                InvalidPhase::Format { weekday, date_only } => {
                    let formatted = reply.take().expect("format reply").text();
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
                        return Ok(Step::Complete(Some(false)));
                    }
                    if weekday == Some(true) {
                        return Ok(Step::Complete(Some(true)));
                    }
                    return Ok(Step::Complete(Some(listed(false)?)));
                }
                InvalidPhase::Done => {
                    return Err(error(
                        "Gantt invalid-date operation already finished",
                        ErrorCode::Semantic,
                    )
                    .into())
                }
            }
        }
    }
}
struct Scan {
    cursor: f64,
    end: f64,
    max: f64,
    bad: bool,
    render: Option<f64>,
}
enum CheckPhase<D> {
    Begin,
    Cursor,
    Max { cursor: f64, end: f64 },
    Loop(Scan),
    Invalid { scan: Scan, program: Invalid<D> },
    Extended(Scan),
    Next(Scan),
    Done,
}
pub(super) struct Check<T, D> {
    task: T,
    phase: CheckPhase<D>,
}
impl<T: Clone, D: Clone> Check<T, D> {
    pub(super) fn new(task: T) -> Self {
        Self {
            task,
            phase: CheckPhase::Begin,
        }
    }
    pub(super) fn advance<S: Store<Task = T, Date = D>>(
        &mut self,
        s: &mut S,
        mut reply: Option<CalendarReply>,
    ) -> Result<Step<()>, S::Error> {
        loop {
            match std::mem::replace(&mut self.phase, CheckPhase::Done) {
                CheckPhase::Begin => {
                    s.method("checkTaskDates")?;
                    if s.token_length(false)? == 0. || s.manual(&self.task)? {
                        return Ok(Step::Complete(()));
                    }
                    let start = s.time(&self.task, Time::Start)?;
                    let ms = match start {
                        Some(start) => s.milliseconds(&start)?,
                        None => f64::NAN,
                    };
                    self.phase = CheckPhase::Cursor;
                    return Ok(Step::Resource(CalendarRequest::AddDays {
                        date: ms,
                        amount: 1.,
                    }));
                }
                CheckPhase::Cursor => {
                    let cursor = reply
                        .take()
                        .expect("cursor reply")
                        .number()
                        .expect("cursor");
                    let end = s.time(&self.task, Time::End)?;
                    let end = match end {
                        Some(end) => s.milliseconds(&end)?,
                        None => f64::NAN,
                    };
                    self.phase = CheckPhase::Max { cursor, end };
                    return Ok(Step::Resource(CalendarRequest::AddDays {
                        date: end,
                        amount: 10000.,
                    }));
                }
                CheckPhase::Max { cursor, end } => {
                    let max = reply.take().expect("max reply").number().expect("max");
                    self.phase = CheckPhase::Loop(Scan {
                        cursor,
                        end,
                        max,
                        bad: false,
                        render: None,
                    });
                }
                CheckPhase::Loop(mut scan) => {
                    if !(scan.cursor <= scan.end) {
                        let end = s.date(scan.end)?;
                        s.set_time(&self.task, Time::End, Some(end))?;
                        let render = scan.render.map(|ms| s.date(ms)).transpose()?;
                        s.set_time(&self.task, Time::RenderEnd, render)?;
                        return Ok(Step::Complete(()));
                    }
                    spend(s, 400.)?;
                    if !scan.bad {
                        scan.render = Some(scan.end);
                    }
                    let date = s.date(scan.cursor)?;
                    self.phase = CheckPhase::Invalid {
                        scan,
                        program: Invalid::new(date),
                    };
                }
                CheckPhase::Invalid {
                    mut scan,
                    mut program,
                } => match program.advance(s, reply.take())? {
                    Step::Complete(value) => {
                        scan.bad = value == Some(true);
                        if scan.bad {
                            let date = scan.end;
                            self.phase = CheckPhase::Extended(scan);
                            return Ok(Step::Resource(CalendarRequest::AddDays {
                                date,
                                amount: 1.,
                            }));
                        }
                        let date = scan.cursor;
                        self.phase = CheckPhase::Next(scan);
                        return Ok(Step::Resource(CalendarRequest::AddDays {
                            date,
                            amount: 1.,
                        }));
                    }
                    Step::Resource(op) => {
                        self.phase = CheckPhase::Invalid { scan, program };
                        return Ok(Step::Resource(op));
                    }
                    Step::Warning(w) => {
                        self.phase = CheckPhase::Invalid { scan, program };
                        return Ok(Step::Warning(w));
                    }
                },
                CheckPhase::Extended(mut scan) => {
                    scan.end = reply
                        .take()
                        .expect("extension reply")
                        .number()
                        .expect("end");
                    if scan.end > scan.max {
                        return Err(error("Failed to find a valid date that was not excluded by `excludes` after 10,000 iterations.",ErrorCode::Semantic).into());
                    }
                    let date = scan.cursor;
                    self.phase = CheckPhase::Next(scan);
                    return Ok(Step::Resource(CalendarRequest::AddDays {
                        date,
                        amount: 1.,
                    }));
                }
                CheckPhase::Next(mut scan) => {
                    scan.cursor = reply.take().expect("next reply").number().expect("cursor");
                    self.phase = CheckPhase::Loop(scan);
                }
                CheckPhase::Done => {
                    return Err(error(
                        "Gantt date-check operation already finished",
                        ErrorCode::Semantic,
                    )
                    .into())
                }
            }
        }
    }
}
enum TaskPhase<T, D> {
    Begin,
    Start(Start<T>),
    Today,
    AfterStart,
    End(End<T, D>),
    Manual,
    Check(Check<T, D>),
    Done,
}
pub(in crate::compat::diagrams::gantt) struct Task<T, D> {
    task: T,
    phase: TaskPhase<T, D>,
}
impl<T: Clone, D: Clone> Task<T, D> {
    pub(in crate::compat::diagrams::gantt) fn new(task: T) -> Self {
        Self {
            task,
            phase: TaskPhase::Begin,
        }
    }
    pub(in crate::compat::diagrams::gantt) fn advance<S: Store<Task = T, Date = D>>(
        &mut self,
        s: &mut S,
        mut reply: Option<CalendarReply>,
    ) -> Result<Step<()>, S::Error> {
        loop {
            match std::mem::replace(&mut self.phase, TaskPhase::Done) {
                TaskPhase::Begin => {
                    s.method("compileTask")?;
                    if let Some(raw) = s.raw_start(&self.task)? {
                        self.phase = TaskPhase::Start(Start::new(raw));
                    } else if let Some(previous) = s.previous(&self.task)? {
                        let date = s.time(&previous, Time::End)?;
                        s.set_time(&self.task, Time::Start, date)?;
                        self.phase = TaskPhase::AfterStart;
                    } else {
                        s.method("today")?;
                        let now = s.now()?;
                        self.phase = TaskPhase::Today;
                        return Ok(Step::Resource(CalendarRequest::Today(now)));
                    }
                }
                TaskPhase::Start(mut program) => match program.advance(s, reply.take())? {
                    Step::Complete(date) => {
                        if date.is_some() {
                            s.set_time(&self.task, Time::Start, date)?;
                        }
                        self.phase = TaskPhase::AfterStart;
                    }
                    Step::Resource(op) => {
                        self.phase = TaskPhase::Start(program);
                        return Ok(Step::Resource(op));
                    }
                    Step::Warning(w) => {
                        self.phase = TaskPhase::Start(program);
                        return Ok(Step::Warning(w));
                    }
                },
                TaskPhase::Today => {
                    let date = reply
                        .take()
                        .expect("today reply")
                        .number()
                        .map(|ms| s.date(ms))
                        .transpose()?;
                    s.set_time(&self.task, Time::Start, date)?;
                    self.phase = TaskPhase::AfterStart;
                }
                TaskPhase::AfterStart => {
                    let Some(start) = s.time(&self.task, Time::Start)?.filter(|d| s.date_truth(d))
                    else {
                        return Ok(Step::Complete(()));
                    };
                    let raw = s.raw_end(&self.task)?;
                    self.phase = TaskPhase::End(End::new(start, raw));
                }
                TaskPhase::End(mut program) => match program.advance(s, reply.take())? {
                    Step::Complete(date) => {
                        let truth = date.as_ref().is_some_and(|d| s.date_truth(d));
                        s.set_time(&self.task, Time::End, date)?;
                        if !truth {
                            return Ok(Step::Complete(()));
                        }
                        s.set_flag(&self.task, Flag::Processed, true)?;
                        let raw = s.raw_end(&self.task)?;
                        let op = parse_request(s, raw, "YYYY-MM-DD".into())?;
                        self.phase = TaskPhase::Manual;
                        return Ok(Step::Resource(op));
                    }
                    Step::Resource(op) => {
                        self.phase = TaskPhase::End(program);
                        return Ok(Step::Resource(op));
                    }
                    Step::Warning(w) => {
                        self.phase = TaskPhase::End(program);
                        return Ok(Step::Warning(w));
                    }
                },
                TaskPhase::Manual => {
                    let manual = reply.take().expect("manual reply").number().is_some();
                    s.set_flag(&self.task, Flag::ManualEnd, manual)?;
                    self.phase = TaskPhase::Check(Check::new(self.task.clone()));
                }
                TaskPhase::Check(mut program) => match program.advance(s, reply.take())? {
                    Step::Complete(()) => return Ok(Step::Complete(())),
                    Step::Resource(op) => {
                        self.phase = TaskPhase::Check(program);
                        return Ok(Step::Resource(op));
                    }
                    Step::Warning(w) => {
                        self.phase = TaskPhase::Check(program);
                        return Ok(Step::Warning(w));
                    }
                },
                TaskPhase::Done => {
                    return Err(
                        error("Gantt task operation already finished", ErrorCode::Semantic).into(),
                    )
                }
            }
        }
    }
}
pub(in crate::compat::diagrams::gantt) struct Tasks<T, D, I> {
    iterator: Option<I>,
    all: bool,
    current: Option<Task<T, D>>,
    visits: usize,
}
impl<T: Clone, D: Clone, I> Tasks<T, D, I> {
    pub(in crate::compat::diagrams::gantt) fn new() -> Self {
        Self {
            iterator: None,
            all: true,
            current: None,
            visits: 0,
        }
    }
    pub(in crate::compat::diagrams::gantt) fn advance<
        S: Store<Task = T, Date = D, TaskIterator = I>,
    >(
        &mut self,
        s: &mut S,
        mut reply: Option<CalendarReply>,
    ) -> Result<Step<bool>, S::Error> {
        if self.iterator.is_none() {
            s.method("compileTasks")?;
            self.iterator = Some(s.task_iterator()?);
        }
        loop {
            if let Some(mut task) = self.current.take() {
                match task.advance(s, reply.take())? {
                    Step::Complete(()) => {
                        self.all = self.all && s.processed(&task.task)?;
                    }
                    Step::Resource(op) => {
                        self.current = Some(task);
                        return Ok(Step::Resource(op));
                    }
                    Step::Warning(w) => {
                        self.current = Some(task);
                        return Ok(Step::Warning(w));
                    }
                }
            }
            let Some(task) = s.next_task(self.iterator.as_mut().expect("tasks initialized"))?
            else {
                return Ok(Step::Complete(self.all));
            };
            self.visits += 1;
            if self.visits > 8_000_000 {
                return Err(
                    error("Gantt task iteration exceeds work budget", ErrorCode::Limit).into(),
                );
            }
            self.current = Some(Task::new(task));
        }
    }
}
pub(in crate::compat::diagrams::gantt) struct Schedule<T, D, I> {
    now: Option<f64>,
    iteration: usize,
    tasks: Tasks<T, D, I>,
}
impl<T: Clone, D: Clone, I> Schedule<T, D, I> {
    pub(in crate::compat::diagrams::gantt) fn new(now: f64) -> Self {
        Self {
            now: Some(now),
            iteration: 0,
            tasks: Tasks::new(),
        }
    }
    pub(in crate::compat::diagrams::gantt) fn advance<
        S: Store<Task = T, Date = D, TaskIterator = I>,
    >(
        &mut self,
        s: &mut S,
        mut reply: Option<CalendarReply>,
    ) -> Result<Step<()>, S::Error> {
        if let Some(now) = self.now.take() {
            prepare_schedule(s, now)?;
        }
        loop {
            match self.tasks.advance(s, reply.take())? {
                Step::Complete(all) => {
                    if all || self.iteration == 10 {
                        return Ok(Step::Complete(()));
                    }
                    self.iteration += 1;
                    self.tasks = Tasks::new();
                }
                Step::Resource(op) => return Ok(Step::Resource(op)),
                Step::Warning(w) => return Ok(Step::Warning(w)),
            }
        }
    }
}
