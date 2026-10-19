//! Exact output-limit comparison from Pele core.ts at 458322a5 (MIT, kepano).
use super::{ErrorCode, PeleError, Value, DEFAULT_OUTPUT_LIMIT};

/// Independent returned-SVG byte ceiling for the additive exact-output lane.
/// This bounds accepted output, not peak allocation inside a trusted renderer.
pub const MAX_PUBLIC_OUTPUT_BYTES: usize = 64 * 1024 * 1024;

/// JavaScript output threshold. Both undefined and null select the default;
/// number values preserve direct IEEE comparison and JavaScript formatting.
#[derive(Clone, Copy, Debug, Default)]
pub enum OutputLimit {
    #[default]
    Undefined,
    Null,
    Number(f64),
}
impl OutputLimit {
    pub(crate) fn check_bytes(svg: &str, diagram: &str) -> Result<(), PeleError> {
        if svg.len() > MAX_PUBLIC_OUTPUT_BYTES {
            return Err(Self::byte_error(diagram));
        }
        Ok(())
    }
    pub(crate) fn byte_error(diagram: &str) -> PeleError {
        let mut error =
            PeleError::new("Native render output exceeds byte budget", ErrorCode::Limit);
        error.diagram_type = Some(diagram.to_owned());
        error
    }
    pub(crate) fn check(self, svg: &str, diagram: &str) -> Result<(), PeleError> {
        Self::check_bytes(svg, diagram)?;
        let threshold = match self {
            Self::Undefined | Self::Null => DEFAULT_OUTPUT_LIMIT as f64,
            Self::Number(value) => value,
        };
        // UTF-8 bytes bound UTF-16 code units from above. Most output is far
        // below the default threshold; avoid walking every returned SVG there.
        // JavaScript's comparison against NaN is always false as well. Keep the
        // independent byte guard above both shortcuts.
        if svg.len() as f64 <= threshold || threshold.is_nan() {
            return Ok(());
        }
        // The independent byte ceiling keeps this length exactly representable.
        if svg.encode_utf16().count() as f64 > threshold {
            let mut error = PeleError::new(
                format!(
                    "Diagram output is longer than the limit of {} characters.",
                    Value::Number(threshold).js_string()
                ),
                ErrorCode::Limit,
            );
            error.diagram_type = Some(diagram.to_owned());
            return Err(error);
        }
        Ok(())
    }
}
