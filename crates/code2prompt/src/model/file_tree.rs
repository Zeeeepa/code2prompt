//! File tree state management for the TUI application.
//!
//! This module contains the file tree state, FileNode structure,
//! and related functionality for managing the file selection interface.

use std::path::PathBuf;

/// File tree state containing all file tree related data
#[derive(Default, Debug, Clone)]
pub struct FileTreeState {
    pub file_tree: Vec<FileNode>,
    pub search_query: String,
    pub tree_cursor: usize,
    pub file_tree_scroll: u16,
}

/// File tree node with selection state
#[derive(Debug, Clone)]
pub struct FileNode {
    pub path: PathBuf,
    pub name: String,
    pub is_directory: bool,
    pub is_expanded: bool,
    pub is_selected: bool,
    pub children: Vec<FileNode>,
    pub level: usize,
    pub children_loaded: bool,
}

impl FileTreeState {
    /// Get flattened list of visible file nodes for display
    pub fn get_visible_nodes(&self) -> Vec<&FileNode> {
        let mut visible = Vec::new();
        self.collect_visible_nodes(&self.file_tree, &mut visible);
        visible
    }

    /// Set the file tree
    pub fn set_file_tree(&mut self, tree: Vec<FileNode>) {
        self.file_tree = tree;
    }

    /// Update node selection state - pure logic in Model
    pub fn update_node_selection(
        &mut self,
        path: &std::path::Path,
        selected: bool,
        is_directory: bool,
    ) {
        let path_str = path.to_string_lossy();
        if is_directory {
            Self::toggle_directory_selection_recursive(&mut self.file_tree, &path_str, selected);
        } else {
            Self::update_single_node_selection(&mut self.file_tree, &path_str, selected);
        }
    }

    /// Expand directory at given path
    pub fn expand_directory(&mut self, path: &std::path::Path) {
        let path_str = path.to_string_lossy();
        Self::set_directory_expanded(&mut self.file_tree, &path_str, true);
    }

    /// Collapse directory at given path
    pub fn collapse_directory(&mut self, path: &std::path::Path) {
        let path_str = path.to_string_lossy();
        Self::set_directory_expanded(&mut self.file_tree, &path_str, false);
    }

    /// Load children for a directory if not already loaded
    pub fn load_directory_children(
        &mut self,
        path: &std::path::Path,
    ) -> Result<(), std::io::Error> {
        let path_str = path.to_string_lossy();
        Self::load_children_for_path(&mut self.file_tree, &path_str)
    }

    fn toggle_directory_selection_recursive(nodes: &mut [FileNode], path: &str, selected: bool) {
        for node in nodes {
            if node.path.to_string_lossy() == path {
                node.is_selected = selected;
                // Also update all children
                Self::set_children_selection(&mut node.children, selected);
                return;
            }
            if !node.children.is_empty() {
                Self::toggle_directory_selection_recursive(&mut node.children, path, selected);
            }
        }
    }

    fn update_single_node_selection(nodes: &mut [FileNode], path: &str, selected: bool) {
        for node in nodes {
            if node.path.to_string_lossy() == path {
                node.is_selected = selected;
                return;
            }
            if !node.children.is_empty() {
                Self::update_single_node_selection(&mut node.children, path, selected);
            }
        }
    }

    fn set_directory_expanded(nodes: &mut [FileNode], path: &str, expanded: bool) {
        for node in nodes {
            if node.path.to_string_lossy() == path && node.is_directory {
                node.is_expanded = expanded;
                return;
            }
            if !node.children.is_empty() {
                Self::set_directory_expanded(&mut node.children, path, expanded);
            }
        }
    }

    fn set_children_selection(nodes: &mut [FileNode], selected: bool) {
        for node in nodes {
            node.is_selected = selected;
            if !node.children.is_empty() {
                Self::set_children_selection(&mut node.children, selected);
            }
        }
    }

    fn collect_visible_nodes<'a>(&'a self, nodes: &'a [FileNode], visible: &mut Vec<&'a FileNode>) {
        for node in nodes {
            let matches_search = if self.search_query.is_empty() {
                true
            } else if self.search_query.contains('*') || self.search_query.contains("**") {
                // Glob pattern search
                self.glob_match_search(&self.search_query, &node.name)
                    || self.glob_match_search(&self.search_query, &node.path.to_string_lossy())
            } else {
                // Simple text search
                node.name
                    .to_lowercase()
                    .contains(&self.search_query.to_lowercase())
                    || node
                        .path
                        .to_string_lossy()
                        .to_lowercase()
                        .contains(&self.search_query.to_lowercase())
            };

            if matches_search {
                visible.push(node);
            }

            // Add children if expanded and node matches search or has matching children
            if node.is_expanded && (matches_search || node.is_directory) {
                self.collect_visible_nodes(&node.children, visible);
            }
        }
    }

    /// Simple glob matching for search (similar to utils but accessible from model)
    fn glob_match_search(&self, pattern: &str, text: &str) -> bool {
        // Handle ** for recursive directory matching
        if pattern.contains("**") {
            let parts: Vec<&str> = pattern.split("**").collect();
            if parts.len() == 2 {
                let prefix = parts[0].trim_end_matches('/');
                let suffix = parts[1].trim_start_matches('/');

                if prefix.is_empty() && suffix.is_empty() {
                    return true; // "**" matches everything
                }

                let prefix_match = prefix.is_empty() || text.contains(prefix);
                let suffix_match = suffix.is_empty() || text.contains(suffix);

                return prefix_match && suffix_match;
            }
        }

        // Handle single * wildcard
        if pattern.contains('*') && !pattern.contains("**") {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                return text.contains(parts[0]) && text.contains(parts[1]);
            }
        }

        // Fallback to contains
        text.to_lowercase().contains(&pattern.to_lowercase())
    }

    /// Load children for a specific directory path
    fn load_children_for_path(nodes: &mut [FileNode], path: &str) -> Result<(), std::io::Error> {
        for node in nodes {
            if node.path.to_string_lossy() == path && node.is_directory && !node.children_loaded {
                // Load children from filesystem
                let entries = std::fs::read_dir(&node.path)?;
                let mut children = Vec::new();

                for entry in entries {
                    let entry = entry?;
                    let child_path = entry.path();
                    let child_node = FileNode::new(child_path, node.level + 1);
                    children.push(child_node);
                }

                // Sort children (directories first, then alphabetically)
                children.sort_by(|a, b| match (a.is_directory, b.is_directory) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.name.cmp(&b.name),
                });

                node.children = children;
                node.children_loaded = true;
                return Ok(());
            }
            if !node.children.is_empty() {
                Self::load_children_for_path(&mut node.children, path)?;
            }
        }
        Ok(())
    }
}

impl FileNode {
    pub fn new(path: PathBuf, level: usize) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        let is_directory = path.is_dir();

        Self {
            path,
            name,
            is_directory,
            is_expanded: false,
            is_selected: false,
            children: Vec::new(),
            level,
            children_loaded: false,
        }
    }
}
