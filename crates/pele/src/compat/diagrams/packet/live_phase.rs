//! Packet host reads retain row/block identity across resource callbacks.
use super::render::{Field, PhaseInput};
use crate::compat::{
    public::{LiveRenderRead, LiveStringUse, LiveValueKind, PublicError},
    ErrorCode, Object, PeleError, RenderResources, RenderSettings, Rendered,
};
use std::sync::Arc;
pub(crate) struct PacketDetachedInput {
    limit: Option<PeleError>,
}
impl PacketDetachedInput {
    pub(super) fn new(error: Option<&PeleError>) -> Self {
        Self {
            limit: error
                .filter(|error| error.code == ErrorCode::Limit)
                .cloned(),
        }
    }
}
struct HostPhase<'a, R>(&'a mut R);
fn invalid(message: &str) -> PublicError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("packet".into());
    e.into()
}
impl<R: LiveRenderRead> HostPhase<'_, R> {
    fn field(&mut self, key: &str) -> Result<R::Value, PublicError> {
        let root = self.0.root();
        self.0.get(&root, key)
    }
    fn numeric(&self, value: &R::Value) -> Result<f64, PublicError> {
        match self.0.kind(value) {
            LiveValueKind::Number => Ok(self.0.number(value).unwrap_or(f64::NAN)),
            LiveValueKind::Undefined => Ok(f64::NAN),
            LiveValueKind::Null => Ok(0.),
            LiveValueKind::Boolean => Ok(u8::from(self.0.truthy(value)) as f64),
            _ => Err(invalid("Packet numeric renderer fields must be numbers")),
        }
    }
    fn len(&mut self, value: &R::Value, max: usize) -> Result<usize, PublicError> {
        let length = self.0.get(value, "length")?;
        let n = self
            .0
            .number(&length)
            .ok_or_else(|| invalid("Packet rows and row entries must be Arrays"))?;
        if !n.is_finite() || n < 0. || n.fract() != 0. || n > u32::MAX as f64 {
            return Err(invalid("Packet rows and row entries must be Arrays"));
        }
        if n > max as f64 {
            let mut e = PeleError::new(
                "Packet public model expansion exceeds the work budget",
                ErrorCode::Limit,
            );
            e.diagram_type = Some("packet".into());
            return Err(e.into());
        }
        Ok(n as usize)
    }
    fn strict_text(
        &mut self,
        value: &R::Value,
        receiver: &'static str,
    ) -> Result<String, PublicError> {
        self.0
            .require_string(
                value,
                LiveStringUse {
                    method: "includes",
                    expression_receiver: receiver,
                },
            )?
            .well_formed()
            .ok_or_else(|| invalid("Packet rendering of lone UTF-16 surrogates is not implemented"))
    }
    fn nullish(&self, value: &R::Value) -> bool {
        matches!(
            self.0.kind(value),
            LiveValueKind::Null | LiveValueKind::Undefined
        )
    }
}
impl<R: LiveRenderRead> PhaseInput for HostPhase<'_, R> {
    type Error = PublicError;
    type Rows = R::Value;
    type Row = R::Value;
    type Block = R::Value;
    fn bits_per_row(&mut self) -> Result<f64, PublicError> {
        let value = self.field("bitsPerRow")?;
        self.numeric(&value)
    }
    fn rows(&mut self) -> Result<R::Value, PublicError> {
        self.field("rows")
    }
    fn last_end(&mut self) -> Result<f64, PublicError> {
        let rows = self.field("rows")?;
        let length_rows = self.field("rows")?;
        let n = self.len(&length_rows, 1_000_000)?;
        let row = if n == 0 {
            self.0.get(&rows, "-1")?
        } else {
            self.0.index(&rows, n - 1)?
        };
        if self.nullish(&row) {
            return Ok(0.);
        }
        let n = self.len(&row, 8_000_000)?;
        let block = if n == 0 {
            self.0.get(&row, "-1")?
        } else {
            self.0.index(&row, n - 1)?
        };
        if self.nullish(&block) {
            return Ok(0.);
        }
        let value = self.0.get(&block, "end")?;
        if self.nullish(&value) {
            Ok(0.)
        } else {
            self.numeric(&value)
        }
    }
    fn row_count(&mut self, rows: &R::Value) -> Result<usize, PublicError> {
        self.len(rows, 1_000_000)
    }
    fn has_row(&mut self, rows: &R::Value, index: usize) -> Result<bool, PublicError> {
        self.0.has_index(rows, index)
    }
    fn row(&mut self, rows: &R::Value, index: usize) -> Result<R::Value, PublicError> {
        self.0.index(rows, index)
    }
    fn block_count(&mut self, row: &R::Value) -> Result<usize, PublicError> {
        self.len(row, 8_000_000)
    }
    fn block(&mut self, row: &R::Value, index: usize) -> Result<R::Value, PublicError> {
        self.0.index(row, index)
    }
    fn number(&mut self, block: &R::Value, field: Field) -> Result<f64, PublicError> {
        let value = self.0.get(block, field.name())?;
        if matches!(field, Field::Bits) {
            Ok(self.0.number(&value).unwrap_or(f64::NAN))
        } else {
            self.numeric(&value)
        }
    }
    fn display(&mut self, block: &R::Value, field: Field) -> Result<String, PublicError> {
        let value = self.0.get(block, field.name())?;
        self.0
            .to_string(&value)?
            .well_formed()
            .ok_or_else(|| invalid("Packet rendering of lone UTF-16 surrogates is not implemented"))
    }
    fn label(&mut self, block: &R::Value) -> Result<Arc<str>, PublicError> {
        let value = self.0.get(block, "label")?;
        self.strict_text(&value, "e").map(Into::into)
    }
    fn copy_block(&mut self, block: &R::Value) -> Result<R::Value, PublicError> {
        self.0.shallow_copy(block)
    }
    fn title(&mut self) -> Result<Option<String>, PublicError> {
        let value = self.field("title")?;
        if self.0.kind(&value) == LiveValueKind::Undefined {
            Ok(None)
        } else {
            self.strict_text(&value, "t").map(Some)
        }
    }
    fn accessibility(&mut self, description: bool) -> Result<Option<String>, PublicError> {
        let key = if description { "accDescr" } else { "accTitle" };
        let value = self.field(key)?;
        if !self.0.truthy(&value) {
            return Ok(None);
        }
        let value = self.field(key)?;
        self.strict_text(&value, "e").map(Some)
    }
}
pub(crate) fn render_detached<R: LiveRenderRead>(
    input: &PacketDetachedInput,
    config: &Object,
    settings: &RenderSettings,
    read: &mut R,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PublicError> {
    if let Some(error) = &input.limit {
        return Err(error.clone().into());
    }
    super::render::render_phases(&mut HostPhase(read), config, settings, resources)
}
