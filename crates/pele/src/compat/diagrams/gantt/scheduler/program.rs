//! Owned resource operands and replies; no document/store borrow crosses a run.
use super::super::model::error;
use super::Calendar;
use crate::compat::{ErrorCode, PeleError};
pub(in crate::compat::diagrams::gantt) enum CalendarRequest {
    Today(f64),
    Parse {
        text: String,
        template: String,
        now: f64,
    },
    Loose(String),
    AddDays {
        date: f64,
        amount: f64,
    },
    Add {
        date: f64,
        amount: f64,
        unit: String,
    },
    Weekday(f64),
    Iso(f64),
    Format {
        date: f64,
        template: String,
    },
}
pub(in crate::compat::diagrams::gantt) enum CalendarReply {
    Number(Option<f64>),
    Text(Option<String>),
}
impl CalendarReply {
    pub(in crate::compat::diagrams::gantt) fn number(self) -> Option<f64> {
        match self {
            Self::Number(v) => v,
            _ => unreachable!("native calendar reply kind"),
        }
    }
    pub(super) fn text(self) -> Option<String> {
        match self {
            Self::Text(v) => v,
            _ => unreachable!("native calendar reply kind"),
        }
    }
}
impl CalendarRequest {
    pub(in crate::compat::diagrams::gantt) fn returns_text(&self) -> bool {
        matches!(self, Self::Iso(_) | Self::Format { .. })
    }
    pub(in crate::compat::diagrams::gantt) fn run(
        self,
        cal: &mut Calendar<'_>,
    ) -> Result<CalendarReply, PeleError> {
        use CalendarReply::{Number, Text};
        let value = match self {
            Self::Today(now) => Number(Some(cal.today(now))),
            Self::Parse {
                text,
                template,
                now,
            } => Number(cal.parse_strict(&text, &template, now)),
            Self::Loose(text) => {
                let date = cal.resources.parse_date(&text)?.unwrap_or(f64::NAN);
                let fields = if date.is_finite() {
                    cal.fields(date)
                } else {
                    None
                };
                cal.check()?;
                if !date.is_finite()
                    || fields.is_none_or(|f| !(-10000..=10000).contains(&f.civil.year))
                {
                    return Err(error(format!("Invalid date:{text}"), ErrorCode::Semantic));
                }
                Number(Some(date))
            }
            Self::AddDays { date, amount } => Number(Some(cal.add_days(date, amount))),
            Self::Add { date, amount, unit } => Number(Some(cal.add(date, amount, &unit))),
            Self::Weekday(date) => Number(cal.fields(date).map(|f| f.weekday as f64)),
            Self::Iso(date) => Text(Some(cal.iso(date))),
            Self::Format { date, template } => Text(cal.format(date, &template)),
        };
        cal.check()?;
        Ok(value)
    }
}
pub(in crate::compat::diagrams::gantt) enum Warning {
    References {
        keyword: &'static str,
        missing: Vec<String>,
    },
    Duration {
        text: String,
        template: String,
    },
}
impl Warning {
    pub(in crate::compat::diagrams::gantt) fn message(self) -> String {
        match self {
        Self::References {keyword,missing}=>format!("Gantt: \"{keyword}\" refers to task ids that do not exist: {}.",missing.join(", ")),
        Self::Duration {text,template}=>format!("Gantt: \"{text}\" is neither a date in the \"{template}\" format nor a duration such as \"3d\", so the task gets no duration."),
    }
    }
}
pub(in crate::compat::diagrams::gantt) enum Step<T> {
    Resource(CalendarRequest),
    Warning(Warning),
    Complete(T),
}
