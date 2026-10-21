//! Pele gantt/db.ts scheduling and ordered typed model, 458322a5 (MIT, kepano).
use super::dates::{utc_fields, Calendar};
use crate::compat::{
    source::{space, url::safe_url},
    ErrorCode, PeleError, RenderResources,
};
use regex::Regex;
use serde::{ser::SerializeMap, Serialize};
use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};
fn date_json<S: serde::Serializer>(date: &Option<f64>, s: S) -> Result<S::Ok, S::Error> {
    match date.and_then(utc_fields) {
        None => s.serialize_none(),
        Some(f) => {
            let c = f.civil;
            let year = if (0..=9999).contains(&c.year) {
                format!("{:04}", c.year)
            } else {
                format!("{}{:06}", if c.year < 0 { '-' } else { '+' }, c.year.abs())
            };
            s.serialize_str(&format!(
                "{year}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
                c.month, c.day, c.hour, c.minute, c.second, c.millisecond
            ))
        }
    }
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GanttTask {
    pub id: String,
    pub task: String,
    pub section: String,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "date_json")]
    pub start_time: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "date_json")]
    pub end_time: Option<f64>,
    #[serde(serialize_with = "date_json")]
    pub render_end_time: Option<f64>,
    pub manual_end_time: bool,
    pub processed: bool,
    pub active: bool,
    pub done: bool,
    pub crit: bool,
    pub milestone: bool,
    pub vert: bool,
    pub order: isize,
    pub classes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_task_id: Option<String>,
    pub raw: TaskRaw,
}
#[derive(Clone, Debug, Serialize)]
pub struct TaskRaw {
    pub data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    pub end: String,
}
#[derive(Clone, Debug)]
pub struct GanttModel {
    task_at: HashMap<String, usize>,
    public_task_orders: HashMap<usize, f64>,
    public_task_identities: HashMap<usize, u32>,
    by_at: HashMap<String, usize>,
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    pub date_format: String,
    pub axis_format: String,
    pub tick_interval: Option<String>,
    pub today_marker: String,
    pub includes: Vec<String>,
    pub excludes: Vec<String>,
    pub inclusive_end_dates: bool,
    pub top_axis: bool,
    pub display_mode: String,
    pub weekday: Option<String>,
    pub weekend: String,
    pub sections: Vec<String>,
    pub links: Vec<(String, String)>,
    pub tasks: Vec<GanttTask>,
    pub(crate) by_id: Vec<(String, usize)>,
    current_section: String,
    task_count: usize,
    last_order: usize,
    last_task_id: Option<String>,
    pub(crate) now: f64,
    pub(crate) work: f64,
    include_set: HashSet<String>,
    exclude_set: HashSet<String>,
    excluded_days: Vec<bool>,
    dated_tokens: bool,
}
// Shared task declaration semantics; storage targets choose ownership afterward.
pub(super) struct TaskDeclaration<'a> {
    pub id: Option<&'a str>,
    pub start: Option<&'a str>,
    pub end: &'a str,
    pub active: bool,
    pub done: bool,
    pub crit: bool,
    pub milestone: bool,
    pub vert: bool,
}
pub(super) fn task_declaration<'a>(
    descr: &str,
    data: &'a str,
) -> Result<TaskDeclaration<'a>, PeleError> {
    let mut fields: Vec<&str> = data.strip_prefix(':').unwrap_or(data).split(',').collect();
    let mut active = false;
    let mut done = false;
    let mut crit = false;
    let mut milestone = false;
    let mut vert = false;
    while let Some(tag) = fields.first().and_then(|s| patterns()[1].captures(s)) {
        match &tag[1] {
            "active" => active = true,
            "done" => done = true,
            "crit" => crit = true,
            "milestone" => milestone = true,
            "vert" => vert = true,
            _ => {}
        }
        fields.remove(0);
    }
    if fields.is_empty() {
        return Err(error(
            format!(
                "Task \"{}\" needs a duration or an end date.",
                descr.trim_matches(space)
            ),
            ErrorCode::Semantic,
        ));
    }
    if fields.len() > 3 {
        return Err(error(format!("Task \"{}\" has too many fields. A task takes an id, a start, and an end or duration.", descr.trim_matches(space)), ErrorCode::Semantic));
    }
    let fields: Vec<_> = fields.iter().map(|s| s.trim_matches(space)).collect();
    Ok(TaskDeclaration {
        id: (fields.len() == 3).then(|| fields[0]),
        start: (fields.len() > 1).then(|| fields[fields.len() - 2]),
        end: fields[fields.len() - 1],
        active,
        done,
        crit,
        milestone,
        vert,
    })
}
impl Serialize for GanttModel {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut m = s.serialize_map(None)?;
        m.serialize_entry("type", "gantt")?;
        for (key, value) in [
            ("title", &self.title),
            ("accTitle", &self.acc_title),
            ("accDescr", &self.acc_descr),
        ] {
            if let Some(v) = value {
                m.serialize_entry(key, v)?
            }
        }
        m.serialize_entry("dateFormat", &self.date_format)?;
        m.serialize_entry("axisFormat", &self.axis_format)?;
        if let Some(v) = &self.tick_interval {
            m.serialize_entry("tickInterval", v)?
        }
        m.serialize_entry("todayMarker", &self.today_marker)?;
        m.serialize_entry("includes", &self.includes)?;
        m.serialize_entry("excludes", &self.excludes)?;
        m.serialize_entry("inclusiveEndDates", &self.inclusive_end_dates)?;
        m.serialize_entry("topAxis", &self.top_axis)?;
        m.serialize_entry("displayMode", &self.display_mode)?;
        if let Some(v) = &self.weekday {
            m.serialize_entry("weekday", v)?
        }
        m.serialize_entry("weekend", &self.weekend)?;
        m.serialize_entry("sections", &self.sections)?;
        m.serialize_entry("links", &self.links)?;
        m.serialize_entry("tasks", &self.tasks)?;
        m.serialize_entry(
            "byId",
            &self
                .by_id
                .iter()
                .map(|(id, i)| (id, &self.tasks[*i]))
                .collect::<Vec<_>>(),
        )?;
        m.serialize_entry("currentSection", &self.current_section)?;
        m.serialize_entry("taskCount", &self.task_count)?;
        m.serialize_entry("lastOrder", &self.last_order)?;
        if let Some(v) = &self.last_task_id {
            m.serialize_entry("lastTaskId", v)?
        }
        m.serialize_entry("options", &serde_json::json!({}))?;
        m.serialize_entry("now", &self.now)?;
        m.serialize_entry("work", &self.work)?;
        m.serialize_entry("includeSet", &serde_json::json!({}))?;
        m.serialize_entry("excludeSet", &serde_json::json!({}))?;
        m.serialize_entry("excludedDays", &self.excluded_days)?;
        m.serialize_entry("datedTokens", &self.dated_tokens)?;
        m.end()
    }
}
pub fn error(message: impl Into<String>, code: ErrorCode) -> PeleError {
    let mut e = PeleError::new(message, code);
    e.diagram_type = Some("gantt".into());
    e
}
pub(super) fn patterns() -> &'static Vec<Regex> {
    static R: OnceLock<Vec<Regex>> = OnceLock::new();
    R.get_or_init(|| {
        [
            r"^[YMDHhmsSZXx\-/.:, T]*$",
            r"^\s*(active|done|crit|milestone|vert)\s*$",
            r"^after\s+([0-9A-Za-z_ -]+)",
            r"^until\s+([0-9A-Za-z_ -]+)",
            r"^([0-9]+(?:\.[0-9]+)?)([Mdhmswy]|ms)$",
            r"[\s,]+",
            r"^[0-9]+$",
        ]
        .iter()
        .map(|s| compiled_regex(s).unwrap())
        .collect()
    })
}
impl GanttModel {
    pub fn new(title: Option<&str>, display_mode: String) -> Self {
        Self {
            task_at: HashMap::new(),
            public_task_orders: HashMap::new(),
            public_task_identities: HashMap::new(),
            by_at: HashMap::new(),
            title: title.map(str::to_string),
            acc_title: None,
            acc_descr: None,
            date_format: String::new(),
            axis_format: String::new(),
            tick_interval: None,
            today_marker: String::new(),
            includes: vec![],
            excludes: vec![],
            inclusive_end_dates: false,
            top_axis: false,
            display_mode,
            weekday: None,
            weekend: "saturday".into(),
            sections: vec![],
            links: vec![],
            tasks: vec![],
            by_id: vec![],
            current_section: String::new(),
            task_count: 0,
            last_order: 0,
            last_task_id: None,
            now: 0.,
            work: 0.,
            include_set: HashSet::new(),
            exclude_set: HashSet::new(),
            excluded_days: vec![],
            dated_tokens: false,
        }
    }
    pub fn tokens(existing: &mut Vec<String>, text: &str) {
        for t in patterns()[5]
            .split(&text.to_lowercase())
            .filter(|s| !s.is_empty())
        {
            if !existing.iter().any(|s| s == t) {
                existing.push(t.into())
            }
        }
    }
    pub fn add_section(&mut self, text: String) {
        self.current_section = text.clone();
        self.sections.push(text)
    }
    pub fn add_task(&mut self, descr: String, data: String) -> Result<(), PeleError> {
        let declaration = task_declaration(&descr, &data)?;
        let id = match declaration.id {
            Some(id) => id.to_owned(),
            None => {
                self.task_count += 1;
                format!("task{}", self.task_count)
            }
        };
        let order = if declaration.vert {
            -1
        } else {
            let value = self.last_order as isize;
            self.last_order += 1;
            value
        };
        let task = GanttTask {
            id,
            task: descr,
            section: self.current_section.clone(),
            start_time: None,
            end_time: None,
            render_end_time: None,
            manual_end_time: false,
            processed: false,
            active: declaration.active,
            done: declaration.done,
            crit: declaration.crit,
            milestone: declaration.milestone,
            vert: declaration.vert,
            order,
            classes: vec![],
            prev_task_id: self.last_task_id.clone(),
            raw: TaskRaw {
                data: data.clone(),
                start: declaration.start.map(str::to_owned),
                end: declaration.end.to_owned(),
            },
        };
        self.last_task_id = Some(task.id.clone());
        if let Some(&at) = self.by_at.get(&task.id) {
            self.by_id[at].1 = self.tasks.len()
        } else {
            self.by_at.insert(task.id.clone(), self.by_id.len());
            self.by_id.push((task.id.clone(), self.tasks.len()))
        }
        self.task_at.insert(task.id.clone(), self.tasks.len());
        self.tasks.push(task);
        Ok(())
    }
    pub fn class(&mut self, ids: &str) {
        for id in ids.split(',') {
            if let Some(i) = self.task_at.get(id) {
                self.tasks[*i].classes.push("clickable".into())
            }
        }
    }
    pub fn link(&mut self, ids: &str, url: &str) -> Result<(), PeleError> {
        let href = safe_url(url)?;
        for id in ids.split(',') {
            if self.task_at.contains_key(id) {
                if let Some((_, v)) = self.links.iter_mut().find(|(k, _)| k == id) {
                    *v = href.clone()
                } else {
                    self.links.push((id.into(), href.clone()))
                }
            }
        }
        self.class(ids);
        Ok(())
    }
    /// Schedule a reusable parsed document against the host calendar and clock.
    pub fn get_tasks(
        &self,
        now: Option<f64>,
        resources: &mut dyn RenderResources,
    ) -> Result<Vec<GanttTask>, PeleError> {
        let now = if let Some(now) = now.filter(|n| n.is_finite()) {
            now
        } else {
            resources.now_millis()?
        };
        Ok(self.scheduled(now, resources)?.tasks)
    }
    pub fn scheduled(
        &self,
        now: f64,
        resources: &mut dyn RenderResources,
    ) -> Result<Self, PeleError> {
        let now = if now.is_finite() {
            now
        } else {
            resources.now_millis()?
        };
        let mut m = self.clone();
        let mut cal = Calendar::new(resources);
        m.schedule(now, &mut cal)?;
        if let Some(e) = cal.error {
            return Err(e);
        }
        Ok(m)
    }
    pub(crate) fn schedule(&mut self, now: f64, cal: &mut Calendar<'_>) -> Result<(), PeleError> {
        super::scheduler::schedule(self, now, cal)
    }
    pub(crate) fn invalid(&mut self, date: f64, cal: &mut Calendar<'_>) -> Result<bool, PeleError> {
        Ok(super::scheduler::invalid(self, &date, cal)? == Some(true))
    }
    pub(super) fn set_public_task_identity(&mut self, index: usize, identity: u32) {
        self.public_task_identities.insert(index, identity);
    }
    pub(super) fn public_task_identity(&self, index: usize) -> u64 {
        self.public_task_identities
            .get(&index)
            .map_or((index as u64) << 1, |id| (u64::from(*id) << 1) | 1)
    }
    pub(super) fn set_public_task_order(&mut self, index: usize, order: f64) {
        self.public_task_orders.insert(index, order);
    }
    pub(super) fn public_task_order(&self, index: usize) -> f64 {
        self.public_task_orders
            .get(&index)
            .copied()
            .unwrap_or(self.tasks[index].order as f64)
    }
    pub(super) fn public_schedule_cache(&self) -> (Vec<String>, Vec<String>, Vec<bool>, bool) {
        let ordered = |values: &Vec<String>, set: &HashSet<String>| {
            let mut seen = HashSet::new();
            values
                .iter()
                .filter(|s| set.contains(*s) && seen.insert((*s).clone()))
                .cloned()
                .collect()
        };
        (
            ordered(&self.includes, &self.include_set),
            ordered(&self.excludes, &self.exclude_set),
            self.excluded_days.clone(),
            self.dated_tokens,
        )
    }
    pub(super) fn import_public_cache(
        &mut self,
        now: f64,
        work: f64,
        includes: Vec<String>,
        excludes: Vec<String>,
        days: Vec<bool>,
        dated: bool,
    ) {
        self.now = now;
        self.work = work;
        self.include_set = includes.into_iter().collect();
        self.exclude_set = excludes.into_iter().collect();
        self.excluded_days = days;
        self.dated_tokens = dated;
    }
    pub(super) fn public_builder_state(&self) -> (&str, usize, usize, Option<&str>) {
        (
            &self.current_section,
            self.task_count,
            self.last_order,
            self.last_task_id.as_deref(),
        )
    }
}

fn compiled_regex(pattern: &str) -> Result<Regex, regex::Error> {
    Regex::new(&pattern.replace(r"\s", crate::compat::text::SPACE))
}

impl super::scheduler::Store for GanttModel {
    type Task = usize;
    type TaskIterator = std::ops::Range<usize>;
    type Date = f64;
    type Token = String;
    type Error = PeleError;
    fn now(&mut self) -> Result<f64, PeleError> {
        Ok(self.now)
    }
    fn set_now(&mut self, n: f64) -> Result<(), PeleError> {
        self.now = n;
        Ok(())
    }
    fn work(&mut self) -> Result<f64, PeleError> {
        Ok(self.work)
    }
    fn set_work(&mut self, n: f64) -> Result<(), PeleError> {
        self.work = n;
        Ok(())
    }
    fn tokens(&mut self, include: bool) -> Result<Vec<String>, PeleError> {
        Ok(if include {
            &self.includes
        } else {
            &self.excludes
        }
        .clone())
    }
    fn token_is(&self, token: &String, text: &str) -> bool {
        token == text
    }
    fn token_length(&mut self, include: bool) -> Result<f64, PeleError> {
        Ok(if include {
            self.includes.len()
        } else {
            self.excludes.len()
        } as f64)
    }
    fn set_tokens(&mut self, include: bool, tokens: Vec<String>) -> Result<(), PeleError> {
        if include {
            self.include_set = tokens.into_iter().collect()
        } else {
            self.exclude_set = tokens.into_iter().collect()
        }
        Ok(())
    }
    fn contains(&mut self, include: bool, text: &str) -> Result<bool, PeleError> {
        Ok(if include {
            &self.include_set
        } else {
            &self.exclude_set
        }
        .contains(text))
    }
    fn weekend_start(&mut self) -> Result<Option<usize>, PeleError> {
        Ok(match self.weekend.as_str() {
            "friday" => Some(5),
            "saturday" => Some(6),
            _ => None,
        })
    }
    fn set_excluded_days(&mut self, days: Vec<bool>) -> Result<(), PeleError> {
        self.excluded_days = days;
        Ok(())
    }
    fn excluded_day(&mut self, day: Option<usize>) -> Result<Option<bool>, PeleError> {
        Ok(day.and_then(|day| self.excluded_days.get(day).copied()))
    }
    fn dated_tokens(&mut self) -> Result<bool, PeleError> {
        Ok(self.dated_tokens)
    }
    fn set_dated_tokens(&mut self, value: bool) -> Result<(), PeleError> {
        self.dated_tokens = value;
        Ok(())
    }
    fn date_format(&mut self) -> Result<String, PeleError> {
        Ok(self.date_format.clone())
    }
    fn inclusive(&mut self) -> Result<bool, PeleError> {
        Ok(self.inclusive_end_dates)
    }
    fn task_iterator(&mut self) -> Result<Self::TaskIterator, PeleError> {
        Ok(0..self.tasks.len())
    }
    fn next_task(&mut self, iterator: &mut Self::TaskIterator) -> Result<Option<usize>, PeleError> {
        Ok(iterator.next())
    }
    fn tasks(&mut self) -> Result<Vec<usize>, PeleError> {
        Ok((0..self.tasks.len()).collect())
    }
    fn previous(&mut self, task: &usize) -> Result<Option<usize>, PeleError> {
        Ok(self.tasks[*task]
            .prev_task_id
            .as_ref()
            .and_then(|id| self.task_at.get(id))
            .copied())
    }
    fn lookup(&mut self, id: &str) -> Result<Option<usize>, PeleError> {
        Ok(self.task_at.get(id).copied())
    }
    fn raw_start(&mut self, task: &usize) -> Result<Option<String>, PeleError> {
        Ok(self.tasks[*task].raw.start.clone())
    }
    fn raw_end(&mut self, task: &usize) -> Result<String, PeleError> {
        Ok(self.tasks[*task].raw.end.clone())
    }
    fn time(
        &mut self,
        task: &usize,
        field: super::scheduler::Time,
    ) -> Result<Option<f64>, PeleError> {
        use super::scheduler::Time;
        let task = &self.tasks[*task];
        Ok(match field {
            Time::Start => task.start_time,
            Time::End => task.end_time,
            Time::RenderEnd => task.render_end_time,
        })
    }
    fn set_time(
        &mut self,
        task: &usize,
        field: super::scheduler::Time,
        value: Option<f64>,
    ) -> Result<(), PeleError> {
        use super::scheduler::Time;
        let task = &mut self.tasks[*task];
        match field {
            Time::Start => task.start_time = value,
            Time::End => task.end_time = value,
            Time::RenderEnd => task.render_end_time = value,
        }
        Ok(())
    }
    fn date(&mut self, ms: f64) -> Result<f64, PeleError> {
        Ok(super::dates::clip(ms))
    }
    fn milliseconds(&mut self, date: &f64) -> Result<f64, PeleError> {
        Ok(*date)
    }
    fn date_truth(&self, _: &f64) -> bool {
        true
    }
    fn manual(&mut self, task: &usize) -> Result<bool, PeleError> {
        Ok(self.tasks[*task].manual_end_time)
    }
    fn processed(&mut self, task: &usize) -> Result<bool, PeleError> {
        Ok(self.tasks[*task].processed)
    }
    fn set_flag(
        &mut self,
        task: &usize,
        field: super::scheduler::Flag,
        value: bool,
    ) -> Result<(), PeleError> {
        use super::scheduler::Flag;
        match field {
            Flag::Processed => self.tasks[*task].processed = value,
            Flag::ManualEnd => self.tasks[*task].manual_end_time = value,
        }
        Ok(())
    }
    fn warn(&mut self, _: String) -> Result<(), PeleError> {
        Ok(())
    }
}
