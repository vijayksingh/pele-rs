//! Gantt drawing reads, shared by typed storage and detached live owners.
use super::{dates::Calendar, model::GanttModel};
use crate::compat::{PeleError, RenderResources};

pub(super) trait PhaseInput {
    type Task: Clone;
    type Error: From<PeleError>;
    fn root_text(&mut self, key: &'static str) -> Result<Option<String>, Self::Error>;
    fn root_bool(&mut self, key: &'static str) -> Result<bool, Self::Error>;
    fn tasks(
        &mut self,
        now: f64,
        resources: &mut dyn RenderResources,
    ) -> Result<Vec<Self::Task>, Self::Error>;
    fn text(&mut self, task: &Self::Task, key: &'static str) -> Result<String, Self::Error>;
    fn flag(&mut self, task: &Self::Task, key: &'static str) -> Result<bool, Self::Error>;
    fn date(
        &mut self,
        task: &Self::Task,
        key: &'static str,
        resources: &mut dyn RenderResources,
    ) -> Result<f64, Self::Error>;
    fn render_end(
        &mut self,
        task: &Self::Task,
        resources: &mut dyn RenderResources,
    ) -> Result<f64, Self::Error>;
    fn order(
        &mut self,
        task: &Self::Task,
        resources: &mut dyn RenderResources,
    ) -> Result<f64, Self::Error>;
    fn identity(&mut self, task: &Self::Task) -> Result<u64, Self::Error>;
    fn clickable(&mut self, task: &Self::Task) -> Result<bool, Self::Error>;
    fn link(&mut self, task: &Self::Task) -> Result<Option<(String, String)>, Self::Error>;
    fn has_exclusions(&mut self) -> Result<bool, Self::Error>;
    fn invalid(&mut self, day: f64, cal: &mut Calendar<'_>) -> Result<bool, Self::Error>;
    fn work(&mut self, _n: usize) -> Result<(), Self::Error> {
        Ok(())
    }
    fn keep_text(&mut self, _n: usize) -> Result<(), Self::Error> {
        Ok(())
    }
    fn scheduled(
        &mut self,
        task: &Self::Task,
        resources: &mut dyn RenderResources,
    ) -> Result<bool, Self::Error> {
        Ok(self.date(task, "startTime", resources)?.is_finite()
            && self.date(task, "endTime", resources)?.is_finite())
    }
}
pub(super) struct Typed<'a> {
    pub model: &'a mut GanttModel,
    pub schedule: bool,
    pub filter_callable: bool,
    pub missing_filter: &'a mut bool,
}
impl PhaseInput for Typed<'_> {
    type Task = usize;
    type Error = PeleError;
    fn root_text(&mut self, key: &'static str) -> Result<Option<String>, PeleError> {
        let m = &self.model;
        Ok(match key {
            "title" => m.title.clone(),
            "accTitle" => m.acc_title.clone(),
            "accDescr" => m.acc_descr.clone(),
            "displayMode" => Some(m.display_mode.clone()),
            "axisFormat" => Some(m.axis_format.clone()),
            "dateFormat" => Some(m.date_format.clone()),
            "tickInterval" => m.tick_interval.clone(),
            "weekday" => m.weekday.clone(),
            "todayMarker" => Some(m.today_marker.clone()),
            _ => None,
        })
    }
    fn root_bool(&mut self, key: &'static str) -> Result<bool, PeleError> {
        Ok(key == "topAxis" && self.model.top_axis)
    }
    fn tasks(
        &mut self,
        now: f64,
        resources: &mut dyn RenderResources,
    ) -> Result<Vec<usize>, PeleError> {
        if self.schedule {
            let mut cal = Calendar::new(resources);
            self.model.schedule(now, &mut cal)?;
            if let Some(e) = cal.error {
                return Err(e);
            }
        }
        Ok((0..self.model.tasks.len()).collect())
    }
    fn text(&mut self, t: &usize, k: &'static str) -> Result<String, PeleError> {
        let t = &self.model.tasks[*t];
        Ok(match k {
            "id" => t.id.clone(),
            "task" => t.task.clone(),
            "section" => t.section.clone(),
            _ => String::new(),
        })
    }
    fn flag(&mut self, t: &usize, k: &'static str) -> Result<bool, PeleError> {
        let t = &self.model.tasks[*t];
        Ok(match k {
            "vert" => t.vert,
            "active" => t.active,
            "done" => t.done,
            "crit" => t.crit,
            "milestone" => t.milestone,
            _ => false,
        })
    }
    fn date(
        &mut self,
        t: &usize,
        k: &'static str,
        _: &mut dyn RenderResources,
    ) -> Result<f64, PeleError> {
        let t = &self.model.tasks[*t];
        Ok(match k {
            "startTime" => t.start_time,
            "endTime" => t.end_time,
            "renderEndTime" => t.render_end_time,
            _ => None,
        }
        .unwrap_or(f64::NAN))
    }
    fn render_end(&mut self, t: &usize, _: &mut dyn RenderResources) -> Result<f64, PeleError> {
        let t = &self.model.tasks[*t];
        Ok(t.render_end_time.or(t.end_time).unwrap_or(f64::NAN))
    }
    fn order(&mut self, t: &usize, _: &mut dyn RenderResources) -> Result<f64, PeleError> {
        Ok(self.model.public_task_order(*t))
    }
    fn identity(&mut self, t: &usize) -> Result<u64, PeleError> {
        Ok(self.model.public_task_identity(*t) as u64)
    }
    fn clickable(&mut self, t: &usize) -> Result<bool, PeleError> {
        Ok(self.model.tasks[*t]
            .classes
            .iter()
            .any(|s| s == "clickable"))
    }
    fn link(&mut self, t: &usize) -> Result<Option<(String, String)>, PeleError> {
        let id = &self.model.tasks[*t].id;
        Ok(self
            .model
            .links
            .iter()
            .find(|(key, _)| key == id)
            .map(|(_, url)| (id.clone(), url.clone())))
    }
    fn has_exclusions(&mut self) -> Result<bool, PeleError> {
        Ok(!self.model.excludes.is_empty() || !self.model.includes.is_empty())
    }
    fn invalid(&mut self, day: f64, cal: &mut Calendar<'_>) -> Result<bool, PeleError> {
        if !self.filter_callable {
            *self.missing_filter = true;
            return Err(PeleError::new(
                "Gantt missing date filter",
                crate::compat::ErrorCode::Semantic,
            ));
        }
        self.model.invalid(day, cal)
    }
}
