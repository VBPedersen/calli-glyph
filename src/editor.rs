use crate::config::editor_settings;
use crate::cursor::Cursor;
use crate::cursor::CursorPosition;
use crate::errors::{ClipboardError, EditorError, TextSelectionError};

/// handles editor content
#[derive(Debug)]
pub struct Editor {
    pub editor_content: Vec<String>,
    pub visual_cursor_x: i16,
    pub cursor: Cursor, //to save position in editor, when toggling area
    pub text_selection_start: Option<CursorPosition>,
    pub text_selection_end: Option<CursorPosition>,
    pub editor_width: i16,
    pub scroll_offset: i16,
    pub editor_height: u16,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            editor_content: vec![],
            visual_cursor_x: 0,
            text_selection_start: None,
            text_selection_end: None,
            cursor: Cursor::new(),
            editor_width: 0,
            scroll_offset: 0,
            editor_height: 0,
        }
    }

    //copy, cut and paste

    ///copies text within bound of text selected to copied_text
    pub fn copy_selected_text(&mut self) -> Result<Vec<String>, TextSelectionError> {
        if let (Some(start), Some(end)) = (
            self.text_selection_start,
            self.text_selection_end,
        ) {
            let mut selected_text: Vec<String> = Vec::new();
            let lines = &self.editor_content[start.y..=end.y];

            if lines.len() > 1 {
                for (y, line) in lines.iter().enumerate() {
                    let mut line_chars: Vec<char> = line.chars().collect();
                    let extracted_text: String;

                    //if first line drain all from start x,
                    // else if last line drain to end .x, else drain all
                    if y == 0 {
                        extracted_text = line_chars.drain(start.x..).collect();
                    } else if y == lines.len() - 1 {
                        extracted_text = line_chars.drain(..end.x).collect();
                    } else {
                        extracted_text = line_chars.into_iter().collect();
                    }

                    selected_text.push(extracted_text);
                }
            } else {
                let mut line_chars: Vec<char> =
                    self.editor_content[start.y].chars().collect();
                let extracted_text: String = line_chars.drain(start.x..end.x).collect();
                selected_text.push(extracted_text);
            }

            Ok(selected_text)
        } else {
            Err(TextSelectionError::NoTextSelected)
        }
    }

    ///cuts text within bound of text selected to copied_text
    pub fn cut_selected_text(&mut self) -> Result<Vec<String>, TextSelectionError> {
        if let (Some(start), Some(end)) = (
            self.text_selection_start,
            self.text_selection_end,
        ) {
            let mut selected_text: Vec<String> = Vec::new();
            let lines = self.editor_content[start.y..=end.y].as_mut();
            let line_length = lines.len();
            if lines.len() > 1 {
                for (y, line) in lines.iter_mut().enumerate() {
                    let mut line_chars: Vec<char> = line.as_mut().chars().collect();
                    let extracted_text: String;

                    //if first line drain all from start x,
                    // else if last line drain to end .x, else drain all
                    if y == 0 {
                        extracted_text = line_chars.drain(start.x..).collect();
                    } else if y == line_length - 1 {
                        extracted_text = line_chars.drain(..end.x).collect();
                    } else {
                        extracted_text = line_chars.drain(..).collect();
                    }

                    selected_text.push(extracted_text);
                    *line = line_chars.into_iter().collect();
                }
            } else {
                let lines = self.editor_content[start.y..start.y + 1].as_mut();
                let line = lines.iter_mut().next().unwrap();
                let mut line_chars: Vec<char> = line.as_mut().chars().collect();
                let extracted_text: String = line_chars.drain(start.x..end.x).collect();
                selected_text.push(extracted_text);

                *line = line_chars.into_iter().collect();
            }

            Ok(selected_text)
        } else {
            Err(TextSelectionError::NoTextSelected)
        }
    }

    ///pastes text from copied text to editor content
    pub fn paste_selected_text(&mut self, copied_text: Vec<String>) -> Result<(), ClipboardError> {
        //if no text in copied text
        if copied_text.is_empty() {
            return Err(ClipboardError::NoCopiedText);
        }

        let insert_y = self.cursor.y as usize;
        let insert_x = self.cursor.x as usize;

        while self.editor_content.len() < insert_y + copied_text.len() - 1 {
            self.editor_content.push(String::new());
        }

        let current_line = &self.editor_content[insert_y];

        // Convert the line into a Vec<char> to handle multi-byte characters correctly
        let chars: Vec<char> = current_line.chars().collect();
        let (before_cursor, after_cursor) = chars.split_at(insert_x.min(chars.len()));

        if copied_text.len() == 1 {
            // Single-line paste: correctly insert into character-safe split
            let new_line = format!(
                "{}{}{}",
                before_cursor.iter().collect::<String>(),
                copied_text[0],
                after_cursor.iter().collect::<String>()
            );
            self.editor_content[insert_y] = new_line;
        } else {
            // Multi-line paste
            let mut new_lines = Vec::new();

            // First line: insert copied text at cursor position
            new_lines.push(format!(
                "{}{}",
                before_cursor.iter().collect::<String>(),
                copied_text[0]
            ));

            // Middle lines: insert as separate lines
            for line in &copied_text[1..copied_text.len() - 1] {
                new_lines.push(line.clone());
            }

            // Last copied line + remainder of the original line
            let last_copied_line =
                &copied_text[copied_text.len() - 1];
            new_lines.push(format!(
                "{}{}",
                last_copied_line,
                after_cursor.iter().collect::<String>()
            ));

            // Replace the current line and insert new lines
            self
                .editor_content
                .splice(insert_y..=insert_y, new_lines);
        }

        // Clear copied text after pasting
        //self.copied_text.clear();
        Ok(())
    }




    //editor writing
    ///writes char to y position line, with x position
    pub fn write_char(&mut self, c: char) {
        //creating lines until y position of cursor
        while self.editor_content.len() <= self.cursor.y as usize {
            self.editor_content.push(String::new());
        }

        let line = &mut self.editor_content[self.cursor.y as usize];

        let char_count = line.chars().count();
        //position cursor to line end in chars count
        if char_count < self.cursor.x as usize {
            self.cursor.x = char_count as i16;
        }

        let mut line_chars_vec: Vec<char> = line.chars().collect();

        line_chars_vec.insert(self.cursor.x as usize, c);

        *line = line_chars_vec.into_iter().collect();

        self.move_cursor(1, 0);
    }


    ///replaces all selected text with char to y position line, with x position
    pub fn write_char_text_is_selected(&mut self, c: char) {
        let start = self.text_selection_start.unwrap();
        let end = self.text_selection_end.unwrap();
        let lines = &mut self.editor_content[start.y..=end.y];
        let lines_length = lines.len();
        if lines_length > 1 {
            for (y, line) in lines.iter_mut().enumerate() {
                let mut line_chars_vec: Vec<char> = line.chars().collect();

                //last line selected
                if y == lines_length - 1 {
                    line_chars_vec.drain(0..end.x);
                } else {
                    line_chars_vec.drain(start.x..line.chars().count());
                }

                //start line selected
                if y == 0 {
                    line_chars_vec.insert(start.x, c);
                }

                *line = line_chars_vec.into_iter().collect();
            }
        } else {
            let line = &mut self.editor_content[start.y];
            let mut line_chars_vec: Vec<char> = line.chars().collect();
            line_chars_vec.drain(start.x..end.x);
            line_chars_vec.insert(start.x, c);
            *line = line_chars_vec.into_iter().collect();
        }
        self.cursor.x = self.text_selection_start.unwrap().x as i16;
        self.cursor.y = self.text_selection_start.unwrap().y as i16;
        self.text_selection_start = None;
        self.text_selection_end = None;
        self.move_cursor(1, 0);
    }

    //editor tab character
    ///handles TAB action in editor, by writing \t to editor content.
    pub fn tab(&mut self) {
        let line = &mut self.editor_content[self.cursor.y as usize];

        let mut line_chars_vec: Vec<char> = line.chars().collect();

        line_chars_vec.insert(self.cursor.x as usize, '\t');

        *line = line_chars_vec.into_iter().collect();

        self.move_cursor(1, 0)
    }

    //editor enter
    ///handles enter new line, with possible move of text
    pub fn enter(&mut self) {
        let line = &mut self.editor_content[self.cursor.y as usize];
        //if at end of line len, then just move cursor and make new line, else move text too
        if self.cursor.x >= line.chars().count() as i16 {
            self
                .editor_content
                .insert(self.cursor.y as usize + 1, String::new());
            self.move_cursor(0, 1);
        } else {
            //split current line and remove split part
            let mut line_chars_vec: Vec<char> = line.chars().collect();
            let line_end = line_chars_vec.split_off(self.cursor.x as usize);
            *line = line_chars_vec.into_iter().collect();

            //move down and insert split line to next line
            self.move_cursor(0, 1);
            self.editor_content
                .insert(self.cursor.y as usize, String::new());
            self.editor_content[self.cursor.y as usize] =
                line_end.into_iter().collect();
            self.cursor.x = 0;
        }
    }


    //editor backspace
    ///handles backspace in editor, removes char at y line x position and sets new cursor position
    pub fn backspace_in_editor(&mut self) {
        let line_char_count = self.editor_content[self.cursor.y as usize]
            .chars()
            .count() as i16;
        if self.cursor.x > 0 && self.cursor.x <= line_char_count {
            let line = &mut self.editor_content[self.cursor.y as usize];
            let mut line_chars_vec: Vec<char> = line.chars().collect();

            line_chars_vec.remove(self.cursor.x as usize - 1);

            *line = line_chars_vec.into_iter().collect();
            //line.remove(self.editor.cursor.x as usize -1);
            self.move_cursor(-1, 0);
        } else if self.cursor.y > 0 {
            let line = &mut self
                .editor_content
                .remove(self.cursor.y as usize);
            let new_x_value = self.editor_content[(self.cursor.y - 1) as usize]
                .chars()
                .count() as i16;
            self.cursor.y -= 1;
            self.cursor.x = new_x_value;
            self.editor_content[self.cursor.y as usize].push_str(line);
        }
    }

    ///handles backspace in editor, removes char at y line x position and sets new cursor position
    pub fn backspace_text_is_selected(&mut self) {
        let start = self.text_selection_start.unwrap();
        let end = self.text_selection_end.unwrap();
        let lines = &mut self.editor_content[start.y..=end.y];
        let lines_length = lines.len();
        if lines_length > 1 {
            for (y, line) in lines.iter_mut().enumerate() {
                let mut line_chars_vec: Vec<char> = line.chars().collect();
                //last line selected
                if y == lines_length - 1 {
                    line_chars_vec.drain(0..end.x);
                } else {
                    line_chars_vec.drain(start.x..line.chars().count());
                }

                *line = line_chars_vec.into_iter().collect();
            }
        } else {
            let line = &mut self.editor_content[start.y];
            let mut line_chars_vec: Vec<char> = line.chars().collect();
            line_chars_vec.drain(start.x..end.x);
            *line = line_chars_vec.into_iter().collect();
        }
        self.cursor.x = self.text_selection_start.unwrap().x as i16;
        self.cursor.y = self.text_selection_start.unwrap().y as i16;
        self.text_selection_start = None;
        self.text_selection_end = None;
        //replace visual cursor
        self.visual_cursor_x = self.calculate_visual_x() as i16;
    }

    //editor delete functions

    ///handles DELETE action, of deleting char in editor at x +1 position
    pub(crate) fn delete_in_editor(&mut self) {
        let current_line_len = self.editor_content[self.cursor.y as usize]
            .chars()
            .count() as i16;

        if current_line_len == 0 {
            return;
        }
        //if at line end, move line below up,  else if current line length is bigger than current cursor x pos, remove char
        if self.cursor.x >= current_line_len - 1
            && self.editor_content.len() > (self.cursor.y + 1) as usize
        {
            let line = &mut self
                .editor_content
                .remove((self.cursor.y + 1) as usize);
            self.editor_content[self.cursor.y as usize].push_str(line);
        } else if current_line_len > (self.cursor.x + 1) {
            let line = &mut self.editor_content[self.cursor.y as usize];
            let mut line_chars_vec: Vec<char> = line.chars().collect();

            line_chars_vec.remove(self.cursor.x as usize + 1);

            *line = line_chars_vec.into_iter().collect();
            //line.remove((self.editor.cursor.x+1) as usize);
        }
    }

    ///handles delete in editor, removes char at y line x position and sets new cursor position
    pub fn delete_text_is_selected(&mut self) {
        let start = self.text_selection_start.unwrap();
        let end = self.text_selection_end.unwrap();
        let lines = &mut self.editor_content[start.y..=end.y];
        let lines_length = lines.len();
        if lines_length > 1 {
            for (y, line) in lines.iter_mut().enumerate() {
                let mut line_chars_vec: Vec<char> = line.chars().collect();

                //last line selected
                if y == lines_length - 1 {
                    //line_chars_vec.drain(0..end.x);
                    line_chars_vec[0..end.x].fill(' ');
                } else {
                    //line_chars_vec[start.x..line.chars().count()].fill(' ');
                    line_chars_vec.drain(start.x..line.chars().count());
                }

                *line = line_chars_vec.into_iter().collect();
            }
        } else {
            let line = &mut self.editor_content[start.y];
            let mut line_chars_vec: Vec<char> = line.chars().collect();
            line_chars_vec[start.x..end.x].fill(' ');
            //line_chars_vec.drain(start.x..end.x);
            *line = line_chars_vec.into_iter().collect();
        }
        self.cursor.x = self.text_selection_end.unwrap().x as i16;
        self.cursor.y = self.text_selection_end.unwrap().y as i16;
        self.text_selection_start = None;
        self.text_selection_end = None;
        //replace visual cursor
        self.visual_cursor_x = self.calculate_visual_x() as i16;
    }


    //editor cursor moving


    ///moves the cursor in relation to editor content
    pub fn move_cursor(&mut self, x: i16, y: i16) {
        if self.cursor.y == 0 && y == -1 {
            return;
        }
        //if wanting to go beyond current length of editor
        while self.editor_content.len() <= (self.cursor.y + y) as usize {
            self.editor_content.push(String::new());
        }

        let max_x_pos = self.editor_content[(self.cursor.y + y) as usize]
            .chars()
            .count() as i16;
        //let current_line = &self.editor.editor_content[self.editor.cursor.y as usize];

        // Moving Right →
        if x > 0 && self.cursor.x < max_x_pos {
            self.cursor.x += x;
        } else if x == 1
            && self.cursor.x
            >= self.editor_content[self.cursor.y as usize]
            .chars()
            .count() as i16
            && self.editor_content.len() > self.cursor.y as usize + 1
        {
            //else if end of line and more lines
            self.cursor.y += 1;
            self.cursor.x = 0;
            self.visual_cursor_x = self.calculate_visual_x() as i16;
            return;
        }

        // Moving Left ←
        if x < 0 && self.cursor.x > 0 {
            self.cursor.x += x;
        } else if self.cursor.x == 0 && x == -1 && self.cursor.y != 0 {
            //else if start of line and more lines
            self.cursor.y -= 1;
            self.cursor.x = self.editor_content[self.cursor.y as usize]
                .chars()
                .count() as i16;
            self.visual_cursor_x = self.calculate_visual_x() as i16;
            return;
        }

        let (top, bottom) = self.is_cursor_top_or_bottom_of_editor();
        //to offset scroll
        if (y == 1 && bottom) || (y == -1 && top) {
            self.scroll_offset = (self.scroll_offset + y).clamp(0, i16::MAX);
            return;
        }

        self.cursor.x = self.cursor.x.clamp(0, max_x_pos);
        self.cursor.y = (self.cursor.y + y).clamp(0, i16::MAX);
        self.visual_cursor_x = self.calculate_visual_x() as i16;

    }

    ///moves selection cursor
    pub(crate) fn move_selection_cursor(&mut self, x: i16, y: i16) {
        let old_x = self.cursor.x;
        let old_y = self.cursor.y;
        self.move_cursor(x, y);
        let new_x = self.cursor.x;
        let new_y = self.cursor.y;

        let new_pos = CursorPosition {
            x: new_x as usize,
            y: new_y as usize,
        };
        let old_pos = CursorPosition {
            x: old_x as usize,
            y: old_y as usize,
        };

        if self.text_selection_start.is_none() {
            // Initialize selection start on first move
            self.text_selection_start = Some(old_pos);
        }

        if self.text_selection_end.is_none() {
            // Initialize selection end on first move
            self.text_selection_end = Some(old_pos);
        }

        let (at_start, at_end) = self.is_selection_cursor_start_or_end(old_pos);

        if x > 0 || y > 0 {
            // Moving right/down → Extend selection

            if at_start && !at_end {
                //is at start pos and should move start instead of end
                self.text_selection_start = Some(new_pos);
            } else if at_end && !at_start {
                //is at end pos and should move end instead of start
                self.text_selection_end = Some(new_pos);
            } else {
                //is at both start and end, should move end
                self.text_selection_end = Some(new_pos);
            }
        } else if x < 0 || y < 0 {
            // Moving left/up → Adjust start instead of resetting
            if at_start && !at_end {
                //is at start pos and should move start instead of end
                self.text_selection_start = Some(new_pos);
            } else if at_end && !at_start {
                //is at end pos and should move end instead of start
                self.text_selection_end = Some(new_pos);
            } else {
                //is at both start and end, should move start
                self.text_selection_start = Some(new_pos);
            }
        }
    }

    fn is_selection_cursor_start_or_end(&self, current_pos: CursorPosition) -> (bool, bool) {
        let start = current_pos.x == self.text_selection_start.unwrap().x
            && current_pos.y == self.text_selection_start.unwrap().y;
        let end = current_pos.x == self.text_selection_end.unwrap().x
            && current_pos.y == self.text_selection_end.unwrap().y;
        (start, end)
    }


    //SCROLL
    ///moves the scroll offset
    pub(crate) fn move_scroll_offset(&mut self, offset: i16) {
        let (top, bottom) = self.is_cursor_top_or_bottom_of_editor();

        //if on way down and at bottom, move scroll
        if (offset == 1 && bottom) || (offset == -1 && top) {
            self.scroll_offset = (self.scroll_offset + offset).clamp(0, i16::MAX);
            return;
        }

        self.move_cursor(0, offset);
    }

    ///calculates the visual position of the cursor
    fn calculate_visual_x(&mut self) -> usize {
        let line = &self.editor_content[self.cursor.y as usize];
        let cursor_x = self.cursor.x as usize;
        let tab_width = editor_settings::TAB_WIDTH as usize;
        let mut visual_x = 0;
        for (i, c) in line.chars().enumerate() {
            if i == cursor_x {
                break;
            }

            if c == '\t' {
                visual_x += tab_width - (visual_x % tab_width);
            } else {
                visual_x += 1;
            }
        }

        visual_x
    }
    ///checks if cursor is at top or bottom of the screen
    fn is_cursor_top_or_bottom_of_editor(&self) -> (bool, bool) {
        let top = self.cursor.y == self.scroll_offset;
        let bottom = self.cursor.y == self.scroll_offset + (self.editor_height as i16);
        (top, bottom)
    }




}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}
