//! Pele git/render.ts commit lanes, symbols and connectors, 458322a5 (MIT, kepano).
use super::{
    model::GitModel,
    phase::{PhaseInput, Typed},
};
use crate::compat::number_math::{round, NumberMath};
use crate::compat::{
    diagrams::common::plain_label::plain_label,
    svg::{self, esc_text, label_svg, num, series_color},
    text::{layout_label, Label},
    Object, PeleError, RenderResources, RenderSettings, Rendered, Value,
};
use std::collections::HashMap;
const HALF: f64 = std::f64::consts::FRAC_1_SQRT_2;
const RADIUS: &str = "var(--pele-radius,4px)";
const CLASSES: [&str; 5] = ["normal", "reverse", "highlight", "merge", "cherry-pick"];
struct View<V> {
    commit: V,
    lane: usize,
    m: f64,
    label: Option<Label>,
    tags: Vec<Label>,
}
fn flag(config: &Object, key: &str, fallback: bool) -> bool {
    match config
        .get("gitGraph")
        .and_then(Value::as_object)
        .and_then(|g| g.get(key))
    {
        Some(Value::Bool(v)) => *v,
        _ => fallback,
    }
}
fn tag_width(label: &Label) -> f64 {
    7. + 4. + label.width + 6.
}
fn symbol(kind: u8, x: f64, y: f64, color: &str) -> String {
    let at = format!("cx=\"{}\" cy=\"{}\"", num(x), num(y));
    if kind == 2 {
        return format!(
            "<rect x=\"{}\" y=\"{}\" width=\"16\" height=\"16\" rx=\"2\" fill=\"{color}\"/>",
            num(x - 8.),
            num(y - 8.)
        );
    }
    if kind == 3 {
        return format!("<circle {at} r=\"6\" fill=\"var(--_bg)\" stroke=\"{color}\" stroke-width=\"2\"/><circle {at} r=\"2.5\" fill=\"{color}\"/>");
    }
    let dot = format!("<circle {at} r=\"7\" fill=\"{color}\"/>");
    if kind == 1 {
        return format!("{dot}<path d=\"M{},{}l6,6M{},{}l6,-6\" fill=\"none\" stroke=\"var(--_bg)\" stroke-width=\"1.5\" stroke-linecap=\"round\"/>",num(x-3.),num(y-3.),num(x-3.),num(y+3.));
    }
    if kind == 4 {
        return format!("{dot}<path d=\"M{},{}L{},{}L{},{}\" fill=\"none\" stroke=\"var(--_bg)\" stroke-linejoin=\"round\"/><circle cx=\"{}\" cy=\"{}\" r=\"1.8\" fill=\"var(--_bg)\"/><circle cx=\"{}\" cy=\"{}\" r=\"1.8\" fill=\"var(--_bg)\"/>",num(x-2.6),num(y+1.),num(x+0.8),num(y-4.2),num(x+2.6),num(y+1.),num(x-2.6),num(y+2.2),num(x+2.6),num(y+2.2));
    }
    dot
}
struct Bounds {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}
impl Bounds {
    fn grow(&mut self, x0: f64, y0: f64, x1: f64, y1: f64) {
        if x0 < self.min_x {
            self.min_x = x0
        }
        if y0 < self.min_y {
            self.min_y = y0
        }
        if x1 > self.max_x {
            self.max_x = x1
        }
        if y1 > self.max_y {
            self.max_y = y1
        }
    }
}
pub(super) fn render(
    model: &GitModel,
    config: &Object,
    options: &RenderSettings,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PeleError> {
    render_phases(&mut Typed(model), config, options, resources)
}
pub(super) fn render_phases<P: PhaseInput>(
    input: &mut P,
    config: &Object,
    options: &RenderSettings,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, P::Error> {
    let root = input.root();
    let direction = {
        let value = input.get(&root, "direction")?;
        input.string_key(&value)?.unwrap_or_default()
    };
    if direction != "LR" {
        let direction = {
            let value = input.get(&root, "direction")?;
            input.string_key(&value)?.unwrap_or_default()
        };
        return draw(input, config, options, resources, &direction);
    }
    let first = draw(input, config, options, resources, "LR")?;
    let Some(available) = options.max_width.filter(|w| *w > 0.) else {
        return Ok(first);
    };
    if options.auto_direction == Some(false)
        || first.width <= available
        || available
            >= options
                .direction_breakpoint
                .filter(|b| *b > 0.)
                .unwrap_or(640.)
    {
        return Ok(first);
    }
    let down = draw(input, config, options, resources, "TB")?;
    Ok(if down.width < first.width {
        down
    } else {
        first
    })
}
fn draw<P: PhaseInput>(
    input: &mut P,
    config: &Object,
    options: &RenderSettings,
    resources: &mut dyn RenderResources,
    direction: &str,
) -> Result<Rendered, P::Error> {
    input.begin_draw();
    let root = input.root();
    let size = options.font_size.unwrap_or(16.);
    let small = round(size * 0.75);
    let pad = options.padding.unwrap_or(8.);
    let show_branches = flag(config, "showBranches", true);
    let show_label = flag(config, "showCommitLabel", true);
    let rotate = flag(config, "rotateCommitLabel", true);
    let parallel = flag(config, "parallelCommits", false);
    let vertical = direction != "LR";
    let flip = direction == "BT";
    let mut lane_of = HashMap::new();
    let lanes = input.get(&root, "lanes")?;
    let count = input.count(&lanes)?;
    for i in 0..count {
        input.work(1)?;
        if input.present(&lanes, i)? {
            let value = input.index(&lanes, i)?;
            lane_of.insert(input.text(&value)?, i);
        }
    }
    let lanes = input.get(&root, "lanes")?;
    let lane_count = input.count(&lanes)?;
    let medium = round(size * 0.875);
    let mut names = Vec::new();
    let lanes = input.get(&root, "lanes")?;
    let count = input.count(&lanes)?;
    for i in 0..count {
        input.work(1)?;
        let value = input.index(&lanes, i)?;
        let name = if show_branches {
            input.text(&value)?
        } else {
            String::new()
        };
        input.keep_label(name.encode_utf16().count())?;
        names.push(plain_label(
            Some(&name),
            resources,
            options,
            medium,
            4000.,
            0,
        )?);
    }
    let mut views = Vec::new();
    let mut by_id = HashMap::new();
    let map = input.get(&root, "commits")?;
    let commits = input.map_values(&map)?;
    let mut sorted = Vec::with_capacity(commits.len());
    for commit in commits {
        input.work(1)?;
        let seq = input.sequence(&commit)?;
        sorted.push((commit, seq));
    }
    sorted.sort_by(|a, b| a.1.compare(&b.1));
    for (commit, _) in sorted {
        let labelled = show_label
            && input.kind(&commit, "type")? != 4
            && (input.kind(&commit, "type")? != 3 || {
                let v = input.get(&commit, "customId")?;
                input.is_true(&v)
            });
        let mut tags = Vec::new();
        let list = input.get(&commit, "tags")?;
        let mut i = 0;
        while i < input.count(&list)? {
            input.work(1)?;
            let value = input.index(&list, i)?;
            i += 1;
            let tag = input.text(&value)?;
            input.keep_label(tag.encode_utf16().count())?;
            let label = plain_label(Some(&tag), resources, options, small, 4000., 0)?;
            if !label.lines.is_empty() {
                tags.push(label)
            }
        }
        // Branch is read before the commit-id label's callbacks in the object literal.
        let branch = input.get(&commit, "branch")?;
        let branch = input.string_key(&branch)?;
        let lane = branch
            .as_ref()
            .and_then(|key| lane_of.get(key))
            .copied()
            .unwrap_or(0);
        if lane >= lane_count {
            return Err(PeleError::new(
                "GitGraph live lane is outside captured lanes",
                crate::compat::ErrorCode::Semantic,
            )
            .into());
        }
        let label = if labelled {
            let id = input.field_text(&commit, "id")?.unwrap_or_default();
            input.keep_label(id.encode_utf16().count())?;
            Some(plain_label(Some(&id), resources, options, small, 4000., 0)?)
        } else {
            None
        };
        let id = input.field_text(&commit, "id")?.unwrap_or_default();
        by_id.insert(id, views.len());
        views.push(View {
            commit,
            lane,
            m: 0.,
            label,
            tags,
        });
    }
    let mut half = vec![0.; views.len()];
    let mut tag_half = vec![0.; views.len()];
    let mut lo = vec![7_f64; lane_count];
    let mut hi = vec![7_f64; lane_count];
    for (k, view) in views.iter().enumerate() {
        let mut tags_main = 0_f64;
        let mut tags_cross = 0.;
        for tag in &view.tags {
            let w = tag_width(tag);
            if vertical {
                tags_main = tags_main.js_max(tag.height);
                tags_cross += w + 4.
            } else {
                tags_main = tags_main.js_max(w);
                tags_cross += tag.height + 3.
            }
        }
        let mut label_main = 0.;
        let mut label_cross = 0.;
        if let Some(label) = view.label.as_ref().filter(|l| !l.lines.is_empty()) {
            if rotate {
                label_cross = 3. + HALF * (label.width + label.height)
            } else {
                label_main = if vertical { label.height } else { label.width };
                label_cross = 6. + if vertical { label.width } else { label.height }
            }
        }
        half[k] = 7_f64.js_max(tags_main / 2.).js_max(label_main / 2.);
        tag_half[k] = tags_main / 2.;
        let tag_side = 7.
            + if !view.tags.is_empty() {
                6. + tags_cross
            } else {
                0.
            };
        let label_side = 7. + label_cross;
        lo[view.lane] = lo[view.lane].js_max(if vertical { label_side } else { tag_side });
        hi[view.lane] = hi[view.lane].js_max(if vertical { tag_side } else { label_side })
    }
    let mut lead = vec![0.; views.len()];
    if !vertical {
        for (k, view) in views.iter().enumerate() {
            if tag_half[k] == 0. {
                continue;
            }
            let list = input.get(&view.commit, "parents")?;
            for id in input.strings(&list)? {
                if let Some(&p) = id.as_ref().and_then(|key| by_id.get(key)) {
                    if views[p].lane < view.lane {
                        lead[k] = tag_half[k] + 4.
                    }
                }
            }
        }
    }
    let space = |p: usize, k: usize| {
        44_f64
            .js_max(half[p] + half[k] + 8.)
            .js_max(if lead[k] > 0. {
                half[p] + 36. + lead[k]
            } else {
                0.
            })
    };
    for k in 0..views.len() {
        if !parallel {
            views[k].m = if k == 0 {
                0.
            } else {
                views[k - 1].m + space(k - 1, k)
            };
            continue;
        }
        let mut m = 0_f64;
        let list = input.get(&views[k].commit, "parents")?;
        for id in input.strings(&list)? {
            if let Some(&p) = id.as_ref().and_then(|key| by_id.get(key)) {
                if p < k {
                    m = m.js_max(views[p].m + space(p, k))
                }
            }
        }
        views[k].m = m
    }
    let mut end = 0_f64;
    for view in &views {
        end = end.js_max(view.m)
    }
    let mut cross = vec![0.; lane_count];
    for i in 0..lane_count {
        let name = &names[i];
        let reach = if name.lines.is_empty() {
            0.
        } else if vertical {
            name.width / 2. + 8.
        } else {
            name.height / 2. + 2.
        };
        lo[i] = lo[i].js_max(reach);
        hi[i] = hi[i].js_max(reach);
        if i > 0 {
            cross[i] = cross[i - 1] + 44_f64.js_max(hi[i - 1] + lo[i] + 8.)
        }
    }
    let px = |m: f64, c: f64| if vertical { c } else { m };
    let py = |m: f64, c: f64| {
        if vertical {
            if flip {
                -m
            } else {
                m
            }
        } else {
            c
        }
    };
    let point = |m, c| format!("{},{}", num(px(m, c)), num(py(m, c)));
    let mut bounds = Bounds {
        min_x: f64::INFINITY,
        min_y: f64::INFINITY,
        max_x: f64::NEG_INFINITY,
        max_y: f64::NEG_INFINITY,
    };
    let mut branches = String::new();
    if show_branches {
        for i in 0..lane_count {
            let name = &names[i];
            let color = series_color(i);
            let c = cross[i];
            let w = name.width + 16.;
            let h = name.height + 4.;
            let from = if name.lines.is_empty() {
                -20.
            } else {
                -20. - if vertical { h } else { w }
            };
            let cx = px(from + if vertical { h } else { w } / 2., c);
            let cy = py(from + if vertical { h } else { w } / 2., c);
            bounds.grow(px(-20., c), py(-20., c), px(-20., c), py(-20., c));
            bounds.grow(
                px(end + 20., c),
                py(end + 20., c),
                px(end + 20., c),
                py(end + 20., c),
            );
            branches+=&format!("<g class=\"pele-branch\" data-id=\"{}\"><path class=\"pele-lane\" d=\"M{}L{}\" stroke=\"{color}\" stroke-dasharray=\"2 4\"/>",esc_text(&{let lanes=input.get(&root,"lanes")?;let value=input.index(&lanes,i)?;input.text(&value)?})?,point(-20.,c),point(end+20.,c));
            if !name.lines.is_empty() {
                bounds.grow(cx - w / 2., cy - h / 2., cx + w / 2., cy + h / 2.);
                branches+=&format!("<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{RADIUS}\" fill=\"{color}\"/>{}",num(cx-w/2.),num(cy-h/2.),num(w),num(h),label_svg(name,cx,cy," class=\"pele-label\" fill=\"var(--_bg)\""))
            }
            branches += "</g>"
        }
    }
    let mut taken = vec![Vec::new(); lane_count];
    for view in &views {
        taken[view.lane].push(view.m)
    }
    if parallel {
        for list in &mut taken {
            list.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        }
    }
    let busy = |lane: usize, from: f64, to: f64| {
        let list = &taken[lane];
        let low = list.partition_point(|m| *m <= from);
        low < list.len() && list[low] < to
    };
    let mut curves = String::new();
    let mut lines = String::new();
    for (k, view) in views.iter().enumerate() {
        let commit = &view.commit;
        let c2 = cross[view.lane];
        let m2 = view.m;
        let parents = input.get(commit, "parents")?;
        for (j, id) in input.strings(&parents)?.into_iter().enumerate() {
            let Some(&p) = id.as_ref().and_then(|key| by_id.get(key)) else {
                continue;
            };
            if p == k {
                continue;
            }
            let parent = &views[p];
            let c1 = cross[parent.lane];
            let m1 = parent.m;
            if parent.lane == view.lane {
                lines += &format!(
                    "<path class=\"pele-edge\" d=\"M{}L{}\" stroke=\"{}\"/>",
                    point(m1, c1),
                    point(m2, c2),
                    series_color(view.lane)
                );
                continue;
            }
            let merged = j > 0 && input.kind(commit, "type")? == 3;
            let picked = j > 0 && !merged;
            let mut d = format!("M{}", point(m1, c1));
            if m2 <= m1 {
                let mid = (m1 + m2) / 2.;
                d += &format!("C{} {} {}", point(mid, c1), point(mid, c2), point(m2, c2))
            } else if !merged && !busy(view.lane, m1, m2) {
                let s = 36_f64.js_min(m2 - m1);
                d += &format!(
                    "C{} {} {}L{}",
                    point(m1 + s / 2., c1),
                    point(m1 + s / 2., c2),
                    point(m1 + s, c2),
                    point(m2, c2)
                )
            } else {
                let to = if parent.lane < view.lane && m2 - lead[k] > m1 {
                    m2 - lead[k]
                } else {
                    m2
                };
                let mut s = 36_f64.js_min(to - m1);
                let mut along = c1;
                if !merged || busy(parent.lane, m1, m2) {
                    along = if c1 > c2 {
                        c2 + hi[view.lane] + 4.
                    } else {
                        c2 - lo[view.lane] - 4.
                    };
                    s = 36_f64.js_min((to - m1) / 2.);
                    d += &format!(
                        "C{} {} {}",
                        point(m1 + s / 2., c1),
                        point(m1 + s / 2., along),
                        point(m1 + s, along)
                    )
                }
                d += &format!(
                    "L{}C{} {} {}",
                    point(to - s, along),
                    point(to - s / 2., along),
                    point(to - s / 2., c2),
                    point(to, c2)
                );
                if to < m2 {
                    d += &format!("L{}", point(m2, c2))
                }
            }
            curves += &format!(
                "<path class=\"pele-edge{}\" d=\"{d}\" stroke=\"{}\"{}/>",
                if merged {
                    " pele-edge-merge"
                } else if picked {
                    " pele-edge-cherry-pick"
                } else {
                    ""
                },
                series_color(if merged { parent.lane } else { view.lane }),
                if picked {
                    " stroke-dasharray=\"4 3\""
                } else {
                    ""
                }
            )
        }
    }
    let mut commits = String::new();
    for view in &views {
        let commit = &view.commit;
        let x = px(view.m, cross[view.lane]);
        let y = py(view.m, cross[view.lane]);
        let custom = input.get(commit, "customType")?;
        let kind = if input.nullish(&custom) {
            input.kind(commit, "type")?
        } else {
            input.cached_kind(&custom)?
        };
        bounds.grow(x - 8., y - 8., x + 8., y + 8.);
        let written = input.kind(commit, "type")? < 3
            && !input
                .field_text(commit, "message")?
                .unwrap_or_default()
                .is_empty();
        let mut body = if written {
            format!(
                "<title>{}</title>",
                esc_text(&input.field_text(commit, "message")?.unwrap_or_default())?
            )
        } else {
            String::new()
        };
        body += &symbol(kind, x, y, &series_color(view.lane));
        if let Some(label) = view.label.as_ref().filter(|l| !l.lines.is_empty()) {
            let (w, h) = (label.width, label.height);
            let back = |cx: f64, cy: f64| {
                format!("<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"3\" fill=\"var(--_bg)\"/>",num(cx-w/2.-2.),num(cy-h/2.+2.),num(w+4.),num(h-4.))
            };
            if rotate {
                let k = HALF * h / 2.;
                let ax = if vertical { x - 7. - 3. - k } else { x + 3. };
                let ay = if vertical { y } else { y + 7. + 3. + k };
                let reach = HALF * w + k;
                bounds.grow(
                    ax - reach,
                    if flip { ay - reach } else { ay - k },
                    ax + k,
                    if flip { ay + k } else { ay + reach },
                );
                body += &format!(
                    "<g class=\"pele-commit-label\" transform=\"rotate({} {} {})\">{}{}</g>",
                    if flip { 45 } else { -45 },
                    num(ax),
                    num(ay),
                    back(ax - w / 2., ay),
                    label_svg(label, ax - w / 2., ay, "")
                )
            } else {
                let cx = if vertical { x - 7. - 6. - w / 2. } else { x };
                let cy = if vertical { y } else { y + 7. + 6. + h / 2. };
                bounds.grow(cx - w / 2., cy - h / 2., cx + w / 2., cy + h / 2.);
                body += &format!(
                    "<g class=\"pele-commit-label\">{}{}</g>",
                    back(cx, cy),
                    label_svg(label, cx, cy, "")
                )
            }
        }
        let mut offset = 13.;
        let tags: Box<dyn Iterator<Item = &Label>> = if vertical {
            Box::new(view.tags.iter())
        } else {
            Box::new(view.tags.iter().rev())
        };
        for tag in tags {
            let w = tag_width(tag);
            let h = tag.height;
            let left = if vertical { x + offset } else { x - w / 2. };
            let cy = if vertical { y } else { y - offset - h / 2. };
            offset += if vertical { w + 4. } else { h + 3. };
            bounds.grow(left, cy - h / 2., left + w, cy + h / 2.);
            body+=&format!("<g class=\"pele-tag\"><path d=\"M{},{}L{},{}H{}V{}H{}Z\" fill=\"var(--_s)\" stroke=\"var(--_b)\" stroke-linejoin=\"round\"/><circle cx=\"{}\" cy=\"{}\" r=\"1.5\" fill=\"var(--_b)\"/>{}</g>",num(left),num(cy),num(left+7.),num(cy-h/2.),num(left+w),num(cy+h/2.),num(left+7.),num(left+7.),num(cy),label_svg(tag,left+7.+4.+tag.width/2.,cy," fill=\"var(--_fg)\""))
        }
        let merge = if input.kind(commit, "type")? == 3 && kind != 3 {
            " pele-commit-merge"
        } else {
            ""
        };
        commits += &format!(
            "<g class=\"pele-commit pele-commit-{}{merge}\" data-id=\"{}\">{body}</g>",
            CLASSES.get(kind as usize).unwrap_or(&CLASSES[0]),
            esc_text(&input.field_text(commit, "id")?.unwrap_or_default())?
        )
    }
    if bounds.min_x > bounds.max_x {
        bounds.grow(0., 0., 0., 0.)
    }
    let title = input.field_text(&root, "title")?;
    input.keep_label(title.as_ref().map_or(0, |s| s.encode_utf16().count()))?;
    let title = layout_label(title.as_deref(), false, resources, options, size, 4000., 1)?;
    let title_height = if title.height > 0. {
        title.height + 16.
    } else {
        0.
    };
    let inner = (bounds.max_x - bounds.min_x).js_max(title.width);
    let width = (inner + 2. * pad).ceil();
    let height = (bounds.max_y - bounds.min_y + title_height + 2. * pad).ceil();
    let dx = pad - bounds.min_x + (inner - (bounds.max_x - bounds.min_x)) / 2.;
    let dy = pad + title_height - bounds.min_y;
    let mut content = label_svg(
        &title,
        width / 2.,
        pad + title.height / 2.,
        " class=\"pele-title\" font-weight=\"var(--_tw)\"",
    );
    content += &format!("<g transform=\"translate({},{})\">", num(dx), num(dy));
    if !branches.is_empty() {
        content += &format!(
            "<g class=\"pele-branches\" fill=\"none\" font-size=\"{}\">{branches}</g>",
            svg::js_num(medium)
        )
    }
    if !curves.is_empty() || !lines.is_empty() {
        content+=&format!("<g class=\"pele-edges\" fill=\"none\" stroke-width=\"2\" stroke-linecap=\"round\">{curves}{lines}</g>")
    }
    if !commits.is_empty() {
        content += &format!(
            "<g class=\"pele-commits\" font-size=\"{}\" fill=\"var(--_m)\">{commits}</g>",
            svg::js_num(small)
        )
    }
    content += "</g>";
    Ok(Rendered {
        svg: svg::document(
            "gitGraph",
            width,
            height,
            size,
            options,
            input.field_text(&root, "accTitle")?.as_deref(),
            input.field_text(&root, "accDescr")?.as_deref(),
            &content,
        )?,
        width,
        height,
        links: Vec::new(),
    })
}
