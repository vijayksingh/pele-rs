//! Native-selected C4 field phases; immutable and live inputs share geometry.
use super::model::{C4Element, C4Model, ElementRef};
use crate::compat::PeleError;
#[derive(Clone, Copy)]
pub(super) enum Record {
    Root,
    Element(ElementRef),
    Relation(usize),
}
#[derive(Clone, Copy)]
pub(super) enum Collection {
    Boundaries,
    Shapes,
    Relations,
}
pub(super) trait PhaseInput {
    type Error: From<PeleError>;
    type Record: Clone;
    type List;
    fn begin_draw(&mut self) {}
    fn present(&mut self, _: &Self::List, _: usize) -> Result<bool, Self::Error> {
        Ok(true)
    }
    fn root(&self) -> Self::Record;
    fn list(&mut self, name: Collection) -> Result<Self::List, Self::Error>;
    fn len(&mut self, list: &Self::List) -> Result<usize, Self::Error>;
    fn at(&mut self, list: &Self::List, index: usize) -> Result<Self::Record, Self::Error>;
    fn identity(&mut self, record: &Self::Record) -> Result<ElementRef, Self::Error>;
    fn parent(&mut self, record: &Self::Record) -> Result<Option<ElementRef>, Self::Error>;
    fn text(
        &mut self,
        record: &Self::Record,
        key: &'static str,
    ) -> Result<Option<String>, Self::Error>;
    fn flag(&mut self, record: &Self::Record, key: &'static str) -> Result<bool, Self::Error>;
    fn number(
        &mut self,
        record: &Self::Record,
        key: &'static str,
    ) -> Result<Option<f64>, Self::Error>;
    fn lookup(&mut self, alias: &str) -> Result<Option<ElementRef>, Self::Error>;
    fn retain_label(&mut self, _: Option<&str>) -> Result<(), Self::Error> {
        Ok(())
    }
    fn step(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn missing_helper(&mut self) -> Result<(), Self::Error>;
}
pub(super) struct Typed<'a> {
    pub model: &'a C4Model,
    pub shape_lookup: bool,
}
impl Typed<'_> {
    fn element(&self, record: &Record) -> Option<&C4Element> {
        if let Record::Element(id) = record {
            Some(self.model.element(*id))
        } else {
            None
        }
    }
}
impl PhaseInput for Typed<'_> {
    type Error = PeleError;
    type Record = Record;
    type List = Collection;
    fn root(&self) -> Record {
        Record::Root
    }
    fn list(&mut self, name: Collection) -> Result<Collection, PeleError> {
        Ok(name)
    }
    fn len(&mut self, list: &Collection) -> Result<usize, PeleError> {
        Ok(match list {
            Collection::Boundaries => self.model.boundaries.len(),
            Collection::Shapes => self.model.shapes.len(),
            Collection::Relations => self.model.rels.len(),
        })
    }
    fn at(&mut self, list: &Collection, index: usize) -> Result<Record, PeleError> {
        Ok(match list {
            Collection::Boundaries => Record::Element(ElementRef::Boundary(index)),
            Collection::Shapes => Record::Element(ElementRef::Shape(index)),
            Collection::Relations => Record::Relation(index),
        })
    }
    fn identity(&mut self, record: &Record) -> Result<ElementRef, PeleError> {
        let Record::Element(id) = record else {
            unreachable!()
        };
        Ok(self.model.identity(*id))
    }
    fn parent(&mut self, record: &Record) -> Result<Option<ElementRef>, PeleError> {
        Ok(self
            .element(record)
            .and_then(|e| e.parent)
            .map(|id| self.model.identity(ElementRef::Boundary(id))))
    }
    fn text(&mut self, record: &Record, key: &'static str) -> Result<Option<String>, PeleError> {
        let value = match record {
            Record::Root => match key {
                "title" => self.model.title.clone(),
                "accTitle" => self.model.acc_title.clone(),
                "accDescr" => self.model.acc_descr.clone(),
                "c4Type" => self.model.c4_type.clone(),
                _ => None,
            },
            Record::Relation(i) => {
                let r = &self.model.rels[*i];
                match key {
                    "from" => Some(r.from.clone()),
                    "to" => Some(r.to.clone()),
                    "label" => Some(r.label.clone()),
                    "type" => Some(r.rel_type.clone()),
                    "techn" => r.techn.clone(),
                    "descr" => r.descr.clone(),
                    "lineColor" => r.line_color.clone(),
                    "textColor" => r.text_color.clone(),
                    "link" => r.link.clone(),
                    _ => None,
                }
            }
            Record::Element(id) => {
                let e = self.model.element(*id);
                match key {
                    "alias" => Some(e.alias.clone()),
                    "label" => Some(e.label.clone()),
                    "type" => e.element_type.clone(),
                    "descr" => e.descr.clone(),
                    "techn" => e.techn.clone(),
                    "sprite" => e.sprite.clone(),
                    "tags" => e.tags.clone(),
                    "link" => e.link.clone(),
                    "bgColor" => e.bg_color.clone(),
                    "fontColor" => e.font_color.clone(),
                    "borderColor" => e.border_color.clone(),
                    "shape" => e.shape.clone(),
                    "kind" => {
                        if let ElementRef::Shape(i) = id {
                            Some(self.model.shapes[*i].kind.clone())
                        } else {
                            None
                        }
                    }
                    "nodeType" => {
                        if let ElementRef::Boundary(i) = id {
                            self.model.boundaries[*i].node_type.clone()
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
        };
        Ok(value)
    }
    fn flag(&mut self, record: &Record, key: &'static str) -> Result<bool, PeleError> {
        if let Record::Relation(i) = record {
            let r = &self.model.rels[*i];
            if key == "fromAttr" {
                return Ok(r.from_attr.is_some());
            }
            if key == "toAttr" {
                return Ok(r.to_attr.is_some());
            }
        }
        Ok(self.text(record, key)?.is_some_and(|s| !s.is_empty()))
    }
    fn number(&mut self, record: &Record, key: &'static str) -> Result<Option<f64>, PeleError> {
        Ok(match record {
            Record::Root => match key {
                "shapeInRow" => Some(self.model.shape_in_row),
                "boundaryInRow" => Some(self.model.boundary_in_row),
                _ => None,
            },
            Record::Relation(i) => match key {
                "offsetX" => self.model.rels[*i].offset_x,
                "offsetY" => self.model.rels[*i].offset_y,
                _ => None,
            },
            _ => None,
        })
    }
    fn lookup(&mut self, alias: &str) -> Result<Option<ElementRef>, PeleError> {
        self.missing_helper()?;
        Ok(self.model.find(alias))
    }
    fn missing_helper(&mut self) -> Result<(), PeleError> {
        if self.shape_lookup {
            Ok(())
        } else {
            Err(PeleError::new(
                "t.getC4Shape is not a function",
                crate::compat::ErrorCode::Semantic,
            ))
        }
    }
}
