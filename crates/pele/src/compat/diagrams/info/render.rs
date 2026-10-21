//! Pele info/render.ts, 458322a5 (MIT, kepano).
use super::InfoModel;
use crate::compat::{
    svg::{document, label_svg},
    text::layout_label,
    PeleError, RenderResources, RenderSettings, Rendered,
};
pub(super) trait PhaseInput {
    type Error: From<PeleError>;
    fn title(&mut self) -> Result<Option<String>, Self::Error>;
    fn accessibility(&mut self, description: bool) -> Result<Option<String>, Self::Error>;
}
struct Typed<'a>(&'a InfoModel);
impl PhaseInput for Typed<'_> {
    type Error = PeleError;
    fn title(&mut self) -> Result<Option<String>, PeleError> {
        Ok(self.0.title.clone())
    }
    fn accessibility(&mut self, description: bool) -> Result<Option<String>, PeleError> {
        Ok(if description {
            self.0.acc_descr.clone()
        } else {
            self.0.acc_title.clone()
        })
    }
}
// Layout resolves host icons; rendering preserves their reserved slots with empty markup.
pub(super) fn render(
    model: &InfoModel,
    options: &RenderSettings,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PeleError> {
    render_phases(&mut Typed(model), options, resources)
}
pub(super) fn render_phases<P: PhaseInput>(
    input: &mut P,
    options: &RenderSettings,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, P::Error> {
    let title_text = input.title()?;
    let size = options.font_size.unwrap_or(16.);
    let pad = options.padding.unwrap_or(8.);
    let title = layout_label(
        title_text.as_deref(),
        false,
        resources,
        options,
        size,
        4000.,
        1,
    )?;
    let title_height = if title.height > 0. {
        title.height + 8.
    } else {
        0.
    };
    let text = layout_label(
        Some("Pele v1.6.0"),
        false,
        resources,
        options,
        size,
        4000.,
        0,
    )?;
    let width = (title.width.max(text.width) + 2. * pad).ceil();
    let height = (title_height + text.height + 2. * pad).ceil();
    let content = format!(
        "{}{}",
        label_svg(
            &title,
            width / 2.,
            pad + title.height / 2.,
            " class=\"pele-title\" font-weight=\"var(--_tw)\""
        ),
        label_svg(
            &text,
            width / 2.,
            pad + title_height + text.height / 2.,
            " class=\"pele-version\" fill=\"var(--_m)\""
        )
    );
    let acc_title = input.accessibility(false)?;
    let acc_descr = input.accessibility(true)?;
    Ok(Rendered {
        svg: document(
            "info",
            width,
            height,
            size,
            options,
            acc_title.as_deref(),
            acc_descr.as_deref(),
            &content,
        )?,
        width,
        height,
        links: Vec::new(),
    })
}
