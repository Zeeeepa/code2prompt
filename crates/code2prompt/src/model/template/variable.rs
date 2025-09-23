//! Template variable state management.
//!
//! This module contains the state and logic for managing template variables,
//! including system variables, user-defined variables, and missing variables.

use std::collections::HashMap;

/// Variable categories for display and management
#[derive(Debug, Clone, PartialEq)]
pub enum VariableCategory {
    System,  // From build_template_data
    User,    // User-defined
    Missing, // In template but not defined
}

/// Information about a template variable
#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub name: String,
    pub value: Option<String>,
    pub category: VariableCategory,
    pub description: Option<String>,
}

/// State for the template variable component
#[derive(Debug, Clone)]
pub struct VariableState {
    pub system_variables: HashMap<String, String>, // System variables with descriptions
    pub user_variables: HashMap<String, String>,   // User-defined variables
    pub missing_variables: Vec<String>,            // Variables in template but not defined
    pub cursor: usize,                             // Current cursor position in variable list
    pub editing_variable: Option<String>,          // Currently editing variable name
    pub variable_input_content: String,            // Content being typed for variable
    pub show_variable_input: bool,                 // Show variable input dialog
}

impl Default for VariableState {
    fn default() -> Self {
        Self {
            system_variables: Self::get_default_system_variables(),
            user_variables: HashMap::new(),
            missing_variables: Vec::new(),
            cursor: 0,
            editing_variable: None,
            variable_input_content: String::new(),
            show_variable_input: false,
        }
    }
}

impl VariableState {
    /// Get default system variables that are available from build_template_data
    fn get_default_system_variables() -> HashMap<String, String> {
        let mut vars = HashMap::new();

        // Main template variables from build_template_data()
        vars.insert(
            "absolute_code_path".to_string(),
            "Path to the codebase directory".to_string(),
        );
        vars.insert(
            "source_tree".to_string(),
            "Directory tree structure".to_string(),
        );
        vars.insert(
            "files".to_string(),
            "Array of file objects with content".to_string(),
        );
        vars.insert(
            "git_diff".to_string(),
            "Git diff output (if enabled)".to_string(),
        );
        vars.insert(
            "git_diff_branch".to_string(),
            "Git diff between branches".to_string(),
        );
        vars.insert(
            "git_log_branch".to_string(),
            "Git log between branches".to_string(),
        );

        // File object properties (used within {{#each files}} loops)
        vars.insert(
            "path".to_string(),
            "File path (available in {{#each files}} context)".to_string(),
        );
        vars.insert(
            "code".to_string(),
            "File content (available in {{#each files}} context)".to_string(),
        );
        vars.insert(
            "extension".to_string(),
            "File extension (available in {{#each files}} context)".to_string(),
        );
        vars.insert(
            "token_count".to_string(),
            "Token count for file (available in {{#each files}} context)".to_string(),
        );
        vars.insert(
            "metadata".to_string(),
            "File metadata (available in {{#each files}} context)".to_string(),
        );
        vars.insert(
            "mod_time".to_string(),
            "File modification time (available in {{#each files}} context)".to_string(),
        );

        vars
    }

    /// Update missing variables based on template variables
    pub fn update_missing_variables(&mut self, template_variables: &[String]) {
        self.missing_variables.clear();

        for var in template_variables {
            if !self.system_variables.contains_key(var) && !self.user_variables.contains_key(var) {
                self.missing_variables.push(var.clone());
            }
        }

        self.missing_variables.sort();
    }

    /// Get all variables organized by category for display
    pub fn get_organized_variables(&self, template_variables: &[String]) -> Vec<VariableInfo> {
        let mut variables = Vec::new();

        // System variables (only those used in template)
        for var in template_variables {
            if let Some(desc) = self.system_variables.get(var) {
                variables.push(VariableInfo {
                    name: var.clone(),
                    value: Some("(system)".to_string()),
                    category: VariableCategory::System,
                    description: Some(desc.clone()),
                });
            }
        }

        // User variables (only those used in template)
        for var in template_variables {
            if let Some(value) = self.user_variables.get(var) {
                variables.push(VariableInfo {
                    name: var.clone(),
                    value: Some(value.clone()),
                    category: VariableCategory::User,
                    description: None,
                });
            }
        }

        // Missing variables
        for var in &self.missing_variables {
            variables.push(VariableInfo {
                name: var.clone(),
                value: None,
                category: VariableCategory::Missing,
                description: Some("⚠️ Not defined".to_string()),
            });
        }

        variables
    }

    /// Set a user variable
    pub fn set_user_variable(&mut self, key: String, value: String) {
        self.user_variables.insert(key, value);
    }

    /// Check if there are missing variables
    pub fn has_missing_variables(&self) -> bool {
        !self.missing_variables.is_empty()
    }

    /// Cancel variable editing
    pub fn cancel_editing(&mut self) {
        self.editing_variable = None;
        self.variable_input_content.clear();
        self.show_variable_input = false;
    }

    /// Finish editing variable and save
    pub fn finish_editing(&mut self) -> Option<(String, String)> {
        if let Some(var_name) = self.editing_variable.take() {
            let value = self.variable_input_content.clone();
            self.set_user_variable(var_name.clone(), value.clone());
            self.variable_input_content.clear();
            self.show_variable_input = false;
            Some((var_name, value))
        } else {
            None
        }
    }

    /// Add character to variable input
    pub fn add_char_to_input(&mut self, c: char) {
        self.variable_input_content.push(c);
    }

    /// Remove character from variable input
    pub fn remove_char_from_input(&mut self) {
        self.variable_input_content.pop();
    }

    /// Get current variable input content
    pub fn get_input_content(&self) -> &str {
        &self.variable_input_content
    }

    /// Check if currently editing a variable
    pub fn is_editing(&self) -> bool {
        self.show_variable_input
    }

    /// Get currently editing variable name
    pub fn get_editing_variable(&self) -> Option<&String> {
        self.editing_variable.as_ref()
    }

    /// Move cursor to first missing/user-defined variable
    pub fn move_to_first_missing_variable(&mut self) {
        // This will be called when entering variable editing mode
        // For now, just reset cursor to 0, but we could enhance this
        // to find the first missing variable in the organized list
        self.cursor = 0;
    }
}
