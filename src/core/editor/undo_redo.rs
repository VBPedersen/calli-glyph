use super::editor::EditAction;
use crate::errors::editor_errors::{RedoError, UndoError};
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct UndoRedoManager {
    pub undo_stack: VecDeque<EditAction>,
    pub redo_stack: VecDeque<EditAction>,
    max_history: usize,
    last_saved_index: usize,
}

impl UndoRedoManager {
    pub fn new(max_history: usize) -> UndoRedoManager {
        Self {
            undo_stack: VecDeque::with_capacity(max_history),
            redo_stack: VecDeque::new(),
            max_history,
            last_saved_index: 0,
        }
    }
    
    // saved index management (changes)
    
    /// Checks if change in undo tree (signifies changes in file)
    pub fn is_dirty(&self) -> bool {
        self.undo_stack.len() != self.last_saved_index
    }
    
    /// marks new saved index to current length of stack
    pub fn mark_saved(&mut self) {
        self.last_saved_index = self.undo_stack.len();
    }
    
    
    /// Records and action done to the undo stack, and clears redo stack.
    pub fn record_undo(&mut self, action: EditAction) {
        self.undo_stack.push_back(action);

        // Limit history size
        while self.undo_stack.len() > self.max_history {
            self.undo_stack.pop_front();
        }

        self.redo_stack.clear();
    }

    /// Function to change max limit of undo history
    pub fn update_limit(&mut self, new_limit: usize) {
        self.max_history = new_limit;

        // Trim if necessary
        while self.undo_stack.len() > new_limit {
            self.undo_stack.pop_front();
        }
    }

    // UNDO AND REDO FUNCTIONALITY
    /// undo's last action of user
    pub fn undo(&mut self) -> Result<EditAction, UndoError> {
        if let Some(last_action) = self.undo_stack.pop_back() {
            let action_reversed = self.reverse_action(&last_action);
            self.redo_stack.push_back(last_action);
            Ok(action_reversed)
        } else {
            Err(UndoError::NoActionToUndo)
        }
    }

    /// redo's last action of user
    pub fn redo(&mut self) -> Result<EditAction, RedoError> {
        if let Some(last_action) = self.redo_stack.pop_back() {
            self.undo_stack.push_back(last_action.clone());
            Ok(last_action)
        } else {
            Err(RedoError::NoActionToRedo)
        }
    }

    /// returns the reverse action of the given action
    fn reverse_action(&mut self, action: &EditAction) -> EditAction {
        match action {
            EditAction::Insert { pos, c } => EditAction::Delete {
                pos: *pos,
                deleted_char: *c,
            },
            EditAction::Delete { pos, deleted_char } => EditAction::Insert {
                pos: *pos,
                c: *deleted_char,
            },
            EditAction::Replace {
                start,
                end,
                old,
                new,
            } => EditAction::Replace {
                start: *start,
                end: *end,
                old: new.clone(),
                new: old.clone(),
            },
            EditAction::ReplaceRange {
                start,
                end,
                old,
                new,
            } => EditAction::ReplaceRange {
                start: *start,
                end: *end,
                old: new.clone(),
                new: old.clone(),
            },
            EditAction::InsertLines { start, lines } => EditAction::DeleteLines {
                start: *start,
                deleted: lines.clone(),
            },
            EditAction::DeleteLines { start, deleted } => EditAction::InsertLines {
                start: *start,
                lines: deleted.clone(),
            },
            EditAction::DeleteRange {
                start,
                end,
                deleted,
            } => EditAction::InsertRange {
                start: *start,
                end: *end,
                lines: deleted.clone(),
            },
            EditAction::InsertRange { start, end, lines } => EditAction::DeleteRange {
                start: *start,
                end: *end,
                deleted: lines.clone(),
            },
            EditAction::SplitLine { pos, left, right } => EditAction::JoinLine {
                pos: *pos,
                merged: format!("{}{}", left, right),
            },
            EditAction::JoinLine { pos, merged } => EditAction::SplitLine {
                pos: *pos,
                left: merged[..pos.x].to_string(),
                right: merged[pos.x..].to_string(),
            },
        }
    }
}
