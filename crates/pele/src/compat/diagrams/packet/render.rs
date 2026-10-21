//! Pele packet/render.ts and common/{literal-label,fit-text,fit-width}.ts, 458322a5 (MIT, kepano).
use super::{packet_option, PacketBlock, PacketModel};
use crate::compat::number_math::{round, NumberMath};
use crate::compat::{
    source::{decode_entities, space},
    svg::{self, esc, js_num, label_svg, num},
    text::{layout_label, measure, Label, Span},
    ErrorCode, Object, PeleError, RenderResources, RenderSettings, Rendered, Value,
};
use std::sync::Arc;
fn literal(
    text: &str,
    r: &mut dyn RenderResources,
    o: &RenderSettings,
    size: f64,
    room: f64,
) -> Result<Label, PeleError> {
    let mut lines = Vec::new();
    for raw in text.split('\n') {
        let chars: Vec<char> = raw.chars().collect();
        if chars.is_empty() {
            lines.push(String::new())
        }
        let mut from = 0;
        while from < chars.len() {
            let fits = |n: usize, r: &mut dyn RenderResources| {
                measure(
                    r,
                    o,
                    &chars[from..from + n].iter().collect::<String>(),
                    size,
                    0,
                )
                .map(|w| w <= room)
            };
            let mut hi = 1;
            while hi < chars.len() - from && fits(hi, r)? {
                hi = (chars.len() - from).min(hi * 2)
            }
            let mut lo = 1;
            while lo < hi {
                let mid = (lo + hi + 1) / 2;
                if fits(mid, r)? {
                    lo = mid
                } else {
                    hi = mid - 1
                }
            }
            let mut take = lo;
            let wraps = from + take < chars.len();
            if wraps && !space(chars[from + take]) {
                for i in (1..take).rev() {
                    if space(chars[from + i]) {
                        take = i + 1;
                        break;
                    }
                }
            }
            let line: String = chars[from..from + take].iter().collect();
            let trimmed = line.trim_end_matches(space);
            lines.push(if wraps && !trimmed.is_empty() {
                trimmed.into()
            } else {
                line
            });
            from += take;
            while wraps && from < chars.len() && space(chars[from]) {
                from += 1
            }
        }
    }
    if text.is_empty() {
        lines.clear()
    }
    let line_height = round(size * 1.5);
    let mut out = Label {
        size,
        line_height,
        ..Label::default()
    };
    for line in lines {
        let width = measure(r, o, &line, size, 0)?;
        out.width = out.width.js_max(width);
        out.widths.push(width);
        out.lines.push(vec![Span {
            text: line,
            style: 0,
            icon: None,
            width,
        }]);
    }
    out.height = out.lines.len() as f64 * line_height;
    Ok(out)
}
fn fit(
    text: &str,
    max: f64,
    size: f64,
    min: f64,
    r: &mut dyn RenderResources,
    o: &RenderSettings,
) -> Result<Option<(String, f64)>, PeleError> {
    if text.is_empty() || max <= 0. {
        return Ok(None);
    }
    let width = measure(r, o, text, size, 0)?;
    if width <= max {
        return Ok(Some((text.into(), size)));
    }
    let mut fitted = min.js_max((size * max / width).floor());
    while fitted >= min {
        if measure(r, o, text, fitted, 0)? <= max {
            return Ok(Some((text.into(), fitted)));
        }
        fitted -= 1.;
    }
    let chars: Vec<char> = text.chars().collect();
    let mut low = 0;
    let mut high = chars.len() - 1;
    while low < high {
        let mid = (low + high + 1) / 2;
        let s = chars[..mid]
            .iter()
            .collect::<String>()
            .trim_end_matches(space)
            .to_string()
            + "…";
        if measure(r, o, &s, min, 0)? <= max {
            low = mid
        } else {
            high = mid - 1
        }
    }
    if low == 0 {
        return Ok(None);
    }
    let cut = chars[..low]
        .iter()
        .collect::<String>()
        .trim_end_matches(space)
        .to_string()
        + "…";
    Ok(if measure(r, o, &cut, min, 0)? <= max {
        Some((cut, min))
    } else {
        None
    })
}
#[derive(Clone, Copy)]
pub(super) enum Field {
    Start,
    End,
    Bits,
}
impl Field {
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::End => "end",
            Self::Bits => "bits",
        }
    }
}
pub(super) trait PhaseInput {
    type Error: From<PeleError>;
    type Rows;
    type Row;
    type Block: Clone;
    fn bits_per_row(&mut self) -> Result<f64, Self::Error>;
    fn rows(&mut self) -> Result<Self::Rows, Self::Error>;
    fn last_end(&mut self) -> Result<f64, Self::Error>;
    fn row_count(&mut self, rows: &Self::Rows) -> Result<usize, Self::Error>;
    fn has_row(&mut self, rows: &Self::Rows, index: usize) -> Result<bool, Self::Error>;
    fn row(&mut self, rows: &Self::Rows, index: usize) -> Result<Self::Row, Self::Error>;
    fn block_count(&mut self, row: &Self::Row) -> Result<usize, Self::Error>;
    fn block(&mut self, row: &Self::Row, index: usize) -> Result<Self::Block, Self::Error>;
    fn number(&mut self, block: &Self::Block, field: Field) -> Result<f64, Self::Error>;
    fn display(&mut self, block: &Self::Block, field: Field) -> Result<String, Self::Error>;
    fn label(&mut self, block: &Self::Block) -> Result<Arc<str>, Self::Error>;
    fn copy_block(&mut self, block: &Self::Block) -> Result<Self::Block, Self::Error>;
    fn title(&mut self) -> Result<Option<String>, Self::Error>;
    fn accessibility(&mut self, description: bool) -> Result<Option<String>, Self::Error>;
}
struct Typed<'a> {
    model: &'a PacketModel,
    holes: &'a [usize],
}
impl<'a> PhaseInput for Typed<'a> {
    type Error = PeleError;
    type Rows = &'a [Vec<PacketBlock>];
    type Row = &'a [PacketBlock];
    type Block = PacketBlock;
    fn bits_per_row(&mut self) -> Result<f64, PeleError> {
        Ok(self.model.bits_per_row)
    }
    fn rows(&mut self) -> Result<Self::Rows, PeleError> {
        Ok(&self.model.rows)
    }
    fn last_end(&mut self) -> Result<f64, PeleError> {
        Ok(self
            .model
            .rows
            .last()
            .and_then(|row| row.last())
            .map_or(0., |b| b.end))
    }
    fn row_count(&mut self, rows: &Self::Rows) -> Result<usize, PeleError> {
        Ok(rows.len())
    }
    fn has_row(&mut self, _: &Self::Rows, i: usize) -> Result<bool, PeleError> {
        Ok(self.holes.binary_search(&i).is_err())
    }
    fn row(&mut self, rows: &Self::Rows, i: usize) -> Result<Self::Row, PeleError> {
        if self.holes.binary_search(&i).is_ok() {
            Err(absent_row())
        } else {
            Ok(&rows[i])
        }
    }
    fn block_count(&mut self, row: &Self::Row) -> Result<usize, PeleError> {
        Ok(row.len())
    }
    fn block(&mut self, row: &Self::Row, i: usize) -> Result<Self::Block, PeleError> {
        Ok(row[i].clone())
    }
    fn number(&mut self, b: &Self::Block, f: Field) -> Result<f64, PeleError> {
        Ok(block_number(b, f))
    }
    fn display(&mut self, b: &Self::Block, f: Field) -> Result<String, PeleError> {
        Ok(js_num(block_number(b, f)))
    }
    fn label(&mut self, b: &Self::Block) -> Result<Arc<str>, PeleError> {
        Ok(b.label.clone())
    }
    fn copy_block(&mut self, b: &Self::Block) -> Result<PacketBlock, PeleError> {
        Ok(b.clone())
    }
    fn title(&mut self) -> Result<Option<String>, PeleError> {
        Ok(self.model.title.clone())
    }
    fn accessibility(&mut self, description: bool) -> Result<Option<String>, PeleError> {
        Ok(if description {
            self.model.acc_descr.clone()
        } else {
            self.model.acc_title.clone()
        })
    }
}
fn block_number(b: &PacketBlock, f: Field) -> f64 {
    match f {
        Field::Start => b.start,
        Field::End => b.end,
        Field::Bits => b.bits,
    }
}
fn limit() -> PeleError {
    let mut e = PeleError::new(
        "Packet public model expansion exceeds the work budget",
        ErrorCode::Limit,
    );
    e.diagram_type = Some("packet".into());
    e
}
fn absent_row() -> PeleError {
    let mut e = PeleError::new("row is not iterable", ErrorCode::Semantic);
    e.diagram_type = Some("packet".into());
    e
}
#[derive(Clone)]
struct CopiedBlock<B> {
    value: B,
    start: Option<f64>,
    end: Option<f64>,
    bits: Option<f64>,
}
impl<B> CopiedBlock<B> {
    fn number<P: PhaseInput<Block = B>>(&self, p: &mut P, f: Field) -> Result<f64, P::Error> {
        let override_value = match f {
            Field::Start => self.start,
            Field::End => self.end,
            Field::Bits => self.bits,
        };
        match override_value {
            Some(n) => Ok(n),
            None => p.number(&self.value, f),
        }
    }
    fn display<P: PhaseInput<Block = B>>(&self, p: &mut P, f: Field) -> Result<String, P::Error> {
        let override_value = match f {
            Field::Start => self.start,
            Field::End => self.end,
            Field::Bits => self.bits,
        };
        match override_value {
            Some(n) => Ok(js_num(n)),
            None => p.display(&self.value, f),
        }
    }
}
enum DrawRows<P: PhaseInput> {
    Live(P::Rows),
    Reflow(Vec<Option<Vec<CopiedBlock<P::Block>>>>),
}
enum DrawRow<R> {
    Live(R),
    Reflow(usize),
}
enum DrawBlock<B> {
    Live(B),
    Copied(CopiedBlock<B>),
}
impl<P: PhaseInput> DrawRows<P> {
    fn len(&self, p: &mut P) -> Result<usize, P::Error> {
        match self {
            Self::Live(r) => p.row_count(r),
            Self::Reflow(r) => Ok(r.len()),
        }
    }
    fn has(&self, p: &mut P, i: usize) -> Result<bool, P::Error> {
        match self {
            Self::Live(r) => p.has_row(r, i),
            Self::Reflow(r) => Ok(r.get(i).is_some_and(Option::is_some)),
        }
    }
    fn row(&self, p: &mut P, i: usize) -> Result<DrawRow<P::Row>, P::Error> {
        match self {
            Self::Live(r) => p.row(r, i).map(DrawRow::Live),
            Self::Reflow(_) => Ok(DrawRow::Reflow(i)),
        }
    }
    fn block_count(&self, p: &mut P, r: &DrawRow<P::Row>) -> Result<usize, P::Error> {
        match r {
            DrawRow::Live(r) => p.block_count(r),
            DrawRow::Reflow(i) => {
                let Self::Reflow(rows) = self else {
                    unreachable!()
                };
                rows[*i]
                    .as_ref()
                    .map(Vec::len)
                    .ok_or_else(|| absent_row().into())
            }
        }
    }
    fn block(
        &self,
        p: &mut P,
        r: &DrawRow<P::Row>,
        i: usize,
    ) -> Result<DrawBlock<P::Block>, P::Error> {
        match r {
            DrawRow::Live(r) => p.block(r, i).map(DrawBlock::Live),
            DrawRow::Reflow(n) => {
                let Self::Reflow(rows) = self else {
                    unreachable!()
                };
                Ok(DrawBlock::Copied(
                    rows[*n]
                        .as_ref()
                        .ok_or_else(|| P::Error::from(absent_row()))?[i]
                        .clone(),
                ))
            }
        }
    }
}
impl<B> DrawBlock<B> {
    fn number<P: PhaseInput<Block = B>>(&self, p: &mut P, f: Field) -> Result<f64, P::Error> {
        match self {
            Self::Live(b) => p.number(b, f),
            Self::Copied(b) => b.number(p, f),
        }
    }
    fn display<P: PhaseInput<Block = B>>(&self, p: &mut P, f: Field) -> Result<String, P::Error> {
        match self {
            Self::Live(b) => p.display(b, f),
            Self::Copied(b) => b.display(p, f),
        }
    }
    fn label<P: PhaseInput<Block = B>>(&self, p: &mut P) -> Result<Arc<str>, P::Error> {
        match self {
            Self::Live(b) => p.label(b),
            Self::Copied(b) => p.label(&b.value),
        }
    }
}
pub(super) fn render(
    m: &PacketModel,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Rendered, PeleError> {
    render_with_row_holes(m, c, o, r, &[])
}
/// Sparse outer rows preserve Array.map/forEach behavior. Typed source models
/// remain dense; mutable public documents provide the absent index sidecar.
pub(super) fn render_with_row_holes(
    m: &PacketModel,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
    holes: &[usize],
) -> Result<Rendered, PeleError> {
    render_phases(&mut Typed { model: m, holes }, c, o, r)
}
pub(super) fn render_phases<P: PhaseInput>(
    input: &mut P,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Rendered, P::Error> {
    let across = draw(input, c, o, r, false)?;
    let Some(available) = o.max_width.filter(|w| *w > 0.) else {
        return Ok(across);
    };
    let breakpoint = o.direction_breakpoint.filter(|v| *v > 0.).unwrap_or(640.);
    if o.auto_direction == Some(false) || across.width <= available || available >= breakpoint {
        return Ok(across);
    }
    let down = draw(input, c, o, r, true)?;
    Ok(if down.width < across.width {
        down
    } else {
        across
    })
}
fn draw<P: PhaseInput>(
    input: &mut P,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
    down: bool,
) -> Result<Rendered, P::Error> {
    let size = o.font_size.unwrap_or(16.);
    let label_size = round(size * 0.875);
    let bit_size = round(size * 0.625);
    let pad = o.padding.unwrap_or(8.);
    let packet = c.get("packet").and_then(Value::as_object);
    let row_height = packet_option(c, "rowHeight", 32., 1., 400.);
    let mut bit_width = packet_option(c, "bitWidth", 32., 1., 400.);
    let padding_x = packet_option(c, "paddingX", 5., 0., 400.);
    let padding_y = packet_option(c, "paddingY", 5., 0., 400.);
    let show_bits = packet.and_then(|p| p.get("showBits")) != Some(&Value::Bool(false));
    let descending =
        packet.and_then(|p| p.get("bitOrder")) == Some(&Value::String("descending".into()));
    let mut bits_per_row = input.bits_per_row()?;
    let original_rows = input.rows()?;
    let mut rows = DrawRows::Live(original_rows);
    if down {
        let count = ((o.max_width.unwrap_or(320.) - 2. * pad) / bit_width.js_min(18.))
            .floor()
            .js_max(1.);
        bits_per_row = bits_per_row.js_min(2f64.powf(count.log2().floor()).js_max(8.));
        let last = input.last_end()?;
        if ((last + 1.) / bits_per_row).ceil() > 10000. {
            bits_per_row = input.bits_per_row()?;
        }
        bit_width =
            bit_width.js_min(((o.max_width.unwrap_or(320.) - 2. * pad) / bits_per_row).js_max(18.));
        let source = input.rows()?;
        let mut whole: Vec<CopiedBlock<P::Block>> = Vec::new();
        let mut i = 0;
        let mut work = 0usize;
        while i < input.row_count(&source)? {
            let row = input.row(&source, i)?;
            let mut j = 0;
            while j < input.block_count(&row)? {
                work += 1;
                if work > 8_000_000 {
                    return Err(limit().into());
                }
                let block = input.block(&row, j)?;
                let mut merge = false;
                if let Some(prev) = whole.last() {
                    if prev.number(input, Field::Bits)?
                        == prev.number(input, Field::End)? - prev.number(input, Field::Start)?
                    {
                        let bits = input.number(&block, Field::Bits)?;
                        let end = input.number(&block, Field::End)?;
                        let start = input.number(&block, Field::Start)?;
                        if bits == end - start
                            && input.label(&prev.value)? == input.label(&block)?
                            && input.number(&block, Field::Start)?
                                == prev.number(input, Field::End)? + 1.
                        {
                            let start = input.number(&block, Field::Start)?;
                            merge = start % input.bits_per_row()? == 0.;
                        }
                    }
                }
                if merge {
                    let prev = whole.last_mut().expect("matching preceding block");
                    prev.end = Some(input.number(&block, Field::End)?);
                    prev.bits =
                        Some(prev.number(input, Field::End)? - prev.number(input, Field::Start)?);
                } else {
                    whole.push(CopiedBlock {
                        value: input.copy_block(&block)?,
                        start: None,
                        end: None,
                        bits: None,
                    });
                }
                j += 1;
            }
            i += 1;
        }
        let mut split: Vec<Option<Vec<CopiedBlock<P::Block>>>> = Vec::new();
        let mut split_work = 0usize;
        for block in whole {
            let mut start = block.number(input, Field::Start)?;
            while start <= block.number(input, Field::End)? {
                let end = block
                    .number(input, Field::End)?
                    .js_min(((start / bits_per_row).floor() + 1.) * bits_per_row - 1.);
                let index = (start / bits_per_row).floor();
                let next = end + 1.;
                split_work += 1;
                if split_work > 1_000_000
                    || index >= 1_000_000.
                    || next == start
                    || (next < start && bits_per_row.is_finite())
                {
                    return Err(limit().into());
                }
                if index >= 0. && index.is_finite() {
                    let index = index as usize;
                    if split.len() <= index {
                        split.resize_with(index + 1, || None);
                    }
                    split[index].get_or_insert_with(Vec::new).push(CopiedBlock {
                        start: Some(start),
                        end: Some(end),
                        ..block.clone()
                    });
                }
                start = next;
            }
        }
        rows = DrawRows::Reflow(split);
    }
    let mut labels: Vec<Vec<Label>> = Vec::new();
    if down {
        let mut i = 0;
        while i < rows.len(input)? {
            let row = rows.row(input, i)?;
            let mut line = Vec::new();
            let mut j = 0;
            while j < rows.block_count(input, &row)? {
                let block = rows.block(input, &row, j)?;
                let room =
                    ((block.number(input, Field::End)? - block.number(input, Field::Start)? + 1.)
                        * bit_width
                        - padding_x.js_min(bit_width - 1.)
                        - 6.)
                        .js_max(1.);
                let label = block.label(input)?;
                line.push(literal(
                    decode_entities(&label)?.trim_matches(space),
                    r,
                    o,
                    label_size,
                    room,
                )?);
                j += 1;
            }
            labels.push(line);
            i += 1;
        }
    }
    let numbers = if show_bits { bit_size + 4. } else { 0. };
    let mut heights = Vec::new();
    let mut tops = Vec::new();
    let mut rows_height = 0.;
    let height_count = rows.len(input)?;
    for i in 0..height_count {
        let present = rows.has(input, i)?;
        if present {
            let _ = rows.row(input, i)?;
        }
        let mut height = if !present { f64::NAN } else { row_height };
        if down {
            for label in &labels[i] {
                height = height.js_max(label.height + 10.)
            }
        }
        tops.push(rows_height);
        rows_height += numbers + height + padding_y;
        heights.push(height);
    }
    if rows.len(input)? != 0 {
        rows_height -= padding_y
    }
    let gap = padding_x.js_min(bit_width - 1.);
    let width = if rows.len(input)? != 0 {
        bits_per_row * bit_width - gap
    } else {
        0.
    };
    let title_room = if down {
        o.max_width
            .filter(|v| *v > 0.)
            .map_or(4000., |v| (v - 2. * pad).js_max(80.))
    } else {
        4000.
    };
    let title_text = input.title()?;
    let title = layout_label(title_text.as_deref(), false, r, o, size, title_room, 1)?;
    let title_height = if title.height > 0. {
        title.height + if rows_height > 0. { 12. } else { 0. }
    } else {
        0.
    };
    let left = pad + ((title.width - width) / 2.).js_max(0.);
    let mut fields = String::new();
    let mut bits = String::new();
    let mut fits: Vec<(std::sync::Arc<str>, f64, Option<(String, f64)>)> = Vec::new();
    let paint_count = rows.len(input)?;
    let mut paint_work = 0usize;
    for i in 0..paint_count {
        if !rows.has(input, i)? {
            continue;
        }
        let row = rows.row(input, i)?;
        let y = pad + tops.get(i).copied().unwrap_or(f64::NAN) + numbers;
        let height = heights.get(i).copied().unwrap_or(f64::NAN);
        let mut j = 0;
        while j < rows.block_count(input, &row)? {
            paint_work += 1;
            if paint_work > 8_000_000 {
                return Err(limit().into());
            }
            let block = rows.block(input, &row, j)?;
            let count = block.number(input, Field::End)? - block.number(input, Field::Start)? + 1.;
            let first = block.number(input, Field::Start)? % bits_per_row;
            let x = left
                + if descending {
                    bits_per_row - first - count
                } else {
                    first
                } * bit_width;
            let w = count * bit_width - gap;
            let fitted = if down {
                None
            } else {
                let label = block.label(input)?;
                let max = w - 6.;
                if let Some((_, _, f)) = fits.iter().find(|(l, v, _)| l == &label && *v == max) {
                    f.clone()
                } else {
                    let f = fit(
                        decode_entities(&label)?.trim_matches(space),
                        max,
                        label_size,
                        round(size * 0.5),
                        r,
                        o,
                    )?;
                    fits.push((label.clone(), max, f.clone()));
                    f
                }
            };
            let text = if down {
                label_svg(&labels[i][j], x + w / 2., y + height / 2., "")
            } else if let Some((text, font)) = fitted.filter(|(_, font)| *font <= height) {
                format!(
                    "<text x=\"{}\" y=\"{}\"{}>{}</text>",
                    num(x + w / 2.),
                    num(y + height / 2. + font * 0.35),
                    if font == label_size {
                        String::new()
                    } else {
                        format!(" font-size=\"{}\"", num(font))
                    },
                    esc(&text)
                )
            } else {
                String::new()
            };
            fields.push_str(&format!("<g class=\"pele-node\" data-id=\"{}-{}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"var(--pele-radius,4px)\" fill=\"var(--_s)\" stroke=\"var(--_b)\"/>{text}</g>",block.display(input,Field::Start)?,block.display(input,Field::End)?,num(x),num(y),num(w),num(height)));
            j += 1;
            if !show_bits {
                continue;
            }
            let leading =
                block.display(input, if descending { Field::End } else { Field::Start })?;
            let trailing =
                block.display(input, if descending { Field::Start } else { Field::End })?;
            if count == 1. {
                bits.push_str(&format!(
                    "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\">{}</text>",
                    num(x + w / 2.),
                    num(y - 4.),
                    leading
                ))
            } else {
                bits.push_str(&format!("<text x=\"{}\" y=\"{}\">{}</text><text x=\"{}\" y=\"{}\" text-anchor=\"end\">{}</text>",num(x),num(y-4.),leading,num(x+w),num(y-4.),trailing))
            }
        }
    }
    let total_width = (width.js_max(title.width) + 2. * pad).ceil();
    let total_height = (rows_height + title_height + 2. * pad).ceil();
    let content = format!(
        "{}{}{}",
        if fields.is_empty() {
            String::new()
        } else {
            format!("<g class=\"pele-packet-fields\" font-size=\"{}\" text-anchor=\"middle\">{fields}</g>",js_num(label_size))
        },
        if bits.is_empty() {
            String::new()
        } else {
            format!(
                "<g class=\"pele-packet-bits\" font-size=\"{}\" fill=\"var(--_m)\">{bits}</g>",
                js_num(bit_size)
            )
        },
        label_svg(
            &title,
            total_width / 2.,
            total_height - pad - title.height / 2.,
            " class=\"pele-title\" font-weight=\"var(--_tw)\""
        )
    );
    let acc_title = input.accessibility(false)?;
    let acc_descr = input.accessibility(true)?;
    Ok(Rendered {
        svg: svg::document(
            "packet",
            total_width,
            total_height,
            size,
            o,
            acc_title.as_deref(),
            acc_descr.as_deref(),
            &content,
        )?,
        width: total_width,
        height: total_height,
        links: Vec::new(),
    })
}
