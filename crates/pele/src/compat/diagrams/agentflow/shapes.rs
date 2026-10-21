//! Pele agentflow/shapes.ts constrained shapes and semantic aliases, 458322a5 (MIT).
use super::model::DiagnosticId;
pub(super) fn resolve(s: &str) -> &str {
    match s {
        "task" => "roundedRect",
        "tool" => "subroutine",
        "input" => "lean-right",
        "decision" => "diamond",
        "refdoc" => "lin-doc",
        "action" => "hexagon",
        "round" => "rect",
        _ => s,
    }
}
pub(super) fn valid(s: &str) -> bool {
    super::super::flowchart::shapes::valid(s) || s == "constructor"
}
pub(super) fn canonical(s: Option<&str>) -> &str {
    super::super::flowchart::shapes::canonical(s)
}
pub(super) fn normalize_with<E>(
    nodes: &mut [super::graph::GraphNode],
    mut emit: impl FnMut(DiagnosticId, &str, String, &str) -> Result<(), E>,
) -> Result<(), E> {
    const REMOVED: &[&str] = &[
        "doc",
        "stadium",
        "terminal",
        "circle",
        "trapezoid",
        "inv_trapezoid",
        "inv-trapezoid",
        "doublecircle",
        "double-circle",
        "typeDeclaration",
        "procs",
        "lean_left",
        "lean-left",
        "in-out",
        "cylinder",
        "ellipse",
        "odd",
        "tag-rect",
        "tagged-rectangle",
        "delay",
        "half-rounded-rectangle",
        "lin-rect",
        "lined-rectangle",
        "win-pane",
        "window-pane",
        "curv-trap",
        "curved-trapezoid",
    ];
    const ALLOWED: &[&str] = &[
        "roundedRect",
        "subroutine",
        "subprocess",
        "subproc",
        "framed-rectangle",
        "lean-right",
        "diamond",
        "lin-doc",
        "lined-document",
        "hexagon",
        "hex",
        "connector",
        "collapsedGroup",
    ];
    for node in nodes {
        if node.is_group {
            continue;
        }
        if matches!(node.shape.as_str(), "" | "squareRect" | "rect") {
            node.shape = "roundedRect".into();
        }
        if REMOVED.contains(&node.shape.as_str()) {
            emit(
                DiagnosticId::ShapeRemoved,
                "error",
                format!(
                    "shape \"{}\" was removed in v0.8.1, using \"roundedRect\"",
                    node.shape
                ),
                &node.id,
            )?;
            node.shape = "roundedRect".into();
        } else if !ALLOWED.contains(&node.shape.as_str()) {
            emit(
                DiagnosticId::ShapeUnsupported,
                "warning",
                format!(
                    "shape \"{}\" is not supported, using \"roundedRect\"",
                    node.shape
                ),
                &node.id,
            )?;
            node.shape = "roundedRect".into();
        }
    }
    Ok(())
}
pub(super) fn kind_slot(s: &str) -> Option<usize> {
    match s {
        "tool" => Some(0),
        "task" => Some(1),
        "decision" => Some(2),
        "input" => Some(3),
        "refdoc" => Some(4),
        "connector" => Some(5),
        "action" => Some(6),
        _ => None,
    }
}
