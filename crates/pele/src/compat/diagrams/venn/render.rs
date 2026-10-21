//! Venn region labels, merged styles and scaled circle/overlap SVG, Pele 458322a5 (MIT).
use super::math::{hypot, max, min};
use super::{
    layout::{self, Circle, Spot},
    model::{normalize, semantic, VennModel, VennStyle, VennSubset, VennText},
};
use crate::compat::number_math::round;
use crate::compat::{
    svg::{
        self,
        edges::theme::{resolve_style, ResolvedStyle},
        esc_text, label_svg, num,
    },
    text::{layout_label, Label},
    Object, PeleError, RenderResources, RenderSettings, Rendered,
};
use std::collections::{HashMap, HashSet};
fn key(ids: &[String]) -> String {
    if ids.len() == 1 {
        ids[0].clone()
    } else {
        serde_json::to_string(ids).unwrap()
    }
}
struct Line {
    id: Option<String>,
    raw: String,
    size: f64,
    base: u8,
    label: Label,
}
struct Region {
    sets: Vec<String>,
    inside: Vec<usize>,
    spot: Spot,
    lines: Vec<Line>,
    style: ResolvedStyle,
}
fn style_of(key: &str, styles: &HashMap<String, Vec<(String, String)>>) -> ResolvedStyle {
    let list: Vec<_> = styles.get(key).map_or_else(Vec::new, |v| {
        v.iter().map(|(k, v)| format!("{k}:{v}")).collect()
    });
    resolve_style(&list)
}
/// Geometry/styles and the ordered region map are captured before the first
/// width callback. Region records and the later text iterator remain live.
pub(super) trait PhaseInput {
    type Error: From<PeleError>;
    type Subset;
    type Texts;
    fn geometry(&mut self) -> Result<Vec<VennSubset>, Self::Error>;
    fn styles(&mut self) -> Result<Vec<VennStyle>, Self::Error>;
    fn stated(&mut self) -> Result<Vec<(String, Self::Subset)>, Self::Error>;
    fn subset(&mut self, value: &Self::Subset) -> Result<VennSubset, Self::Error>;
    fn texts(&mut self) -> Result<Self::Texts, Self::Error>;
    fn next_text(
        &mut self,
        values: &Self::Texts,
        index: usize,
        regions: &HashMap<String, usize>,
    ) -> Result<Option<Option<VennText>>, Self::Error>;
    fn title(&mut self) -> Result<Option<String>, Self::Error>;
}
struct Typed<'a>(&'a VennModel);
impl PhaseInput for Typed<'_> {
    type Error = PeleError;
    type Subset = usize;
    type Texts = ();
    fn geometry(&mut self) -> Result<Vec<VennSubset>, PeleError> {
        Ok(self.0.subsets.clone())
    }
    fn styles(&mut self) -> Result<Vec<VennStyle>, PeleError> {
        Ok(self.0.styles.clone())
    }
    fn stated(&mut self) -> Result<Vec<(String, usize)>, PeleError> {
        let mut values = Vec::new();
        let mut positions = HashMap::new();
        for (i, s) in self.0.subsets.iter().enumerate() {
            let k = key(&s.sets);
            if let Some(&slot) = positions.get(&k) {
                values[slot] = (k, i);
            } else {
                positions.insert(k.clone(), values.len());
                values.push((k, i));
            }
        }
        Ok(values)
    }
    fn subset(&mut self, i: &usize) -> Result<VennSubset, PeleError> {
        Ok(self.0.subsets[*i].clone())
    }
    fn texts(&mut self) -> Result<(), PeleError> {
        Ok(())
    }
    fn next_text(
        &mut self,
        _: &(),
        i: usize,
        _: &HashMap<String, usize>,
    ) -> Result<Option<Option<VennText>>, PeleError> {
        Ok(self.0.text_nodes.get(i).cloned().map(Some))
    }
    fn title(&mut self) -> Result<Option<String>, PeleError> {
        Ok(self.0.title.clone())
    }
}
pub(super) fn render(
    model: &VennModel,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Rendered, PeleError> {
    render_phases(&mut Typed(model), c, o, r)
}
pub(super) fn render_phases<P: PhaseInput>(
    model: &mut P,
    _c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Rendered, P::Error> {
    let subsets = model.geometry()?;
    if subsets.iter().any(|s| s.sets.is_empty()) {
        return Err(semantic("Venn subsets require at least one set identifier.").into());
    }
    let size = o.font_size.unwrap_or(16.);
    let small = round(size * 0.875);
    let tiny = round(size * 0.8125);
    let pad = o.padding.unwrap_or(8.);
    let circles = layout::layout(&subsets)?;
    let index: HashMap<_, _> = circles
        .iter()
        .enumerate()
        .map(|(i, c)| (c.id.clone(), i))
        .collect();
    let mut styles: HashMap<String, Vec<(String, String)>> = HashMap::new();
    for declared in model.styles()? {
        let merged = styles.entry(key(&declared.targets)).or_default();
        for (prop, value) in &declared.styles {
            if let Some((_, old)) = merged.iter_mut().find(|(p, _)| p == prop) {
                *old = value.clone()
            } else {
                merged.push((prop.clone(), value.clone()))
            }
        }
    }
    let mut neighbors = vec![vec![]; circles.len()];
    for a in 0..circles.len() {
        for b in a + 1..circles.len() {
            if hypot(circles[a].x - circles[b].x, circles[a].y - circles[b].y)
                < circles[a].r + circles[b].r
            {
                neighbors[a].push(b);
                neighbors[b].push(a);
            }
        }
    }
    let stated = model.stated()?;
    let positions: HashMap<String, usize> = stated
        .iter()
        .enumerate()
        .map(|(i, (k, _))| (k.clone(), i))
        .collect();
    let mut regions = vec![];
    for (k, handle) in stated {
        let s = model.subset(&handle)?;
        let mut seen = HashSet::new();
        let sets: Vec<_> = s
            .sets
            .iter()
            .filter(|id| seen.insert((*id).clone()))
            .cloned()
            .collect();
        let inside: Vec<_> = sets.iter().map(|id| index.get(id).copied().ok_or_else(|| semantic("Venn callback introduced a region identifier absent from captured geometry"))).collect::<Result<_,_>>()?;
        if inside.is_empty() {
            return Err(semantic("Venn subsets require at least one set identifier.").into());
        }
        let within: HashSet<_> = inside.iter().copied().collect();
        let mut smallest = inside[0];
        for &i in &inside {
            if circles[i].r < circles[smallest].r {
                smallest = i
            }
        }
        let style = style_of(&k, &styles);
        let text = s
            .label
            .as_deref()
            .or_else(|| (s.sets.len() == 1).then(|| s.sets[0].as_str()));
        let base = if style.bold { 1 } else { 0 } | if style.italic { 2 } else { 0 };
        let outside: Vec<_> = neighbors[smallest]
            .iter()
            .copied()
            .filter(|i| !within.contains(i))
            .collect();
        let mut spot = layout::find_spot(&circles, &inside, &outside);
        if spot.margin < circles[smallest].r * 0.2 {
            spot = layout::find_spot(&circles, &inside, &[])
        }
        let mut region = Region {
            sets,
            inside,
            spot,
            lines: vec![],
            style,
        };
        if let Some(text) = text.filter(|s| !s.is_empty()) {
            let font_size = if region.sets.len() == 1 { size } else { small };
            region.lines.push(Line {
                id: None,
                raw: text.into(),
                size: font_size,
                base,
                label: layout_label(Some(text), false, r, o, font_size, 4000., base)?,
            });
        }
        regions.push(region);
    }
    let texts = model.texts()?;
    let mut text_index = 0;
    while let Some(node) = model.next_text(&texts, text_index, &positions)? {
        text_index += 1;
        let Some(node) = node else { continue };
        let raw = node.label.as_deref().unwrap_or(&node.id);
        if let Some(&i) = positions.get(&key(&node.sets)) {
            regions[i].lines.push(Line {
                id: Some(node.id.clone()),
                raw: raw.into(),
                size: tiny,
                base: 0,
                label: layout_label(Some(raw), false, r, o, tiny, 4000., 0)?,
            });
        }
    }
    let mut largest = 0.;
    for c in &circles {
        if c.r > largest {
            largest = c.r
        }
    }
    let mut scale = if largest > 0. { 100. / largest } else { 1. };
    let mut growth = 1f64;
    for region in &regions {
        if region.spot.margin <= 0. {
            continue;
        }
        let (mut width, mut height) = (0f64, 0f64);
        for l in &region.lines {
            width = max(width, l.label.width);
            height += l.label.height;
        }
        let need = hypot(width, height) / 2. + 4.;
        growth = max(growth, min(need / (region.spot.margin * scale), 1.6));
    }
    scale *= growth;
    for region in &mut regions {
        let room = max(region.spot.margin * scale * 1.8, 72.);
        for l in &mut region.lines {
            if l.label.width > room {
                l.label = layout_label(Some(&l.raw), false, r, o, l.size, room, l.base)?;
            }
        }
    }
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    );
    for c in &circles {
        min_x = min(min_x, (c.x - c.r) * scale);
        min_y = min(min_y, (c.y - c.r) * scale);
        max_x = max(max_x, (c.x + c.r) * scale);
        max_y = max(max_y, (c.y + c.r) * scale);
    }
    for region in &regions {
        let (mut wide, mut tall) = (0f64, 0f64);
        for l in &region.lines {
            wide = max(wide, l.label.width);
            tall += l.label.height;
        }
        min_x = min(min_x, region.spot.x * scale - wide / 2.);
        max_x = max(max_x, region.spot.x * scale + wide / 2.);
        min_y = min(min_y, region.spot.y * scale - tall / 2.);
        max_y = max(max_y, region.spot.y * scale + tall / 2.);
    }
    if circles.is_empty() {
        min_x = 0.;
        min_y = 0.;
        max_x = 0.;
        max_y = 0.;
    }
    let current_title = model.title()?;
    let title_text = current_title
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(normalize);
    let title = layout_label(title_text.as_deref(), false, r, o, size, 4000., 1)?;
    let title_height = if title.height > 0. {
        title.height + 12.
    } else {
        0.
    };
    let inner = max(max_x - min_x, title.width);
    let ox = pad + (inner - (max_x - min_x)) / 2. - min_x;
    let oy = pad + title_height - min_y;
    let width = (inner + 2. * pad).ceil();
    let height = (max_y - min_y + title_height + 2. * pad).ceil();
    let placed: Vec<_> = circles
        .iter()
        .map(|c| Circle {
            id: c.id.clone(),
            x: c.x * scale + ox,
            y: c.y * scale + oy,
            r: c.r * scale,
        })
        .collect();
    let mut sets = String::new();
    for (i, c) in placed.iter().enumerate() {
        let color = svg::series_color(i);
        let style = positions
            .get(&c.id)
            .map_or_else(ResolvedStyle::default, |i| regions[*i].style.clone());
        sets+=&format!("<circle class=\"pele-venn-set\" data-id=\"{}\" cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{color}\" fill-opacity=\"0.2\" stroke=\"{color}\"{}/>",esc_text(&c.id)?,num(c.x),num(c.y),num(c.r),style.shape);
    }
    let mut overlaps = String::new();
    let mut labels = String::new();
    for region in &regions {
        let id = esc_text(&region.sets.join("|"))?;
        if region.sets.len() > 1 && !region.style.shape.is_empty() {
            let circles: Vec<_> = region.inside.iter().map(|i| placed[*i].clone()).collect();
            let path = layout::region_path(&circles, num);
            if !path.is_empty() {
                overlaps+=&format!("<path class=\"pele-venn-overlap\" data-id=\"{id}\" d=\"{path}\" fill=\"none\"{}/>",region.style.shape);
            }
        }
        let mut total = 0.;
        for l in &region.lines {
            total += l.label.height;
        }
        if total == 0. {
            continue;
        }
        let x = region.spot.x * scale + ox;
        let mut y = region.spot.y * scale + oy - total / 2.;
        let mut content = String::new();
        for l in &region.lines {
            let attrs = if let Some(id) = &l.id {
                format!(" class=\"pele-venn-text\" data-id=\"{}\" font-size=\"{}\" fill=\"var(--_m)\"{}",esc_text(id)?,svg::js_num(tiny),style_of(id,&styles).text)
            } else {
                format!(
                    " class=\"pele-label\"{}{}",
                    if l.size == size {
                        String::new()
                    } else {
                        format!(" font-size=\"{}\"", svg::js_num(l.size))
                    },
                    region.style.text
                )
            };
            content += &label_svg(&l.label, x, y + l.label.height / 2., &attrs);
            y += l.label.height;
        }
        labels += &format!("<g class=\"pele-venn-region\" data-id=\"{id}\">{content}</g>");
    }
    let content = label_svg(
        &title,
        width / 2.,
        pad + title.height / 2.,
        " class=\"pele-title\" font-weight=\"var(--_tw)\"",
    ) + &format!(
        "<g class=\"pele-venn-sets\">{sets}</g>{}{}",
        if overlaps.is_empty() {
            String::new()
        } else {
            format!("<g class=\"pele-venn-overlaps\">{overlaps}</g>")
        },
        if labels.is_empty() {
            String::new()
        } else {
            format!("<g class=\"pele-venn-labels\">{labels}</g>")
        }
    );
    Ok(Rendered {
        svg: svg::document("venn", width, height, size, o, None, None, &content)?,
        width,
        height,
        links: vec![],
    })
}
