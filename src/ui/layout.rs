use ratatui::layout::Rect;

/// Struct for holding information on UI areas of app
#[derive(Debug, Clone, Copy)]
pub struct UILayout {
    pub status_bar_area: Option<Rect>,
    pub editor_area: Rect, // Full editor area (includes line numbers)
    pub line_number_area: Option<Rect>, // Optional line numbers on left
    pub content_area: Rect, // Actual text content area
    pub command_line_area: Rect,
}

impl UILayout {
    pub fn default(area: Rect) -> Self {
        Self {
            status_bar_area: Some(Rect {
                x: 0,
                y: 0,
                width: area.width,
                height: 1,
            }),
            editor_area: Rect {
                x: 0,
                y: 1,
                width: area.width,
                height: area.height.saturating_sub(2),
            },
            line_number_area: None,
            content_area: Rect {
                x: 0,
                y: 1,
                width: area.width,
                height: area.height.saturating_sub(2),
            },
            command_line_area: Rect {
                x: 0,
                y: area.height.saturating_sub(1),
                width: area.width,
                height: 1,
            },
        }
    }

    /// Get area by name, just to decouple from using direct variable names in plugins
    pub fn get(&self, area_type: &str) -> Option<Rect> {
        match area_type {
            "editor" => Some(self.editor_area),
            "content" => Some(self.content_area),
            "line_numbers" => self.line_number_area,
            "statusbar" => self.status_bar_area,
            "commandline" => Some(self.command_line_area),
            _ => None,
        }
    }
}
