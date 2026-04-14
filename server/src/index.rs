// In-memory index rebuilt from .jsonl on startup.
// Pattern from questhub-kidur-pathb.md handover doc.
// Same architecture as Obsidian: files are truth, index lives in memory.

use std::collections::HashMap;
use kidur_core::{Edge, Node, NodeId};
use kidur_log::{Mutation, MutationLog};

pub struct Index {
    pub nodes: HashMap<NodeId, Node>,
    pub children: HashMap<NodeId, Vec<NodeId>>,
    pub by_supertag: HashMap<String, Vec<NodeId>>,
    pub edges: Vec<Edge>,
}

impl Index {
    /// Rebuild from a .jsonl log file.
    pub fn from_log(path: &std::path::Path) -> kidur_core::KidurResult<Self> {
        let entries = MutationLog::replay(path)?;
        let mut index = Index {
            nodes: HashMap::new(),
            children: HashMap::new(),
            by_supertag: HashMap::new(),
            edges: Vec::new(),
        };

        for entry in entries {
            index.apply_mutation(entry.mutation);
        }

        Ok(index)
    }

    /// Apply a single mutation to the in-memory index.
    pub fn apply_mutation(&mut self, mutation: Mutation) {
        match mutation {
            Mutation::CreateNode { node } | Mutation::UpdateNode { node } => {
                let id = node.id;
                if let Some(pid) = node.parent_id {
                    let children = self.children.entry(pid).or_default();
                    if !children.contains(&id) {
                        children.push(id);
                    }
                }
                if let Some(ref tag) = node.supertag {
                    let ids = self.by_supertag.entry(tag.clone()).or_default();
                    if !ids.contains(&id) {
                        ids.push(id);
                    }
                }
                self.nodes.insert(id, node);
            }
            Mutation::DeleteNode { id } => {
                if let Some(node) = self.nodes.remove(&id) {
                    if let Some(pid) = node.parent_id {
                        if let Some(children) = self.children.get_mut(&pid) {
                            children.retain(|c| *c != id);
                        }
                    }
                    if let Some(ref tag) = node.supertag {
                        if let Some(ids) = self.by_supertag.get_mut(tag) {
                            ids.retain(|i| *i != id);
                        }
                    }
                }
            }
            Mutation::CreateEdge { edge } => {
                self.edges.push(edge);
            }
            Mutation::DeleteEdge {
                from_id,
                to_id,
                kind,
            } => {
                self.edges
                    .retain(|e| !(e.from_id == from_id && e.to_id == to_id && e.kind == kind));
            }
        }
    }

    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }

    pub fn get_children(&self, parent_id: NodeId) -> Vec<&Node> {
        self.children
            .get(&parent_id)
            .map(|ids| ids.iter().filter_map(|id| self.nodes.get(id)).collect())
            .unwrap_or_default()
    }

    pub fn list_by_supertag(&self, tag: &str) -> Vec<&Node> {
        self.by_supertag
            .get(tag)
            .map(|ids| ids.iter().filter_map(|id| self.nodes.get(id)).collect())
            .unwrap_or_default()
    }

    pub fn quest_count(&self) -> usize {
        self.by_supertag
            .get("quest")
            .map(|ids| ids.len())
            .unwrap_or(0)
    }

    /// Iterator over all quest nodes (all nodes with supertag == "quest").
    pub fn all_quest_nodes(&self) -> impl Iterator<Item = &Node> + '_ {
        self.by_supertag
            .get("quest")
            .into_iter()
            .flat_map(|ids| ids.iter().filter_map(|id| self.nodes.get(id)))
    }
}
