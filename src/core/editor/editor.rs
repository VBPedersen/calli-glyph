use super::super::super::core::clipboard::Clipboard;
use super::super::cursor::Cursor;
use super::super::cursor::CursorPosition;
use super::super::errors::editor_errors::EditorError::{
    ClipboardFailure, RedoFailure, TextSelectionFailure, UndoFailure,
};
use super::super::errors::editor_errors::{ClipboardError, EditorError, TextSelectionError};
use super::undo_redo::UndoRedoManager;
use crate::config::editor_settings;
use crate::input::input_action::InputAction;

#[derive(Debug, Clone)]
pub enum EditAction {
    // single-char operations
    Insert {
        pos: CursorPosition,
        c: char,
    },
    Delete {
        pos: CursorPosition,
        deleted_char: char,
    },
    Replace {
        start: CursorPosition,
        end: CursorPosition,
        old: char,
        new: char,
    },
    // multi-char operations
    InsertLines {
        start: CursorPosition,
        lines: Vec<String>,
    },
    DeleteLines {
        start: CursorPosition,
        deleted: Vec<String>,
    },
}

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
    pub clipboard: Clipboard,
    undo_redo_manager: UndoRedoManager,
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
            clipboard: Clipboard::new(),
            undo_redo_manager: UndoRedoManager::new(),
        }
    }

    ///function to handle input action on editor,
    /// responsible for dispatching action to correct internal method.
    pub fn handle_input_action(&mut self, action: InputAction) -> Result<(), EditorError> {
        match action {
            InputAction::MoveCursor(direction) => {
                let (x, y) = direction.to_vector();
                self.move_cursor(x, y);
                self.text_selection_start = None;
                self.text_selection_end = None;
                Ok(())
            }
            InputAction::MoveSelectionCursor(direction) => {
                let (x, y) = direction.to_vector();
                self.move_selection_cursor(x, y);
                Ok(())
            }
            InputAction::TAB => {
                self.tab();
                Ok(())
            }
            InputAction::ENTER => {
                self.enter();
                Ok(())
            }
            InputAction::BACKSPACE => {
                if self.is_text_selected() {
                    self.backspace_text_is_selected();
                } else {
                    self.backspace();
                }
                Ok(())
            }
            InputAction::DELETE => {
                if self.is_text_selected() {
                    self.delete_text_is_selected();
                } else {
                    self.delete();
                }
                Ok(())
            }
            InputAction::COPY => match self.copy() {
                Ok(()) => Ok(()),
                Err(e) => Err(e),
            },
            InputAction::CUT => match self.cut() {
                Ok(()) => Ok(()),
                Err(e) => Err(e),
            },
            InputAction::PASTE => match self.paste() {
                Ok(()) => Ok(()),
                Err(e) => Err(e),
            },
            InputAction::UNDO => match self.undo() {
                Ok(()) => Ok(()),
                Err(e) => Err(e),
            },
            InputAction::REDO => match self.redo() {
                Ok(()) => Ok(()),
                Err(e) => Err(e),
            },
            InputAction::WriteChar(c) => {
                if self.is_text_selected() {
                    self.write_char_text_is_selected(c)
                } else {
                    self.write_char(c)
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    //undo manager
    ///undo wrapper function, that calls the UndoRedoManager
    pub fn undo(&mut self) -> Result<(), EditorError> {
        match self.undo_redo_manager.undo() {
            Ok(action) => {
                self.apply_action(&action);
                Ok(())
            }
            Err(e) => Err(UndoFailure(e)),
        }
    }
    ///redo wrapper function, that calls the UndoRedoManager
    pub fn redo(&mut self) -> Result<(), EditorError> {
        match self.undo_redo_manager.redo() {
            Ok(action) => {
                self.apply_action(&action);
                Ok(())
            }
            Err(e) => Err(RedoFailure(e)),
        }
    }

    /// applies an EditAction
    fn apply_action(&mut self, action: &EditAction) {
        match action {
            EditAction::Insert { pos, c } => {
                self.insert_char_at(*pos, *c);
                let additive_pos = CursorPosition { x: pos.x + 1, y: 0 };
                let end: CursorPosition = *pos + additive_pos;
                self.set_cursor_position(&end);
            }
            EditAction::Delete { pos, .. } => {
                self.delete_char_at(*pos);
                self.set_cursor_position(pos);
            }
            EditAction::Replace {
                start, end, new, ..
            } => {
                self.replace_selection_with_text(*start, *end, *new);
                self.set_cursor_position(start);
            }
            EditAction::InsertLines { start, lines } => {
                self.insert_lines_at(*start, lines.clone());
                //get additive position to get new cursor pos at end of insertion
                let last_line_len = lines.last().map(|s| s.len()).unwrap_or(0);
                let additive_pos = CursorPosition {
                    x: last_line_len,
                    y: lines.iter().count(),
                };
                let end: CursorPosition = *start + additive_pos;
                self.set_cursor_position(&end);
            } //in delete lines cursor position calculated first,
            // as visual x cannot be calculated without specific y line present
            EditAction::DeleteLines { start, deleted } => {
                //get additive position to get new cursor pos at end of insertion
                let last_line_len = deleted.last().map(|s| s.len()).unwrap_or(0);
                let negated_pos = CursorPosition {
                    x: last_line_len,
                    y: deleted.iter().count(),
                };
                let end: CursorPosition = *start - negated_pos;
                self.set_cursor_position(&end);
                self.delete_lines_at(*start, deleted.len());
            }
        }
    }

    //copy, cut and paste

    ///base function for copy that copies if text is selected
    pub fn copy(&mut self) -> Result<(), EditorError> {
        match self.copy_selected_text() {
            Ok(selected_text) => {
                //copy to clipboard
                self.clipboard.copy(&*selected_text);
                //reset text selection
                self.text_selection_start = None;
                self.text_selection_end = None;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    ///copies text within bound of text selected to copied_text
    pub fn copy_selected_text(&mut self) -> Result<Vec<String>, EditorError> {
        if let (Some(start), Some(end)) = (self.text_selection_start, self.text_selection_end) {
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
                let mut line_chars: Vec<char> = self.editor_content[start.y].chars().collect();
                let extracted_text: String = line_chars.drain(start.x..end.x).collect();
                selected_text.push(extracted_text);
            }

            Ok(selected_text)
        } else {
            Err(TextSelectionFailure(TextSelectionError::NoTextSelected))
        }
    }

    ///base function for cut that cuts if text is selected
    pub fn cut(&mut self) -> Result<(), EditorError> {
        match self.cut_selected_text() {
            Ok(selected_text) => {
                //copy to clipboard
                self.clipboard.copy(&*selected_text);
                //reset text selection
                self.text_selection_start = None;
                self.text_selection_end = None;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    ///cuts text within bound of text selected to copied_text
    pub fn cut_selected_text(&mut self) -> Result<Vec<String>, EditorError> {
        if let (Some(start), Some(end)) = (self.text_selection_start, self.text_selection_end) {
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
            Err(TextSelectionFailure(TextSelectionError::NoTextSelected))
        }
    }

    ///base function for paste
    pub fn paste(&mut self) -> Result<(), EditorError> {
        match self.paste_selected_text(self.clipboard.copied_text.clone()) {
            Ok(()) => Ok(()),
            Err(e) => Err(e),
        }
    }

    ///pastes text from copied text to editor content
    pub fn paste_selected_text(&mut self, copied_text: Vec<String>) -> Result<(), EditorError> {
        //if no text in copied text
        if copied_text.is_empty() {
            return Err(ClipboardFailure(ClipboardError::NoCopiedText));
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
            let last_copied_line = &copied_text[copied_text.len() - 1];
            new_lines.push(format!(
                "{}{}",
                last_copied_line,
                after_cursor.iter().collect::<String>()
            ));

            // Replace the current line and insert new lines
            self.editor_content.splice(insert_y..=insert_y, new_lines);
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
        //record undo action (action done)
        self.undo_redo_manager.record_undo(EditAction::Insert {
            pos: CursorPosition {
                x: self.cursor.x as usize,
                y: self.cursor.y as usize,
            },
            c,
        });

        self.move_cursor(1, 0);
    }

    ///replaces all selected text with char to y position line, with x position
    pub fn write_char_text_is_selected(&mut self, c: char) {
        let start = self.text_selection_start.unwrap();
        let end = self.text_selection_end.unwrap();
        let lines = &mut self.editor_content[start.y..=end.y];
        let lines_length = lines.len();
        if lines_length > 1 {
            let mut line_indexes_to_remove: Vec<u16> = vec![];
            for (y, line) in lines.iter_mut().enumerate() {
                let mut line_chars_vec: Vec<char> = line.chars().collect();
                //first line
                if y == 0 {
                    line_chars_vec.drain(start.x..line.chars().count());
                    line_chars_vec.insert(start.x, c); //write chat to start position
                } else if y == lines_length - 1 {
                    //last line selected
                    line_chars_vec.drain(0..end.x);
                } else {
                    line_chars_vec.drain(0..line.chars().count());
                    line_indexes_to_remove.push((start.y + y) as u16);
                }
                *line = line_chars_vec.into_iter().collect();
            }
            // remove the lines that became empty in reverse order
            for &i in line_indexes_to_remove.iter().rev() {
                self.editor_content.remove(i as usize);
            }
            //move content of last line selected to first line start point
            let line = &mut self
                .editor_content
                .remove(end.y - line_indexes_to_remove.len());
            self.editor_content[start.y].push_str(line);
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

        self.undo_redo_manager.record_undo(EditAction::Insert {
            pos: CursorPosition {
                x: self.cursor.x as usize,
                y: self.cursor.y as usize,
            },
            c: '\t',
        });
        self.move_cursor(1, 0)
    }

    //editor enter
    ///handles enter new line, with possible move of text
    pub fn enter(&mut self) {
        let line = &mut self.editor_content[self.cursor.y as usize];
        //if at end of line len, then just move cursor and make new line, else move text too
        if self.cursor.x >= line.chars().count() as i16 {
            self.editor_content
                .insert(self.cursor.y as usize + 1, String::new());
            //record undo
            self.undo_redo_manager.record_undo(EditAction::InsertLines {
                start: CursorPosition {
                    x: self.cursor.x as usize,
                    y: self.cursor.y as usize + 1, //+1 y to insert after current index
                },
                lines: vec![String::new()],
            });
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
            self.editor_content[self.cursor.y as usize] = line_end.into_iter().collect();
            self.cursor.x = 0;
        }
    }

    //editor backspace
    ///handles backspace in editor, removes char at y line x position and sets new cursor position
    pub fn backspace(&mut self) {
        let mut deleted_char: Option<char> = None;
        let line_char_count = self.editor_content[self.cursor.y as usize].chars().count() as i16;
        //if x is more than 0 and less than max line index : should delete char and move back
        // else if y is more than 0, move line up
        if self.cursor.x > 0 && self.cursor.x <= line_char_count {
            let line = &mut self.editor_content[self.cursor.y as usize];
            let mut line_chars_vec: Vec<char> = line.chars().collect();
            let char = line_chars_vec.remove(self.cursor.x as usize - 1);
            deleted_char = Some(char);

            *line = line_chars_vec.into_iter().collect();
            //line.remove(self.editor.cursor.x as usize -1);
            self.move_cursor(-1, 0);
        } else if self.cursor.y > 0 {
            let line = &mut self.editor_content.remove(self.cursor.y as usize);
            let new_x_value = self.editor_content[(self.cursor.y - 1) as usize]
                .chars()
                .count() as i16;
            self.cursor.y -= 1;
            self.cursor.x = new_x_value;
            self.editor_content[self.cursor.y as usize].push_str(line);
        }

        if let Some(char) = deleted_char {
            self.undo_redo_manager.record_undo(EditAction::Delete {
                pos: CursorPosition {
                    x: self.cursor.x as usize,
                    y: self.cursor.y as usize,
                },
                deleted_char: char.clone(),
            });
        }
    }

    ///handles backspace in editor, removes char at y line x position and sets new cursor position
    pub fn backspace_text_is_selected(&mut self) {
        let start = self.text_selection_start.unwrap();
        let end = self.text_selection_end.unwrap();
        let lines = &mut self.editor_content[start.y..=end.y];
        let lines_length = lines.len();
        if lines_length > 1 {
            let mut line_indexes_to_remove: Vec<u16> = vec![];
            for (y, line) in lines.iter_mut().enumerate() {
                let mut line_chars_vec: Vec<char> = line.chars().collect();
                //first line
                if y == 0 {
                    line_chars_vec.drain(start.x..line.chars().count());
                } else if y == lines_length - 1 {
                    //last line selected
                    line_chars_vec.drain(0..end.x);
                } else {
                    line_chars_vec.drain(0..line.chars().count());
                    line_indexes_to_remove.push((start.y + y) as u16);
                }
                *line = line_chars_vec.into_iter().collect();
            }
            // remove the lines that became empty in reverse order
            for &i in line_indexes_to_remove.iter().rev() {
                self.editor_content.remove(i as usize);
            }
            //move content of last line selected to first line start point
            let line = &mut self
                .editor_content
                .remove(end.y - line_indexes_to_remove.len());
            self.editor_content[start.y].push_str(line);
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
    pub(crate) fn delete(&mut self) {
        let current_line_len = self.editor_content[self.cursor.y as usize].chars().count() as i16;

        if current_line_len == 0 {
            return;
        }
        //if at line end, move line below up,  else if current line length is bigger than current cursor x pos, remove char
        if self.cursor.x >= current_line_len - 1
            && self.editor_content.len() > (self.cursor.y + 1) as usize
        {
            let line = &mut self.editor_content.remove((self.cursor.y + 1) as usize);
            self.editor_content[self.cursor.y as usize].push_str(line);
            self.undo_redo_manager.record_undo(EditAction::DeleteLines {
                start: CursorPosition {
                    x: self.cursor.x as usize,
                    y: self.cursor.y as usize,
                },
                deleted: vec![(*line).parse().unwrap()],
            });
        } else if current_line_len > (self.cursor.x + 1) {
            let line = &mut self.editor_content[self.cursor.y as usize];
            let mut line_chars_vec: Vec<char> = line.chars().collect();
            let char = line_chars_vec.remove(self.cursor.x as usize + 1);

            self.undo_redo_manager.record_undo(EditAction::Delete {
                pos: CursorPosition {
                    x: self.cursor.x as usize + 1,
                    y: self.cursor.y as usize,
                },
                deleted_char: char.clone(),
            });

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
                //first line
                if y == 0 {
                    line_chars_vec.drain(start.x..line.chars().count());
                } else if y == lines_length - 1 {
                    //last line selected
                    //line_chars_vec.drain(0..end.x);   this takes away the chars
                    //this solution replaces with whitespace
                    for i in 0..end.x.min(line_chars_vec.len()) {
                        line_chars_vec[i] = ' ';
                    }
                } else {
                    line_chars_vec.drain(0..line.chars().count());
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
            && self.cursor.x >= self.editor_content[self.cursor.y as usize].chars().count() as i16
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
            self.cursor.x = self.editor_content[self.cursor.y as usize].chars().count() as i16;
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

//██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
//██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
//███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
//██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
//██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
//╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝

impl Editor {
    ///function to check if some text is selected
    fn is_text_selected(&self) -> bool {
        self.text_selection_start.is_some() && self.text_selection_end.is_some()
    }

    /// sets cursor position to specified position
    pub(crate) fn set_cursor_position(&mut self, pos: &CursorPosition) {
        //clamp set position to at maximum go to last position available
        //y is len of editor -1
        self.cursor.y = (pos.y as i16).clamp(0, self.editor_content.len() as i16 - 1);
        //length of cursor.y line , aka nr. of chars on line.
        let line_len = self
            .editor_content
            .get(self.cursor.y as usize)
            .map(|s| s.len())
            .unwrap_or(0);
        self.cursor.x = (pos.x as i16).clamp(0, line_len as i16);
        //calculate visual x pos again.
        self.visual_cursor_x = self.calculate_visual_x() as i16;
    }

    /// Insert a character at the specified position (buffer-only: does not touch undo/redo, does not move main cursor)
    pub(crate) fn insert_char_at(&mut self, pos: CursorPosition, c: char) {
        // Ensure the target line exists
        while self.editor_content.len() <= pos.y {
            self.editor_content.push(String::new());
        }
        let line = &mut self.editor_content[pos.y];
        let mut chars: Vec<char> = line.chars().collect();
        // Clamp to actual line length
        let x = pos.x.min(chars.len());
        chars.insert(x, c);
        *line = chars.into_iter().collect();
        // cursor not updated
    }

    /// Delete a character at the specified position (buffer-only)
    pub(crate) fn delete_char_at(&mut self, pos: CursorPosition) {
        if let Some(line) = self.editor_content.get_mut(pos.y) {
            let mut chars: Vec<char> = line.chars().collect();
            if pos.x < chars.len() {
                chars.remove(pos.x);
                *line = chars.into_iter().collect();
            }
        }
    }

    /// Replace a text selection (from start to end) with new text
    pub(crate) fn replace_selection_with_text(
        &mut self,
        start: CursorPosition,
        end: CursorPosition,
        new: char,
    ) {
        if start.y == end.y {
            if let Some(line) = self.editor_content.get_mut(start.y) {
                let mut chars: Vec<char> = line.chars().collect();
                // Clamp positions
                let start_x = start.x.min(chars.len());
                let end_x = end.x.min(chars.len());
                chars.drain(start_x..end_x);
                chars.insert(start_x, new);
                *line = chars.into_iter().collect();
            }
        }
        // For multi-line selection, you may wish to expand further as needed.
    }

    /// Insert multiple lines at a position
    pub(crate) fn insert_lines_at(&mut self, start: CursorPosition, lines: Vec<String>) {
        //calculate start y to insert
        let y = start.y.min(self.editor_content.len());
        for (i, line) in lines.into_iter().enumerate() {
            self.editor_content.insert(y + i, line);
        }
    }

    /// Delete lines starting at a position
    pub(crate) fn delete_lines_at(&mut self, start: CursorPosition, count: usize) {
        let y = start.y;
        for _ in 0..count {
            if y < self.editor_content.len() {
                self.editor_content.remove(y);
            } else {
                break;
            }
        }
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

//████████╗███████╗███████╗████████╗███████╗
//╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//   ██║   █████╗  ███████╗   ██║   ███████╗
//   ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//   ██║   ███████╗███████║   ██║   ███████║
//   ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
#[cfg(test)]
mod unit_editor_write_tests {
    use super::super::super::cursor::CursorPosition;
    use super::super::editor::*;
    use crate::config::editor_settings;

    //init functions
    fn create_editor_with_editor_content(vec: Vec<String>) -> Editor {
        let mut editor = Editor::new();
        editor.editor_content = vec;
        editor.editor_height = 10; //since testing doesnt start ui.rs, height isnt set
        editor
    }

    #[test]
    fn test_write_char() {
        let mut editor = Editor::new();
        editor.write_char('a');
        assert_eq!(editor.editor_content[0], "a");
        assert_eq!(editor.cursor.x, 1);
    }

    #[test]
    fn test_write_char_normal_characters() {
        let mut editor = Editor::new();
        editor.write_char('a');
        editor.write_char('b');
        editor.write_char('c');
        editor.write_char('d');
        assert_eq!(editor.editor_content[0], "abcd");
        assert_eq!(editor.cursor.x, 4);
    }

    #[test]
    fn test_write_char_special_characters() {
        let mut editor = Editor::new();
        editor.write_char('ᚠ');
        editor.write_char('Ω');
        editor.write_char('₿');
        editor.write_char('😎');
        assert_eq!(editor.editor_content[0], "ᚠΩ₿😎");
        assert_eq!(editor.cursor.x, 4);
    }

    #[test]
    fn test_write_char_at_line_10() {
        let mut editor = Editor::new();
        editor.cursor.y = 10;
        editor.write_char('a');
        assert_eq!(editor.editor_content[10], "a");
        assert_eq!(editor.cursor.x, 1);
    }

    #[test]
    fn test_write_char_at_100_x() {
        let mut editor = Editor::new();
        editor.cursor.x = 100;
        editor.write_char('a');
        assert_eq!(editor.editor_content[0], "a");
        assert_eq!(editor.cursor.x, 1);
    }

    //Write char to editor with selected text
    #[test]
    fn test_write_char_with_selected_text() {
        let mut editor = create_editor_with_editor_content(vec!["Hello Denmark".to_string()]);
        editor.text_selection_start = Some(CursorPosition { x: 6, y: 0 });
        editor.text_selection_end = Some(CursorPosition { x: 13, y: 0 });
        editor.cursor.x = 6;
        editor.write_char_text_is_selected('W');
        assert_eq!(editor.editor_content[0], "Hello W");
        assert_eq!(editor.cursor.x, 7);
    }

    #[test]
    fn test_write_char_with_selected_text_multiple_lines() {
        let mut editor = create_editor_with_editor_content(vec![
            "test".to_string(),
            "Hello Denmark".to_string(),
            "Hello Sudetenland".to_string(),
        ]);
        editor.text_selection_start = Some(CursorPosition { x: 6, y: 1 });
        editor.text_selection_end = Some(CursorPosition { x: 13, y: 2 });
        editor.cursor.x = 6;
        editor.write_char_text_is_selected('W');
        assert_eq!(editor.editor_content[0], "test");
        assert_eq!(editor.editor_content[1], "Hello Wland");
        assert_eq!(editor.cursor.x, 7);
    }

    #[test]
    fn test_backspace_in_editor_text_is_selected_multiple_lines_4lines_middle_selected() {
        // Initialize the editor with some content
        let mut editor = create_editor_with_editor_content(vec![
            "first line".to_string(),
            "test".to_string(),
            "Hello Denmark".to_string(),
            "Hello Sudetenland".to_string(),
        ]);

        // Set a selection range (e.g., "Denmark")
        editor.text_selection_start = Some(CursorPosition { x: 2, y: 1 }); // middle of "test"
        editor.text_selection_end = Some(CursorPosition { x: 13, y: 3 }); // End of "sudeten"
                                                                          // Call the function to simulate a backspace with text selected
        editor.backspace_text_is_selected();

        assert_eq!(editor.editor_content.len(), 2);

        assert_eq!(editor.editor_content[0], "first line");
        // Assert that the selected text is removed
        assert_eq!(editor.editor_content[1], "teland");

        // Assert that the selection is cleared after the operation
        assert!(editor.text_selection_start.is_none());
        assert!(editor.text_selection_end.is_none());

        // Assert that the cursor is moved to the correct position
        assert_eq!(editor.cursor.x, 2);
        assert_eq!(editor.cursor.y, 1);
    }

    #[test]
    fn test_write_char_with_selected_text_special_characters() {
        let mut editor = create_editor_with_editor_content(vec!["ᚠΩ₿😎".to_string()]);
        editor.text_selection_start = Some(CursorPosition { x: 1, y: 0 });
        editor.text_selection_end = Some(CursorPosition { x: 2, y: 0 });
        editor.cursor.x = 1;

        editor.write_char_text_is_selected('a');
        assert_eq!(editor.editor_content[0], "ᚠa₿😎");
        assert_eq!(editor.cursor.x, 2);
    }

    //TAB in editor
    #[test]
    fn test_tab_in_editor_start_of_empty_line() {
        let mut editor = create_editor_with_editor_content(vec!["".to_string()]);
        editor.tab();

        assert_eq!(editor.cursor.y, 0); // Cursor should stay on line
        assert_eq!(editor.editor_content.len(), 1); // New line added
        assert_eq!(editor.visual_cursor_x, editor_settings::TAB_WIDTH as i16);
    }

    #[test]
    fn test_tab_in_editor_start_of_line() {
        let mut editor = create_editor_with_editor_content(vec!["HELLO WORLD".to_string()]);
        editor.tab();

        assert_eq!(editor.cursor.y, 0); // Cursor should stay on line
        assert_eq!(editor.editor_content.len(), 1); // New line added
        assert_eq!(editor.visual_cursor_x, editor_settings::TAB_WIDTH as i16);
    }

    #[test]
    fn test_tab_in_editor_mid_of_line_normal_characters() {
        let mut editor = create_editor_with_editor_content(vec!["1234".to_string()]);
        editor.cursor.x = 2;
        editor.tab();

        assert_eq!(editor.cursor.y, 0); // Cursor should stay on line
        assert_eq!(editor.editor_content.len(), 1); // New line added
        assert_eq!(editor.visual_cursor_x, 4);
        editor.move_cursor(10, 0); //move to end
        assert_eq!(editor.editor_content[0].chars().count(), 5); //should contain special plus \t char
        assert_eq!(editor.visual_cursor_x, 6); //at end of line should be 6
    }

    #[test]
    fn test_tab_in_editor_mid_of_line_special_characters() {
        let mut editor = create_editor_with_editor_content(vec!["ᚠΩ₿😎".to_string()]);
        editor.cursor.x = 2;
        editor.tab();

        assert_eq!(editor.cursor.y, 0); // Cursor should stay on line
        assert_eq!(editor.editor_content.len(), 1); // New line added
        assert_eq!(editor.visual_cursor_x, 4);
        editor.move_cursor(10, 0); //move to end
        assert_eq!(editor.editor_content[0].chars().count(), 5); //should contain special plus \t char
        assert_eq!(editor.visual_cursor_x, 6); //at end of line should be 6
    }

    //ENTER in editor

    #[test]
    fn test_enter_in_editor_at_end_of_line() {
        let mut editor = create_editor_with_editor_content(vec!["Hello World".to_string()]);
        editor.cursor.x = editor.editor_content[0].len() as i16; // Set cursor to end of line
        editor.enter();

        assert_eq!(editor.cursor.y, 1); // Cursor should move to the next line
        assert_eq!(editor.editor_content.len(), 2); // New line added
        assert_eq!(editor.editor_content[1], ""); // New line should be empty
    }

    #[test]
    fn test_enter_in_editor_mid_line() {
        let mut editor = create_editor_with_editor_content(vec!["Hello World".to_string()]);
        editor.cursor.x = 5; // Split the line at index 5
        editor.enter();

        assert_eq!(editor.cursor.y, 1); // Cursor should move to next line
        assert_eq!(editor.cursor.x, 0); // Cursor resets to start of new line
        assert_eq!(editor.editor_content[0], "Hello"); // Line before cursor is kept intact
        assert_eq!(editor.editor_content[1], " World"); // Line after cursor is moved to new line
    }
}
#[cfg(test)]
mod unit_editor_delete_tests {
    use super::super::super::cursor::CursorPosition;
    use super::super::editor::*;

    fn create_editor_with_editor_content(vec: Vec<String>) -> Editor {
        let mut editor = Editor::new();
        editor.editor_content = vec;
        editor.editor_height = 10; //since testing doesnt start ui.rs, height isnt set
        editor
    }

    //BACKSPACE IN EDITOR
    #[test]
    fn test_backspace_in_editor() {
        let mut editor = create_editor_with_editor_content(vec!['a'.to_string()]);
        editor.cursor.x = 1;
        editor.backspace();
        assert_eq!(editor.editor_content[0], "");
        assert_eq!(editor.cursor.x, 0);
    }

    #[test]
    fn test_backspace_in_editor_special_characters() {
        let mut editor = create_editor_with_editor_content(vec!["ᚠΩ₿😎".to_string()]);
        editor.cursor.x = 4;
        editor.backspace();
        assert_eq!(editor.editor_content[0], "ᚠΩ₿");
        assert_eq!(editor.cursor.x, 3);
    }

    #[test]
    fn test_backspace_in_editor_should_go_to_previous_line() {
        let mut editor = create_editor_with_editor_content(vec!["a".to_string(), "b".to_string()]);
        editor.cursor.y = 1;
        editor.cursor.x = 0;
        editor.backspace();
        assert_eq!(editor.editor_content[0], "ab");
        assert_eq!(editor.editor_content.len(), 1);
        assert_eq!(editor.cursor.x, 1);
        assert_eq!(editor.cursor.y, 0);
    }

    //TEXT IS SELECTED

    #[test]
    fn test_backspace_in_editor_text_is_selected() {
        // Initialize the editor with some content
        let mut editor = create_editor_with_editor_content(vec!["Hello Denmark".to_string()]);

        // Set a selection range (e.g., "Denmark")
        editor.text_selection_start = Some(CursorPosition { x: 6, y: 0 }); // Start of "Denmark"
        editor.text_selection_end = Some(CursorPosition { x: 13, y: 0 }); // End of "Denmark"
                                                                          // Call the function to simulate a backspace with text selected
        editor.backspace_text_is_selected();

        // Assert that the selected text is removed
        assert_eq!(editor.editor_content, vec!["Hello "]);

        // Assert that the selection is cleared after the operation
        assert!(editor.text_selection_start.is_none());
        assert!(editor.text_selection_end.is_none());

        // Assert that the cursor is moved to the correct position
        assert_eq!(editor.cursor.x, 6);
        assert_eq!(editor.cursor.y, 0);
    }

    #[test]
    fn test_backspace_in_editor_text_is_selected_multiple_lines() {
        // Initialize the editor with some content
        let mut editor = create_editor_with_editor_content(vec![
            "test".to_string(),
            "Hello Denmark".to_string(),
            "Hello Sudetenland".to_string(),
        ]);

        // Set a selection range (e.g., "Denmark")
        editor.text_selection_start = Some(CursorPosition { x: 6, y: 1 }); // Start of "Denmark"
        editor.text_selection_end = Some(CursorPosition { x: 13, y: 2 }); // End of "sudeten"
                                                                          // Call the function to simulate a backspace with text selected
        editor.backspace_text_is_selected();

        assert_eq!(editor.editor_content.len(), 2);

        // Assert that the selected text is removed
        assert_eq!(editor.editor_content[0], "test");
        assert_eq!(editor.editor_content[1], "Hello land");

        // Assert that the selection is cleared after the operation
        assert!(editor.text_selection_start.is_none());
        assert!(editor.text_selection_end.is_none());

        // Assert that the cursor is moved to the correct position
        assert_eq!(editor.cursor.x, 6);
        assert_eq!(editor.cursor.y, 1);
    }

    #[test]
    fn test_backspace_in_editor_text_is_selected_multiple_lines_4lines_middle_selected() {
        // Initialize the editor with some content
        let mut editor = create_editor_with_editor_content(vec![
            "first line".to_string(),
            "test".to_string(),
            "Hello Denmark".to_string(),
            "Hello Sudetenland".to_string(),
        ]);

        // Set a selection range (e.g., "Denmark")
        editor.text_selection_start = Some(CursorPosition { x: 2, y: 1 }); // middle of "test"
        editor.text_selection_end = Some(CursorPosition { x: 13, y: 3 }); // End of "sudeten"
                                                                          // Call the function to simulate a backspace with text selected
        editor.backspace_text_is_selected();

        assert_eq!(editor.editor_content.len(), 2);

        // Assert that the selected text is removed
        assert_eq!(editor.editor_content[1], "teland");

        // Assert that the selection is cleared after the operation
        assert!(editor.text_selection_start.is_none());
        assert!(editor.text_selection_end.is_none());

        // Assert that the cursor is moved to the correct position
        assert_eq!(editor.cursor.x, 2);
        assert_eq!(editor.cursor.y, 1);
    }

    #[test]
    fn test_backspace_in_editor_text_is_selected_empty_text() {
        // Initialize the editor with empty content
        let mut editor = create_editor_with_editor_content(vec!["".to_string()]);

        // Set a selection range (even though the text is empty)
        editor.text_selection_start = Some(CursorPosition { x: 0, y: 0 });
        editor.text_selection_end = Some(CursorPosition { x: 0, y: 0 });

        // Call the function to simulate a backspace with empty text
        editor.backspace_text_is_selected();

        // Assert that the text remains empty
        assert_eq!(editor.editor_content, vec!["".to_string()]);

        // Assert that the selection is cleared
        assert!(editor.text_selection_start.is_none());
        assert!(editor.text_selection_end.is_none());

        // Assert that the cursor position is 0
        assert_eq!(editor.cursor.x, 0);
        assert_eq!(editor.cursor.y, 0);
    }

    #[test]
    fn test_backspace_in_editor_text_is_selected_full_text_selected() {
        // Initialize the editor with some content
        let mut editor = create_editor_with_editor_content(vec!["Hello Denmark".to_string()]);

        // Set a selection range for the entire text
        editor.text_selection_start = Some(CursorPosition { x: 0, y: 0 });
        editor.text_selection_end = Some(CursorPosition { x: 13, y: 0 });

        // Call the function to simulate a backspace with the entire text selected
        editor.backspace_text_is_selected();

        // Assert that all text is removed
        assert_eq!(editor.editor_content, vec!["".to_string()]);

        // Assert that the selection is cleared
        assert!(editor.text_selection_start.is_none());
        assert!(editor.text_selection_end.is_none());

        // Assert that the cursor position is 0
        assert_eq!(editor.cursor.x, 0);
        assert_eq!(editor.cursor.y, 0);
    }

    //DELETE IN EDITOR
    #[test]
    fn test_delete_in_editor() {
        let mut editor = create_editor_with_editor_content(vec!["ab".to_string()]);
        editor.cursor.x = 0;
        editor.delete();
        assert_eq!(editor.editor_content[0], "a");
        assert_eq!(editor.cursor.x, 0);
    }

    #[test]
    fn test_delete_in_editor_special_characters() {
        let mut editor = create_editor_with_editor_content(vec!["ᚠΩ₿😎".to_string()]);
        editor.cursor.x = 2;
        editor.delete();
        assert_eq!(editor.editor_content[0], "ᚠΩ₿");
        assert_eq!(editor.cursor.x, 2);
    }

    #[test]
    fn test_delete_in_editor_should_go_to_previous_line() {
        let mut editor = create_editor_with_editor_content(vec!["a".to_string(), "b".to_string()]);
        editor.cursor.x = 1;
        editor.delete();
        assert_eq!(editor.editor_content[0], "ab");
        assert_eq!(editor.editor_content.len(), 1);
        assert_eq!(editor.cursor.x, 1);
    }

    //TEXT IS SELECTED

    #[test]
    fn test_delete_in_editor_text_is_selected() {
        // Initialize the editor with some content
        let mut editor = create_editor_with_editor_content(vec!["Hello Denmark".to_string()]);

        // Set a selection range (e.g., "Denmark")
        editor.text_selection_start = Some(CursorPosition { x: 6, y: 0 }); // Start of "Denmark"
        editor.text_selection_end = Some(CursorPosition { x: 13, y: 0 }); // End of "Denmark"
                                                                          // Call the function to simulate a backspace with text selected
        editor.delete_text_is_selected();

        // Assert that the selected text is removed
        assert_eq!(editor.editor_content[0], "Hello        ");
        assert_eq!(editor.editor_content[0].len(), 13);

        // Assert that the selection is cleared after the operation
        assert!(editor.text_selection_start.is_none());
        assert!(editor.text_selection_end.is_none());

        // Assert that the cursor is moved to the correct position
        assert_eq!(editor.cursor.x, 13);
        assert_eq!(editor.cursor.y, 0);
    }

    #[test]
    fn test_delete_in_editor_text_is_selected_multiple_lines() {
        // Initialize the editor with some content
        let mut editor = create_editor_with_editor_content(vec![
            "test".to_string(),
            "Hello Denmark".to_string(),
            "Hello Sudetenland".to_string(),
        ]);

        // Set a selection range (e.g., "Denmark")
        editor.text_selection_start = Some(CursorPosition { x: 6, y: 1 }); // Start of "Denmark"
        editor.text_selection_end = Some(CursorPosition { x: 13, y: 2 }); // End of "Denmark"
                                                                          // Call the function to simulate a backspace with text selected
        editor.delete_text_is_selected();

        assert_eq!(editor.editor_content.len(), 3);

        // Assert that the selected text is removed
        assert_eq!(editor.editor_content[0], "test");
        assert_eq!(editor.editor_content[1], "Hello ");
        assert_eq!(editor.editor_content[2].len(), 17);

        // Assert that the selection is cleared after the operation
        assert!(editor.text_selection_start.is_none());
        assert!(editor.text_selection_end.is_none());

        // Assert that the cursor is moved to the correct position
        assert_eq!(editor.cursor.x, 13);
        assert_eq!(editor.cursor.y, 2);
    }

    #[test]
    fn test_delete_in_editor_text_is_selected_multiple_lines_4lines_middle_selected() {
        // Initialize the editor with some content
        let mut editor = create_editor_with_editor_content(vec![
            "first line".to_string(),
            "test".to_string(),
            "Hello Denmark".to_string(),
            "Hello Sudetenland".to_string(),
        ]);

        // Set a selection range (e.g., "Denmark")
        editor.text_selection_start = Some(CursorPosition { x: 2, y: 1 }); // middle of "test"
        editor.text_selection_end = Some(CursorPosition { x: 13, y: 3 }); // End of "sudeten"
                                                                          // Call the function to simulate a backspace with text selected
        editor.delete_text_is_selected();

        assert_eq!(editor.editor_content.len(), 4);

        // Assert that the selected text is removed
        assert_eq!(editor.editor_content[1], "te");
        assert_eq!(editor.editor_content[2], "");
        assert_eq!(editor.editor_content[3], "             land");

        // Assert that the selection is cleared after the operation
        assert!(editor.text_selection_start.is_none());
        assert!(editor.text_selection_end.is_none());

        // Assert that the cursor is moved to the correct position
        assert_eq!(editor.cursor.x, 13);
        assert_eq!(editor.cursor.y, 3);
    }

    #[test]
    fn test_delete_in_editor_text_is_selected_empty_text() {
        // Initialize the editor with empty content
        let mut editor = create_editor_with_editor_content(vec!["".to_string()]);

        // Set a selection range (even though the text is empty)
        editor.text_selection_start = Some(CursorPosition { x: 0, y: 0 });
        editor.text_selection_end = Some(CursorPosition { x: 0, y: 0 });

        // Call the function to simulate a backspace with empty text
        editor.delete_text_is_selected();

        // Assert that the text remains empty
        assert_eq!(editor.editor_content, vec!["".to_string()]);

        // Assert that the selection is cleared
        assert!(editor.text_selection_start.is_none());
        assert!(editor.text_selection_end.is_none());

        // Assert that the cursor position is 0
        assert_eq!(editor.cursor.x, 0);
        assert_eq!(editor.cursor.y, 0);
    }

    #[test]
    fn test_delete_in_editor_text_is_selected_full_text_selected() {
        // Initialize the editor with some content
        let mut editor = create_editor_with_editor_content(vec!["Hello Denmark".to_string()]);

        // Set a selection range for the entire text
        editor.text_selection_start = Some(CursorPosition { x: 0, y: 0 });
        editor.text_selection_end = Some(CursorPosition { x: 13, y: 0 });

        // Call the function to simulate a backspace with the entire text selected
        editor.delete_text_is_selected();

        // Assert that all text is removed
        assert_eq!(editor.editor_content[0].len(), 13);

        // Assert that the selection is cleared
        assert!(editor.text_selection_start.is_none());
        assert!(editor.text_selection_end.is_none());

        // Assert that the cursor position is 0
        assert_eq!(editor.cursor.x, 13);
        assert_eq!(editor.cursor.y, 0);
    }
}

#[cfg(test)]
mod unit_editor_cursor_tests {
    use super::super::editor::*;

    fn create_editor_with_editor_content(vec: Vec<String>) -> Editor {
        let mut editor = Editor::new();
        editor.editor_content = vec;
        editor.editor_height = 10; //since testing doesnt start ui.rs, height isnt set
        editor
    }

    #[test]
    fn test_cursor_move_right_within_line() {
        let mut editor = create_editor_with_editor_content(vec!["Hello World".to_string()]);
        editor.move_cursor(1, 0);

        assert_eq!(editor.cursor.x, 1);
        assert_eq!(editor.cursor.y, 0);
    }

    #[test]
    fn test_cursor_move_left_at_start_should_stay() {
        let mut editor = create_editor_with_editor_content(vec!["Hello World".to_string()]);
        editor.move_cursor(-1, 0);

        assert_eq!(editor.cursor.x, 0);
        assert_eq!(editor.cursor.y, 0);
    }

    #[test]
    fn test_cursor_move_right_within_empty_line_should_stay() {
        let mut editor = create_editor_with_editor_content(vec![]);
        editor.move_cursor(1, 0);

        assert_eq!(editor.cursor.x, 0);
        assert_eq!(editor.cursor.y, 0);
    }

    #[test]
    fn test_cursor_move_right_at_end_of_first_line_should_move_down() {
        let mut editor =
            create_editor_with_editor_content(vec!["First".to_string(), "Second".to_string()]);
        editor.cursor.x = 5;
        editor.move_cursor(1, 0);

        assert_eq!(editor.cursor.x, 0);
        assert_eq!(editor.cursor.y, 1);
    }

    #[test]
    fn test_cursor_move_right_at_end_of_first_line_should_stay() {
        let mut editor = create_editor_with_editor_content(vec!["First".to_string()]);
        editor.cursor.x = 5;
        editor.move_cursor(1, 0);

        assert_eq!(editor.cursor.x, 5);
        assert_eq!(editor.cursor.y, 0);
    }

    #[test]
    fn test_cursor_move_down() {
        let mut editor = create_editor_with_editor_content(vec!["Second Line".to_string()]);
        editor.move_cursor(0, 1);

        assert_eq!(editor.cursor.x, 0); // Cursor stays at column 0
        assert_eq!(editor.cursor.y, 1); // Moves to the second line
    }

    //SELECTION CURSOR

    #[test]
    fn test_selection_cursor_move_up_should_stay() {
        let mut editor = create_editor_with_editor_content(vec![]);
        editor.move_selection_cursor(0, -1);

        assert_eq!(editor.text_selection_start.unwrap().x, 0);
        assert_eq!(editor.text_selection_start.unwrap().y, 0);
        assert_eq!(editor.text_selection_end.unwrap().x, 0);
        assert_eq!(editor.text_selection_end.unwrap().y, 0);
    }

    #[test]
    fn test_selection_cursor_move_down_go_down() {
        let mut editor = create_editor_with_editor_content(vec![]);
        editor.move_selection_cursor(0, 1);

        assert_eq!(editor.text_selection_start.unwrap().x, 0);
        assert_eq!(editor.text_selection_start.unwrap().y, 0);
        assert_eq!(editor.text_selection_end.unwrap().x, 0);
        assert_eq!(editor.text_selection_end.unwrap().y, 1);
    }

    #[test]
    fn test_selection_cursor_move_left_should_stay() {
        let mut editor = create_editor_with_editor_content(vec![]);
        editor.move_selection_cursor(-1, 0);

        assert_eq!(editor.text_selection_start.unwrap().x, 0);
        assert_eq!(editor.text_selection_start.unwrap().y, 0);
        assert_eq!(editor.text_selection_end.unwrap().x, 0);
        assert_eq!(editor.text_selection_end.unwrap().y, 0);
    }

    #[test]
    fn test_selection_cursor_move_right_should_stay() {
        let mut editor = create_editor_with_editor_content(vec![]);
        editor.move_selection_cursor(1, 0);

        assert_eq!(editor.text_selection_start.unwrap().x, 0);
        assert_eq!(editor.text_selection_start.unwrap().y, 0);
        assert_eq!(editor.text_selection_end.unwrap().x, 0);
        assert_eq!(editor.text_selection_end.unwrap().y, 0);
    }

    #[test]
    fn test_selection_cursor_move_up_should_go_up() {
        let mut editor =
            create_editor_with_editor_content(vec!["First".to_string(), "Second".to_string()]);
        editor.cursor.y = 1;
        editor.move_selection_cursor(0, -1);

        assert_eq!(editor.text_selection_start.unwrap().x, 0);
        assert_eq!(editor.text_selection_start.unwrap().y, 0);
        assert_eq!(editor.text_selection_end.unwrap().x, 0);
        assert_eq!(editor.text_selection_end.unwrap().y, 1);
    }

    #[test]
    fn test_selection_cursor_move_down_should_go_down() {
        let mut editor =
            create_editor_with_editor_content(vec!["First".to_string(), "Second".to_string()]);
        editor.move_selection_cursor(0, 1);

        assert_eq!(editor.text_selection_start.unwrap().x, 0);
        assert_eq!(editor.text_selection_start.unwrap().y, 0);
        assert_eq!(editor.text_selection_end.unwrap().x, 0);
        assert_eq!(editor.text_selection_end.unwrap().y, 1);
    }

    #[test]
    fn test_selection_cursor_move_left_should_go_left() {
        let mut editor = create_editor_with_editor_content(vec!["First".to_string()]);
        editor.cursor.x = 1;
        editor.move_selection_cursor(-1, 0);

        assert_eq!(editor.text_selection_start.unwrap().x, 0);
        assert_eq!(editor.text_selection_start.unwrap().y, 0);
        assert_eq!(editor.text_selection_end.unwrap().x, 1);
        assert_eq!(editor.text_selection_end.unwrap().y, 0);
    }

    #[test]
    fn test_selection_cursor_move_right_should_go_right() {
        let mut editor = create_editor_with_editor_content(vec!["First".to_string()]);
        editor.move_selection_cursor(1, 0);

        assert_eq!(editor.text_selection_start.unwrap().x, 0);
        assert_eq!(editor.text_selection_start.unwrap().y, 0);
        assert_eq!(editor.text_selection_end.unwrap().x, 1);
        assert_eq!(editor.text_selection_end.unwrap().y, 0);
    }

    #[test]
    fn test_selection_cursor_move_right_thrice_should_go_right() {
        let mut editor = create_editor_with_editor_content(vec!["First".to_string()]);
        editor.move_selection_cursor(1, 0);
        editor.move_selection_cursor(1, 0);
        editor.move_selection_cursor(1, 0);

        assert_eq!(editor.text_selection_start.unwrap().x, 0);
        assert_eq!(editor.text_selection_start.unwrap().y, 0);
        assert_eq!(editor.text_selection_end.unwrap().x, 3);
        assert_eq!(editor.text_selection_end.unwrap().y, 0);
    }
}
#[cfg(test)]
mod unit_editor_cutcopy_tests {
    use crate::core::cursor::CursorPosition;
    use crate::core::editor::Editor;

    fn create_editor_with_editor_content(vec: Vec<String>) -> Editor {
        let mut editor = Editor::new();
        editor.editor_content = vec;
        editor.editor_height = 10; //since testing doesnt start ui.rs, height isnt set
        editor
    }

    //copy selected text
    #[test]
    fn test_copy_single_line_selection() {
        let mut app = create_editor_with_editor_content(vec!["Hello, world!".to_string()]);
        app.text_selection_start = Some(CursorPosition { x: 7, y: 0 });
        app.text_selection_end = Some(CursorPosition { x: 12, y: 0 });

        let result = app.copy();

        assert!(result.is_ok());
        assert_eq!(app.clipboard.copied_text, vec!["world".to_string()]);
    }

    #[test]
    fn test_copy_multi_line_selection() {
        let mut app = create_editor_with_editor_content(vec![
            "Hello,".to_string(),
            " world!".to_string(),
            " Rust".to_string(),
        ]);
        app.text_selection_start = Some(CursorPosition { x: 4, y: 0 });
        app.text_selection_end = Some(CursorPosition { x: 3, y: 2 });

        let result = app.copy();

        assert!(result.is_ok());
        assert_eq!(
            app.clipboard.copied_text,
            vec!["o,", " world!", " Ru"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<String>>()
        );
    }

    #[test]
    fn test_copy_no_selection() {
        let mut app = create_editor_with_editor_content(vec!["Hello, world!".to_string()]);
        app.text_selection_start = None;
        app.text_selection_end = None;

        let result = app.copy();

        assert!(result.is_err());
        assert!(app.clipboard.copied_text.is_empty());
    }

    //cut selected text
    #[test]
    fn test_cut_single_line_selection() {
        let mut app = create_editor_with_editor_content(vec!["Hello, world!".to_string()]);
        app.text_selection_start = Some(CursorPosition { x: 7, y: 0 });
        app.text_selection_end = Some(CursorPosition { x: 12, y: 0 });

        let result = app.cut();

        assert!(result.is_ok());
        assert_eq!(app.clipboard.copied_text, vec!["world".to_string()]);
        assert!(app.text_selection_start.is_none());
        assert!(app.text_selection_end.is_none());
    }

    #[test]
    fn test_cut_multi_line_selection() {
        let mut app = create_editor_with_editor_content(vec![
            "Hello,".to_string(),
            " world!".to_string(),
            " Rust".to_string(),
        ]);
        app.text_selection_start = Some(CursorPosition { x: 4, y: 0 });
        app.text_selection_end = Some(CursorPosition { x: 3, y: 2 });

        let result = app.cut();

        assert!(result.is_ok());
        assert_eq!(
            app.clipboard.copied_text,
            vec!["o,", " world!", " Ru"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<String>>()
        );
        assert!(app.text_selection_start.is_none());
        assert!(app.text_selection_end.is_none());
    }

    #[test]
    fn test_cut_no_selection() {
        let mut app = create_editor_with_editor_content(vec!["Hello, world!".to_string()]);
        app.text_selection_start = None;
        app.text_selection_end = None;

        let result = app.cut();

        assert!(result.is_err());
        assert!(app.clipboard.copied_text.is_empty());
        assert!(app.text_selection_start.is_none());
        assert!(app.text_selection_end.is_none());
    }

    //paste selected text
    #[test]
    fn test_paste_single_line() {
        let mut app = create_editor_with_editor_content(vec![
            "Hello, world!".to_string(),
            "This is a test.".to_string(),
            "Another line.".to_string(),
        ]);
        app.clipboard.copy(&vec!["PASTED".to_string()]);
        app.cursor.x = 8;
        app.cursor.y = 0;

        app.paste().unwrap();
        assert_eq!(
            app.editor_content,
            vec![
                "Hello, wPASTEDorld!".to_string(),
                "This is a test.".to_string(),
                "Another line.".to_string(),
            ]
        );
    }

    #[test]
    fn test_paste_multiline() {
        let mut app = create_editor_with_editor_content(vec![
            "Hello, world!".to_string(),
            "This is a test.".to_string(),
            "Another line.".to_string(),
        ]);
        app.clipboard
            .copy(&vec!["First".to_string(), "Second ".to_string()]);
        app.cursor.x = 5;
        app.cursor.y = 1;

        app.paste().unwrap();
        assert_eq!(
            app.editor_content,
            vec![
                "Hello, world!".to_string(),
                "This First".to_string(),
                "Second is a test.".to_string(),
                "Another line.".to_string(),
            ]
        );
    }

    #[test]
    fn test_paste_single_line_special_characters() {
        let mut app = create_editor_with_editor_content(vec![
            "Hello, wᚠᚠᚠᚠorld!".to_string(),
            "This is a test.".to_string(),
            "Another line.".to_string(),
        ]);
        app.clipboard.copy(&vec!["PASTED".to_string()]);
        app.cursor.x = 10;
        app.cursor.y = 0;

        app.paste().unwrap();
        assert_eq!(
            app.editor_content,
            vec![
                "Hello, wᚠᚠPASTEDᚠᚠorld!".to_string(),
                "This is a test.".to_string(),
                "Another line.".to_string(),
            ]
        );
    }

    #[test]
    fn test_paste_multiline_special_charaters() {
        let mut app = create_editor_with_editor_content(vec![
            "Hello, world!".to_string(),
            "This ᚠᚠᚠᚠis a test.".to_string(),
            "Another line.".to_string(),
        ]);
        app.clipboard
            .copy(&vec!["First".to_string(), "Second ".to_string()]);
        app.cursor.x = 7;
        app.cursor.y = 1;

        app.paste().unwrap();
        assert_eq!(
            app.editor_content,
            vec![
                "Hello, world!".to_string(),
                "This ᚠᚠFirst".to_string(),
                "Second ᚠᚠis a test.".to_string(),
                "Another line.".to_string(),
            ]
        );
    }

    #[test]
    fn test_paste_at_start_of_line() {
        let mut app = create_editor_with_editor_content(vec![
            "Hello, world!".to_string(),
            "This is a test.".to_string(),
            "Another line.".to_string(),
        ]);
        app.clipboard.copy(&vec!["NewStart".to_string()]);
        app.cursor.x = 0;
        app.cursor.y = 2;

        app.paste().unwrap();

        assert_eq!(
            app.editor_content,
            vec![
                "Hello, world!".to_string(),
                "This is a test.".to_string(),
                "NewStartAnother line.".to_string(),
            ]
        );
    }

    #[test]
    fn test_paste_at_end_of_line() {
        let mut app = create_editor_with_editor_content(vec![
            "Hello, world!".to_string(),
            "This is a test.".to_string(),
            "Another line.".to_string(),
        ]);
        app.clipboard.copy(&vec!["END".to_string()]);
        app.cursor.x = 13;
        app.cursor.y = 0;

        app.paste().unwrap();
        assert_eq!(
            app.editor_content,
            vec![
                "Hello, world!END".to_string(),
                "This is a test.".to_string(),
                "Another line.".to_string(),
            ]
        );
    }

    #[test]
    fn test_paste_with_empty_copied_text() {
        let mut app = create_editor_with_editor_content(vec![
            "Hello, world!".to_string(),
            "This is a test.".to_string(),
            "Another line.".to_string(),
        ]);
        app.clipboard.copy(&vec![]);
        app.cursor.x = 5;
        app.cursor.y = 1;

        assert!(app.paste().is_err());
        assert_eq!(
            app.editor_content,
            vec![
                "Hello, world!".to_string(),
                "This is a test.".to_string(),
                "Another line.".to_string(),
            ]
        );
    }

    #[test]
    fn test_paste_into_empty_editor() {
        let mut app = create_editor_with_editor_content(vec![]);
        app.clipboard
            .copy(&vec!["Hello".to_string(), "World".to_string()]);

        app.paste().unwrap();
        assert_eq!(
            app.editor_content,
            vec!["Hello".to_string(), "World".to_string()]
        );
    }
}
#[cfg(test)]
mod unit_editor_undoredo_tests {
    use super::super::super::cursor::CursorPosition;
    use super::super::editor::EditAction;
    use super::super::editor::Editor;

    //init functions
    fn create_editor_with_editor_content(vec: Vec<String>) -> Editor {
        let mut editor = Editor::new();
        editor.editor_content = vec;
        editor.editor_height = 10; //since testing doesnt start ui.rs, height isnt set
        editor
    }
    // ========== Insert ==========
    #[test]
    fn undo_redo_insert_at_start() {
        let mut editor = create_editor_with_editor_content(vec!["xyz".to_string()]);
        editor.cursor.x = 0;
        editor.write_char('A');
        assert_eq!(editor.editor_content[0], "Axyz");
        editor.undo().unwrap();
        assert_eq!(editor.editor_content[0], "xyz");
        editor.redo().unwrap();
        assert_eq!(editor.editor_content[0], "Axyz");
    }

    #[test]
    fn undo_redo_insert_at_end() {
        let mut editor = create_editor_with_editor_content(vec!["foo".to_string()]);
        editor.cursor.x = 3;
        editor.write_char('B');
        assert_eq!(editor.editor_content[0], "fooB");
        editor.undo().unwrap();
        assert_eq!(editor.editor_content[0], "foo");
        editor.redo().unwrap();
        assert_eq!(editor.editor_content[0], "fooB");
    }

    #[test]
    fn undo_redo_multiple_insert_sequence() {
        let mut editor = create_editor_with_editor_content(vec!["".to_string()]);
        for ch in ['h', 'e', 'l', 'l', 'o'] {
            editor.write_char(ch);
        }
        assert_eq!(editor.editor_content[0], "hello");
        for _ in 0..5 {
            editor.undo().unwrap();
        }
        assert_eq!(editor.editor_content[0], "");
        for _ in 0..5 {
            editor.redo().unwrap();
        }
        assert_eq!(editor.editor_content[0], "hello");
    }

    // ========== Delete ==========
    #[test]
    fn undo_redo_delete_middle_char() {
        let mut editor = create_editor_with_editor_content(vec!["abcde".to_string()]);
        editor.cursor.x = 3;
        editor.backspace(); // remove 'c'
        assert_eq!(editor.editor_content[0], "abde");
        editor.undo().unwrap();
        assert_eq!(editor.editor_content[0], "abcde");
        editor.redo().unwrap();
        assert_eq!(editor.editor_content[0], "abde");
    }

    #[test]
    fn undo_redo_delete_last_char() {
        let mut editor = create_editor_with_editor_content(vec!["test".to_string()]);
        editor.cursor.x = 4;
        editor.backspace(); // remove 't'
        assert_eq!(editor.editor_content[0], "tes");
        editor.undo().unwrap();
        assert_eq!(editor.editor_content[0], "test");
        editor.redo().unwrap();
        assert_eq!(editor.editor_content[0], "tes");
    }

    #[test]
    fn undo_redo_delete_first_char() {
        let mut editor = create_editor_with_editor_content(vec!["tak".to_string()]);
        editor.cursor.x = 1;
        editor.backspace(); // remove 't'
        assert_eq!(editor.editor_content[0], "ak");
        editor.undo().unwrap();
        assert_eq!(editor.editor_content[0], "tak");
        editor.redo().unwrap();
        assert_eq!(editor.editor_content[0], "ak");
    }

    // ========== Replace ==========
    #[test]
    fn undo_redo_single_replace() {
        let mut editor = create_editor_with_editor_content(vec!["foo".to_string()]);
        // Simulate replace: overwrite 'o' (at 2..3) with 'x'
        let start = CursorPosition { x: 2, y: 0 };
        let end = CursorPosition { x: 3, y: 0 };
        let old = 'o';
        let new = 'x';
        editor.undo_redo_manager.record_undo(EditAction::Replace {
            start,
            end,
            old: old.clone(),
            new: new.clone(),
        });
        editor.editor_content[0].replace_range(2..3, "x");
        assert_eq!(editor.editor_content[0], "fox");
        editor.undo().unwrap();
        assert_eq!(editor.editor_content[0], "foo");
        editor.redo().unwrap();
        assert_eq!(editor.editor_content[0], "fox");
    }

    // ========== InsertLines ==========
    #[test]
    fn undo_redo_insert_lines_middle() {
        let mut editor =
            create_editor_with_editor_content(vec!["zero".to_string(), "three".to_string()]);
        let lines = vec!["one".to_string(), "two".to_string()];
        let pos = CursorPosition { x: 0, y: 1 };
        editor
            .undo_redo_manager
            .record_undo(EditAction::InsertLines {
                start: pos,
                lines: lines.clone(),
            });
        editor.editor_content.splice(1..1, lines.clone());
        assert_eq!(editor.editor_content, vec!["zero", "one", "two", "three"]);
        editor.undo().unwrap();
        assert_eq!(editor.editor_content, vec!["zero", "three"]);
        editor.redo().unwrap();
        assert_eq!(editor.editor_content, vec!["zero", "one", "two", "three"]);
    }

    #[test]
    fn undo_redo_insert_lines_at_start_and_end() {
        let mut editor = create_editor_with_editor_content(vec!["mid".to_string()]);
        let start_lines = vec!["a".to_string(), "b".to_string()];
        let end_lines = vec!["x".to_string(), "y".to_string()];
        // Insert at start
        let pos_start = CursorPosition { x: 0, y: 0 };
        editor
            .undo_redo_manager
            .record_undo(EditAction::InsertLines {
                start: pos_start,
                lines: start_lines.clone(),
            });
        editor.editor_content.splice(0..0, start_lines.clone());
        // Insert at end
        let pos_end = CursorPosition { x: 0, y: 3 };
        editor
            .undo_redo_manager
            .record_undo(EditAction::InsertLines {
                start: pos_end,
                lines: end_lines.clone(),
            });
        editor.editor_content.splice(3..3, end_lines.clone());
        assert_eq!(editor.editor_content, vec!["a", "b", "mid", "x", "y"]);
        editor.undo().unwrap();
        assert_eq!(editor.editor_content, vec!["a", "b", "mid"]);
        editor.undo().unwrap();
        assert_eq!(editor.editor_content, vec!["mid"]);
        editor.redo().unwrap();
        assert_eq!(editor.editor_content, vec!["a", "b", "mid"]);
        editor.redo().unwrap();
        assert_eq!(editor.editor_content, vec!["a", "b", "mid", "x", "y"]);
    }

    // ========== DeleteLines ==========
    #[test]
    fn undo_redo_delete_lines_range() {
        let mut editor = create_editor_with_editor_content(vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
            "e".to_string(),
        ]);
        // Remove b,c,d
        let removed = vec!["b".to_string(), "c".to_string(), "d".to_string()];
        let pos = CursorPosition { x: 0, y: 1 };
        editor
            .undo_redo_manager
            .record_undo(EditAction::DeleteLines {
                start: pos,
                deleted: removed.clone(),
            });
        editor.editor_content.drain(1..4);
        assert_eq!(editor.editor_content, vec!["a", "e"]);
        editor.undo().unwrap();
        assert_eq!(editor.editor_content, vec!["a", "b", "c", "d", "e"]);
        editor.redo().unwrap();
        assert_eq!(editor.editor_content, vec!["a", "e"]);
    }

    #[test]
    fn undo_redo_delete_all_lines() {
        let mut editor = create_editor_with_editor_content(vec![
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
        ]);
        let removed = editor.editor_content.clone();
        let pos = CursorPosition { x: 0, y: 0 };
        editor
            .undo_redo_manager
            .record_undo(EditAction::DeleteLines {
                start: pos,
                deleted: removed.clone(),
            });

        editor.editor_content.clear();
        assert_eq!(editor.editor_content, Vec::<String>::new());
        editor.undo().unwrap();
        assert_eq!(editor.editor_content, vec!["1", "2", "3"]);
        editor.redo().unwrap();
        assert_eq!(editor.editor_content, Vec::<String>::new());
    }

    // ========== Edge and Stack Cases ==========
    #[test]
    fn undo_redo_stack_behavior() {
        let mut editor = create_editor_with_editor_content(vec!["".to_string()]);
        // Undo & redo stack empty
        assert!(editor.undo().is_err());
        assert!(editor.redo().is_err());
        // Normal sequence
        editor.write_char('t');
        editor.undo().unwrap();
        assert!(editor.redo().is_ok());
        // After new action, redo stack cleared
        editor.write_char('z');
        assert!(editor.redo().is_err());
    }

    #[test]
    fn alternating_undo_redo_variety() {
        let mut editor = create_editor_with_editor_content(vec!["".to_string()]);
        editor.write_char('a');
        editor.write_char('b');
        editor.write_char('c');
        assert_eq!(editor.editor_content[0], "abc");
        editor.undo().unwrap();
        assert_eq!(editor.editor_content[0], "ab");
        editor.write_char('Z');
        assert_eq!(editor.editor_content[0], "abZ");
        assert!(editor.redo().is_err()); // Redo stack cleared
        editor.undo().unwrap();
        assert_eq!(editor.editor_content[0], "ab");
        editor.undo().unwrap();
        assert_eq!(editor.editor_content[0], "a");
        editor.redo().unwrap();
        assert_eq!(editor.editor_content[0], "ab");
    }
}
