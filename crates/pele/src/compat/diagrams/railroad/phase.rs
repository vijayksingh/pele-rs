//! Bounded grammar traversal feeds one shared box algebra and painter.
use super::model::{RailroadModel, RailroadNode};
use crate::compat::{ErrorCode, PeleError};
#[derive(Clone, Copy)]
pub(crate) enum NodeType {
    Terminal,
    Nonterminal,
    Special,
    Sequence,
    Choice,
    Optional,
    Repetition,
}
pub(crate) trait PhaseInput {
    type Error: From<PeleError>;
    type Node: Clone;
    type Children;
    type Rules;
    type Rule;
    fn begin_draw(&mut self);
    fn identity(&mut self, node: &Self::Node) -> Result<u64, Self::Error>;
    fn children(&mut self, node: &Self::Node) -> Result<Self::Children, Self::Error>;
    fn length(&mut self, children: &Self::Children) -> Result<usize, Self::Error>;
    fn at(&mut self, children: &Self::Children, index: usize) -> Result<Self::Node, Self::Error>;
    fn kind(&mut self, node: &Self::Node) -> Result<NodeType, Self::Error>;
    fn text(&mut self, node: &Self::Node, field: &'static str) -> Result<String, Self::Error>;
    fn repetition(&mut self, node: &Self::Node) -> Result<(f64, f64), Self::Error>;
    fn rules(&mut self) -> Result<Self::Rules, Self::Error>;
    fn next_rule(
        &mut self,
        rules: &Self::Rules,
        index: usize,
    ) -> Result<Option<Self::Rule>, Self::Error>;
    fn definition(&mut self, rule: &Self::Rule) -> Result<Self::Node, Self::Error>;
    fn rule_name(&mut self, rule: &Self::Rule) -> Result<String, Self::Error>;
    fn metadata(&mut self, key: &'static str) -> Result<Option<String>, Self::Error>;
}
pub(crate) struct TypedPhase<'a>(pub &'a RailroadModel);
impl PhaseInput for TypedPhase<'_> {
    type Error = PeleError;
    type Node = usize;
    type Children = Vec<usize>;
    type Rules = ();
    type Rule = usize;
    fn begin_draw(&mut self) {}
    fn identity(&mut self, node: &usize) -> Result<u64, PeleError> {
        Ok(*node as u64)
    }
    fn children(&mut self, node: &usize) -> Result<Vec<usize>, PeleError> {
        Ok(self.0.nodes[*node].children().to_vec())
    }
    fn length(&mut self, children: &Vec<usize>) -> Result<usize, PeleError> {
        Ok(children.len())
    }
    fn at(&mut self, children: &Vec<usize>, index: usize) -> Result<usize, PeleError> {
        Ok(children[index])
    }
    fn kind(&mut self, node: &usize) -> Result<NodeType, PeleError> {
        Ok(match self.0.nodes[*node] {
            RailroadNode::Terminal(_) => NodeType::Terminal,
            RailroadNode::Nonterminal(_) => NodeType::Nonterminal,
            RailroadNode::Special(_) => NodeType::Special,
            RailroadNode::Sequence(_) => NodeType::Sequence,
            RailroadNode::Choice(_) => NodeType::Choice,
            RailroadNode::Optional(_) => NodeType::Optional,
            RailroadNode::Repetition { .. } => NodeType::Repetition,
        })
    }
    fn text(&mut self, node: &usize, _: &'static str) -> Result<String, PeleError> {
        match &self.0.nodes[*node] {
            RailroadNode::Terminal(s) | RailroadNode::Nonterminal(s) | RailroadNode::Special(s) => {
                Ok(s.clone())
            }
            _ => Err(PeleError::new(
                "Railroad node has no text field",
                ErrorCode::Semantic,
            )),
        }
    }
    fn repetition(&mut self, node: &usize) -> Result<(f64, f64), PeleError> {
        match self.0.nodes[*node] {
            RailroadNode::Repetition { min, max, .. } => Ok((min, max)),
            _ => unreachable!(),
        }
    }
    fn rules(&mut self) -> Result<(), PeleError> {
        Ok(())
    }
    fn next_rule(&mut self, _: &(), index: usize) -> Result<Option<usize>, PeleError> {
        Ok((index < self.0.rules.len()).then_some(index))
    }
    fn definition(&mut self, rule: &usize) -> Result<usize, PeleError> {
        Ok(self.0.rules[*rule].definition)
    }
    fn rule_name(&mut self, rule: &usize) -> Result<String, PeleError> {
        Ok(self.0.rules[*rule].name.clone())
    }
    fn metadata(&mut self, key: &'static str) -> Result<Option<String>, PeleError> {
        Ok(match key {
            "title" => self.0.title.clone(),
            "accTitle" => self.0.acc_title.clone(),
            _ => self.0.acc_descr.clone(),
        })
    }
}
