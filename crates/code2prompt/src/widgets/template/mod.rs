//! Template widget module.
//!
//! This module coordinates the three template sub-widgets:
//! - Editor: Template content editing and validation
//! - Variable: Variable management and validation  
//! - Picker: Template selection and loading

pub mod editor;
pub mod picker;
pub mod variable;

pub use editor::TemplateEditorWidget;
pub use picker::TemplatePickerWidget;
pub use variable::TemplateVariableWidget;

use crate::model::template::{TemplateFocus, TemplateState};
use crate::model::Model;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

/// Main Template widget that coordinates the 3 sub-widgets
pub struct TemplateWidget {
    editor: TemplateEditorWidget,
    variables: TemplateVariableWidget,
    picker: TemplatePickerWidget,
}

impl TemplateWidget {
    pub fn new(_model: &Model) -> Self {
        Self {
            editor: TemplateEditorWidget::new(),
            variables: TemplateVariableWidget::new(),
            picker: TemplatePickerWidget::new(),
        }
    }

    /// Render the template widget with 3 columns
    pub fn render(&self, area: Rect, buf: &mut Buffer, state: &mut TemplateState) {
        // Main layout - content and footer
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // Content (3 columns)
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // 3-column layout for content
        self.render_content(chunks[0], buf, state);

        // Footer
        self.render_footer(chunks[1], buf, state);
    }

    /// Render the 3-column content area
    fn render_content(&self, area: Rect, buf: &mut Buffer, state: &mut TemplateState) {
        // Flexible 3-column layout
        let min_width = 30;
        let available_width = area.width.saturating_sub(6); // Account for borders

        let constraints = if available_width >= min_width * 3 {
            // Full 3-column layout
            vec![
                Constraint::Percentage(40), // Editor
                Constraint::Percentage(35), // Variables
                Constraint::Percentage(25), // Picker
            ]
        } else if available_width >= min_width * 2 {
            // 2-column layout, hide picker or make it smaller
            vec![
                Constraint::Percentage(60), // Editor
                Constraint::Percentage(40), // Variables
                Constraint::Length(0),      // Picker hidden
            ]
        } else {
            // Single column, show only focused column
            match state.get_focus() {
                TemplateFocus::Editor => vec![
                    Constraint::Percentage(100),
                    Constraint::Length(0),
                    Constraint::Length(0),
                ],
                TemplateFocus::Variables => vec![
                    Constraint::Length(0),
                    Constraint::Percentage(100),
                    Constraint::Length(0),
                ],
                TemplateFocus::Picker => vec![
                    Constraint::Length(0),
                    Constraint::Length(0),
                    Constraint::Percentage(100),
                ],
            }
        };

        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);

        // Render each column if it has space
        if columns[0].width > 0 {
            let is_editor_focused = state.get_focus() == TemplateFocus::Editor;
            let is_editing_template =
                state.get_focus_mode() == crate::model::template::FocusMode::EditingTemplate;
            let has_missing_vars = state.variables.has_missing_variables();
            self.editor.render(
                columns[0],
                buf,
                &mut state.editor,
                is_editor_focused || is_editing_template,
                has_missing_vars,
            );
        }

        if columns[1].width > 0 {
            let variables = state.get_organized_variables();
            let is_variables_focused = state.get_focus() == TemplateFocus::Variables;
            let is_editing_variable =
                state.get_focus_mode() == crate::model::template::FocusMode::EditingVariable;
            self.variables.render(
                columns[1],
                buf,
                &state.variables,
                &variables,
                is_variables_focused || is_editing_variable,
            );
        }

        if columns[2].width > 0 {
            self.picker.render(
                columns[2],
                buf,
                &state.picker,
                state.get_focus() == TemplateFocus::Picker,
            );
        }
    }

    /// Render the footer with controls and status
    fn render_footer(&self, area: Rect, buf: &mut Buffer, state: &TemplateState) {
        let footer_content = if !state.get_status().is_empty() {
            // Simple text for status messages
            vec![Span::styled(
                state.get_status(),
                Style::default().fg(Color::Gray),
            )]
        } else {
            // Show different controls based on focus mode
            match state.get_focus_mode() {
                crate::model::template::FocusMode::Normal => {
                    // Normal mode: can switch focus with colored letters
                    let mut spans = vec![
                        Span::styled(
                            "Enter: Run Analysis | Focus: ",
                            Style::default().fg(Color::Gray),
                        ),
                        Span::styled(
                            "e",
                            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled("(dit) ", Style::default().fg(Color::Gray)),
                        Span::styled(
                            "v",
                            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled("(ariables) ", Style::default().fg(Color::Gray)),
                        Span::styled(
                            "p",
                            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled("(icker) | ", Style::default().fg(Color::Gray)),
                    ];

                    let specific_controls = match state.get_focus() {
                        TemplateFocus::Editor => "Press 'e' to enter edit mode".to_string(),
                        TemplateFocus::Variables => "Press 'v' to enter variable mode".to_string(),
                        TemplateFocus::Picker => {
                            TemplatePickerWidget::get_help_text(true, state.picker.active_list)
                        }
                    };

                    spans.push(Span::styled(
                        specific_controls,
                        Style::default().fg(Color::Gray),
                    ));
                    spans
                }
                crate::model::template::FocusMode::EditingTemplate => {
                    vec![Span::styled(
                        "EDIT MODE: Type to edit template | ESC: Exit edit mode",
                        Style::default().fg(Color::Gray),
                    )]
                }
                crate::model::template::FocusMode::EditingVariable => {
                    let text = if state.variables.is_editing() {
                        "VARIABLE INPUT: Type value | Enter: Save | ESC: Cancel"
                    } else {
                        "VARIABLE MODE: ↑↓: Navigate | Space: Edit variable | Tab: Next | ESC: Exit"
                    };
                    vec![Span::styled(text, Style::default().fg(Color::Gray))]
                }
            }
        };

        let footer = Paragraph::new(Line::from(footer_content))
            .block(Block::default().borders(Borders::ALL).title("Controls"));
        footer.render(area, buf);
    }
}

impl StatefulWidget for TemplateWidget {
    type State = TemplateState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        TemplateWidget::render(&self, area, buf, state);
    }
}
