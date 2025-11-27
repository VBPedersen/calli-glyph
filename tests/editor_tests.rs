#[cfg(test)]
mod editor_basic_tests {
    use calliglyph::core::cursor::CursorPosition;
    use calliglyph::core::editor::Editor;
    use calliglyph::input::input_action::{Direction, InputAction};
    /// Helper to create an editor with some starting text.
    fn create_editor_with_content(lines: Vec<&str>) -> Editor {
        let mut editor = Editor::new();
        editor.editor_content = lines.into_iter().map(String::from).collect();
        editor.editor_height = 10;
        editor
    }

    #[test]
    fn test_write_and_backspace_single_char() {
        let mut editor = create_editor_with_content(vec!["Hello"]);
        editor.cursor.x = 6;
        // Type a character
        editor
            .handle_input_action(InputAction::WriteChar('!'))
            .unwrap();
        assert_eq!(editor.editor_content[0], "Hello!");

        // Remove it with backspace
        editor.handle_input_action(InputAction::BACKSPACE).unwrap();
        assert_eq!(editor.editor_content[0], "Hello");
    }

    #[test]
    fn test_enter_splits_line_and_backspace_joins() {
        let mut editor = create_editor_with_content(vec!["Hello world", "goodbye world"]);
        editor.cursor.x = 6;
        editor.cursor.y = 0;

        // Press ENTER (split line)
        editor.handle_input_action(InputAction::ENTER).unwrap();
        assert_eq!(
            editor.editor_content.len(),
            3,
            "Line count should be 3 after split"
        );
        assert_eq!(
            editor.editor_content[0], "Hello ",
            "First line should contain the left half of the split"
        );
        assert_eq!(
            editor.editor_content[1], "world",
            "Second line should contain the right half of the split"
        );
        assert_eq!(
            editor.editor_content[2], "goodbye world",
            "Third line should contain the second start sentence"
        );

        // Join back
        editor.cursor.x = 0;
        editor.cursor.y = 1;

        editor.handle_input_action(InputAction::BACKSPACE).unwrap();
        assert_eq!(editor.editor_content.len(), 2);
        assert_eq!(editor.editor_content[0], "Hello world");
        assert_eq!(editor.editor_content[1], "goodbye world");
    }

    #[test]
    fn test_tab_inserts_spaces() {
        let mut editor = create_editor_with_content(vec![""]);
        editor.handle_input_action(InputAction::TAB).unwrap();
        assert!(editor.editor_content[0].starts_with("\t")); // assume 4-space tab
    }

    #[test]
    fn test_move_cursor_left_and_right() {
        let mut editor = create_editor_with_content(vec!["abcdef"]);
        editor.cursor.x = 6;

        editor
            .handle_input_action(InputAction::MoveCursor(Direction::Left))
            .unwrap();
        let left_x = editor.cursor.x;
        assert!(left_x < 6);

        editor
            .handle_input_action(InputAction::MoveCursor(Direction::Right))
            .unwrap();
        assert!(editor.cursor.x > left_x);
    }

    #[test]
    fn test_move_cursor_up_and_down() {
        let mut editor = create_editor_with_content(vec!["line1", "line2", "line3"]);
        editor.cursor.y = 1;

        editor
            .handle_input_action(InputAction::MoveCursor(Direction::Up))
            .unwrap();
        assert_eq!(editor.cursor.y, 0);

        editor
            .handle_input_action(InputAction::MoveCursor(Direction::Down))
            .unwrap();
        assert_eq!(editor.cursor.y, 1);
    }

    #[test]
    fn test_write_undo_and_redo() {
        let mut editor = create_editor_with_content(vec!["foo"]);
        editor.cursor.x = 3;
        // Type '!'
        editor
            .handle_input_action(InputAction::WriteChar('!'))
            .unwrap();
        assert_eq!(editor.editor_content[0], "foo!");

        // Undo it
        editor.handle_input_action(InputAction::UNDO).unwrap();
        assert_eq!(editor.editor_content[0], "foo");

        // Redo it
        editor.handle_input_action(InputAction::REDO).unwrap();
        assert_eq!(editor.editor_content[0], "foo!");
    }

    #[test]
    fn test_multiple_writes_and_undo_redo_chain() {
        let mut editor = create_editor_with_content(vec!["abc"]);
        editor.cursor.x = 3;
        editor.cursor.y = 0;
        for c in ['d', 'e', 'f'] {
            editor
                .handle_input_action(InputAction::WriteChar(c))
                .unwrap();
        }
        assert_eq!(editor.editor_content[0], "abcdef");

        // Undo all
        for _ in 0..3 {
            editor.handle_input_action(InputAction::UNDO).unwrap();
        }
        assert_eq!(editor.editor_content[0], "abc");

        // Redo all
        for _ in 0..3 {
            editor.handle_input_action(InputAction::REDO).unwrap();
        }
        assert_eq!(editor.editor_content[0], "abcdef");
    }

    #[test]
    fn test_multi_line_editing_with_undo() {
        let mut editor = create_editor_with_content(vec!["first", "second"]);
        editor.cursor.y = 0;
        editor
            .handle_input_action(InputAction::WriteChar('A'))
            .unwrap();

        editor.cursor.x = 0;
        editor.cursor.y = 1;
        editor
            .handle_input_action(InputAction::WriteChar('B'))
            .unwrap();

        assert_eq!(editor.editor_content[0], "Afirst");
        assert_eq!(editor.editor_content[1], "Bsecond");

        // Undo last change (B)
        editor.handle_input_action(InputAction::UNDO).unwrap();
        assert_eq!(editor.editor_content[1], "second");

        // Undo first change (A)
        editor.handle_input_action(InputAction::UNDO).unwrap();
        assert_eq!(editor.editor_content[0], "first");
    }

    #[test]
    fn test_text_selection_delete_range() {
        let mut editor = create_editor_with_content(vec!["delete this"]);
        editor.text_selection_start = Some(CursorPosition { x: 0, y: 0 });
        editor.text_selection_end = Some(CursorPosition { x: 11, y: 0 });

        editor.handle_input_action(InputAction::BACKSPACE).unwrap();
        assert_eq!(editor.editor_content[0], "");
    }

    #[test]
    fn test_scroll_offset_and_cursor_bounds() {
        let mut editor =
            create_editor_with_content(vec!["line1", "line2", "line3", "line4", "line5", "line6"]);
        editor.editor_height = 3;
        editor.scroll_offset = 2;
        editor.cursor.y = 5;

        editor
            .handle_input_action(InputAction::MoveCursor(Direction::Up))
            .unwrap();
        assert!(editor.scroll_offset <= 2);
    }

    // ========== Single Line Selection Tests ==========

    #[test]
    fn test_write_char_on_selection_single_line() {
        let mut editor = create_editor_with_content(vec!["Hello World"]);
        editor.text_selection_start = Some(CursorPosition { x: 6, y: 0 });
        editor.text_selection_end = Some(CursorPosition { x: 11, y: 0 });
        editor.cursor.x = 6;
        editor.cursor.y = 0;

        // Replace "World" with "!"
        editor
            .handle_input_action(InputAction::WriteChar('!'))
            .unwrap();
        assert_eq!(editor.editor_content[0], "Hello !");
        assert_eq!(editor.cursor.x, 7);
    }

    #[test]
    fn test_write_char_on_selection_single_line_undo_redo() {
        let mut editor = create_editor_with_content(vec!["Hello World"]);
        editor.text_selection_start = Some(CursorPosition { x: 6, y: 0 });
        editor.text_selection_end = Some(CursorPosition { x: 11, y: 0 });
        editor.cursor.x = 6;
        editor.cursor.y = 0;

        // Replace "World" with "!"
        editor
            .handle_input_action(InputAction::WriteChar('!'))
            .unwrap();
        assert_eq!(editor.editor_content[0], "Hello !");

        // Undo
        editor.handle_input_action(InputAction::UNDO).unwrap();
        assert_eq!(editor.editor_content[0], "Hello World");
        assert_eq!(editor.cursor.x, 11); //end of line after world

        // Redo
        editor.handle_input_action(InputAction::REDO).unwrap();
        assert_eq!(editor.editor_content[0], "Hello !");
        assert_eq!(editor.cursor.x, 7);
    }

    #[test]
    fn test_write_char_on_selection_at_start() {
        let mut editor = create_editor_with_content(vec!["Hello World"]);
        editor.text_selection_start = Some(CursorPosition { x: 0, y: 0 });
        editor.text_selection_end = Some(CursorPosition { x: 5, y: 0 });
        editor.cursor.x = 0;
        editor.cursor.y = 0;

        // Replace "Hello" with "X"
        editor
            .handle_input_action(InputAction::WriteChar('X'))
            .unwrap();
        assert_eq!(editor.editor_content[0], "X World");

        // Undo
        editor.handle_input_action(InputAction::UNDO).unwrap();
        assert_eq!(editor.editor_content[0], "Hello World");

        // Redo
        editor.handle_input_action(InputAction::REDO).unwrap();
        assert_eq!(editor.editor_content[0], "X World");
    }

    #[test]
    fn test_write_char_on_selection_at_end() {
        let mut editor = create_editor_with_content(vec!["Hello World"]);
        editor.text_selection_start = Some(CursorPosition { x: 6, y: 0 });
        editor.text_selection_end = Some(CursorPosition { x: 11, y: 0 });
        editor.cursor.x = 6;
        editor.cursor.y = 0;

        // Replace "World" with "Z"
        editor
            .handle_input_action(InputAction::WriteChar('Z'))
            .unwrap();
        assert_eq!(editor.editor_content[0], "Hello Z");

        // Undo
        editor.handle_input_action(InputAction::UNDO).unwrap();
        assert_eq!(editor.editor_content[0], "Hello World");

        // Redo
        editor.handle_input_action(InputAction::REDO).unwrap();
        assert_eq!(editor.editor_content[0], "Hello Z");
    }

    #[test]
    fn test_write_char_on_entire_line_selection() {
        let mut editor = create_editor_with_content(vec!["Delete Me"]);
        editor.text_selection_start = Some(CursorPosition { x: 0, y: 0 });
        editor.text_selection_end = Some(CursorPosition { x: 9, y: 0 });
        editor.cursor.x = 0;
        editor.cursor.y = 0;

        // Replace entire line
        editor
            .handle_input_action(InputAction::WriteChar('A'))
            .unwrap();
        assert_eq!(editor.editor_content[0], "A");

        // Undo
        editor.handle_input_action(InputAction::UNDO).unwrap();
        assert_eq!(editor.editor_content[0], "Delete Me");

        // Redo
        editor.handle_input_action(InputAction::REDO).unwrap();
        assert_eq!(editor.editor_content[0], "A");
    }

    // ========== Multi-Line Selection Tests ==========

    #[test]
    fn test_write_char_on_selection_multi_line() {
        let mut editor =
            create_editor_with_content(vec!["First Line", "Second Line", "Third Line"]);
        editor.text_selection_start = Some(CursorPosition { x: 6, y: 0 });
        editor.text_selection_end = Some(CursorPosition { x: 6, y: 1 });
        editor.cursor.x = 6;
        editor.cursor.y = 0;

        // Replace selection with "X"
        editor
            .handle_input_action(InputAction::WriteChar('X'))
            .unwrap();
        assert_eq!(editor.editor_content.len(), 2);
        assert_eq!(editor.editor_content[0], "First X Line");
        assert_eq!(editor.editor_content[1], "Third Line");
    }

    #[test]
    fn test_write_char_on_selection_multi_line_undo_redo() {
        let mut editor =
            create_editor_with_content(vec!["First Line", "Second Line", "Third Line"]);
        editor.text_selection_start = Some(CursorPosition { x: 6, y: 0 });
        editor.text_selection_end = Some(CursorPosition { x: 6, y: 1 });
        editor.cursor.x = 6;
        editor.cursor.y = 0;

        // Replace selection
        editor
            .handle_input_action(InputAction::WriteChar('X'))
            .unwrap();
        assert_eq!(editor.editor_content.len(), 2);
        assert_eq!(editor.editor_content[0], "First X Line");

        // Undo
        editor.handle_input_action(InputAction::UNDO).unwrap();
        assert_eq!(editor.editor_content.len(), 3);
        assert_eq!(editor.editor_content[0], "First Line");
        assert_eq!(editor.editor_content[1], "Second Line");
        assert_eq!(editor.editor_content[2], "Third Line");

        // Redo
        editor.handle_input_action(InputAction::REDO).unwrap();
        assert_eq!(editor.editor_content.len(), 2);
        assert_eq!(editor.editor_content[0], "First X Line");
        assert_eq!(editor.editor_content[1], "Third Line");
    }

    #[test]
    fn test_write_char_on_selection_spanning_three_lines() {
        let mut editor =
            create_editor_with_content(vec!["Line One", "Line Two", "Line Three", "Line Four"]);
        editor.text_selection_start = Some(CursorPosition { x: 5, y: 0 });
        editor.text_selection_end = Some(CursorPosition { x: 5, y: 2 });
        editor.cursor.x = 5;
        editor.cursor.y = 0;

        // Replace three-line selection
        editor
            .handle_input_action(InputAction::WriteChar('!'))
            .unwrap();
        assert_eq!(editor.editor_content.len(), 2);
        assert_eq!(editor.editor_content[0], "Line !Three");
        assert_eq!(editor.editor_content[1], "Line Four");

        // Undo
        editor.handle_input_action(InputAction::UNDO).unwrap();
        assert_eq!(editor.editor_content.len(), 4);
        assert_eq!(editor.editor_content[0], "Line One");
        assert_eq!(editor.editor_content[1], "Line Two");
        assert_eq!(editor.editor_content[2], "Line Three");
        assert_eq!(editor.editor_content[3], "Line Four");

        // Redo
        editor.handle_input_action(InputAction::REDO).unwrap();
        assert_eq!(editor.editor_content.len(), 2);
        assert_eq!(editor.editor_content[0], "Line !Three");
        assert_eq!(editor.editor_content[1], "Line Four");
    }

    #[test]
    fn test_write_char_on_selection_start_of_first_to_start_of_last() {
        let mut editor = create_editor_with_content(vec!["AAA", "BBB", "CCC"]);
        editor.text_selection_start = Some(CursorPosition { x: 0, y: 0 });
        editor.text_selection_end = Some(CursorPosition { x: 0, y: 2 });
        editor.cursor.x = 0;
        editor.cursor.y = 0;

        // Replace
        editor
            .handle_input_action(InputAction::WriteChar('Z'))
            .unwrap();
        assert_eq!(editor.editor_content.len(), 1);
        assert_eq!(editor.editor_content[0], "ZCCC");

        // Undo
        editor.handle_input_action(InputAction::UNDO).unwrap();
        assert_eq!(editor.editor_content.len(), 3);
        assert_eq!(editor.editor_content[0], "AAA");
        assert_eq!(editor.editor_content[1], "BBB");
        assert_eq!(editor.editor_content[2], "CCC");

        // Redo
        editor.handle_input_action(InputAction::REDO).unwrap();
        assert_eq!(editor.editor_content.len(), 1);
        assert_eq!(editor.editor_content[0], "ZCCC");
    }

    #[test]
    fn test_write_char_on_selection_middle_to_middle() {
        let mut editor = create_editor_with_content(vec!["abcdef", "ghijkl", "mnopqr"]);
        editor.text_selection_start = Some(CursorPosition { x: 3, y: 0 });
        editor.text_selection_end = Some(CursorPosition { x: 2, y: 2 });
        editor.cursor.x = 3;
        editor.cursor.y = 0;

        // Replace middle sections
        editor
            .handle_input_action(InputAction::WriteChar('*'))
            .unwrap();
        assert_eq!(editor.editor_content.len(), 1);
        assert_eq!(editor.editor_content[0], "abc*opqr");

        // Undo
        editor.handle_input_action(InputAction::UNDO).unwrap();
        assert_eq!(editor.editor_content.len(), 3);
        assert_eq!(editor.editor_content[0], "abcdef");
        assert_eq!(editor.editor_content[1], "ghijkl");
        assert_eq!(editor.editor_content[2], "mnopqr");

        // Redo
        editor.handle_input_action(InputAction::REDO).unwrap();
        assert_eq!(editor.editor_content.len(), 1);
        assert_eq!(editor.editor_content[0], "abc*opqr");
    }

    // ========== Backspace on Selection Tests ==========

    #[test]
    fn test_backspace_on_selection_single_line() {
        let mut editor = create_editor_with_content(vec!["Hello World"]);
        editor.text_selection_start = Some(CursorPosition { x: 6, y: 0 });
        editor.text_selection_end = Some(CursorPosition { x: 11, y: 0 });
        editor.cursor.x = 6;
        editor.cursor.y = 0;

        // Delete "World"
        editor.handle_input_action(InputAction::BACKSPACE).unwrap();
        assert_eq!(editor.editor_content[0], "Hello ");

        // Undo
        editor.handle_input_action(InputAction::UNDO).unwrap();
        assert_eq!(editor.editor_content[0], "Hello World");

        // Redo
        editor.handle_input_action(InputAction::REDO).unwrap();
        assert_eq!(editor.editor_content[0], "Hello ");
    }

    #[test]
    fn test_backspace_on_selection_multi_line() {
        let mut editor = create_editor_with_content(vec!["First", "Second", "Third"]);
        editor.text_selection_start = Some(CursorPosition { x: 2, y: 0 });
        editor.text_selection_end = Some(CursorPosition { x: 3, y: 1 });
        editor.cursor.x = 2;
        editor.cursor.y = 0;

        // Delete selection
        editor.handle_input_action(InputAction::BACKSPACE).unwrap();
        assert_eq!(editor.editor_content.len(), 2);
        assert_eq!(editor.editor_content[0], "Fiond");
        assert_eq!(editor.editor_content[1], "Third");

        // Undo
        editor.handle_input_action(InputAction::UNDO).unwrap();
        assert_eq!(editor.editor_content.len(), 3);
        assert_eq!(editor.editor_content[0], "First");
        assert_eq!(editor.editor_content[1], "Second");
        assert_eq!(editor.editor_content[2], "Third");

        // Redo
        editor.handle_input_action(InputAction::REDO).unwrap();
        assert_eq!(editor.editor_content.len(), 2);
        assert_eq!(editor.editor_content[0], "Fiond");
        assert_eq!(editor.editor_content[1], "Third");
    }

    // ========== Multiple Operations Tests ==========

    #[test]
    fn test_multiple_replace_operations_with_undo_redo() {
        let mut editor = create_editor_with_content(vec!["abc", "def", "ghi"]);

        // First operation: replace "bc" with "X"
        editor.text_selection_start = Some(CursorPosition { x: 1, y: 0 });
        editor.text_selection_end = Some(CursorPosition { x: 3, y: 0 });
        editor.cursor.x = 1;
        editor.cursor.y = 0;
        editor
            .handle_input_action(InputAction::WriteChar('X'))
            .unwrap();
        assert_eq!(editor.editor_content[0], "aX");

        // Second operation: replace "ef" with "Y"
        editor.text_selection_start = Some(CursorPosition { x: 1, y: 1 });
        editor.text_selection_end = Some(CursorPosition { x: 3, y: 1 });
        editor.cursor.x = 1;
        editor.cursor.y = 1;
        editor
            .handle_input_action(InputAction::WriteChar('Y'))
            .unwrap();
        assert_eq!(editor.editor_content[1], "dY");

        // Undo both
        editor.handle_input_action(InputAction::UNDO).unwrap();
        assert_eq!(editor.editor_content[1], "def");

        editor.handle_input_action(InputAction::UNDO).unwrap();
        assert_eq!(editor.editor_content[0], "abc");

        // Redo both
        editor.handle_input_action(InputAction::REDO).unwrap();
        assert_eq!(editor.editor_content[0], "aX");

        editor.handle_input_action(InputAction::REDO).unwrap();
        assert_eq!(editor.editor_content[1], "dY");
    }

    #[test]
    fn test_replace_then_new_action_clears_redo() {
        let mut editor = create_editor_with_content(vec!["Hello World"]);

        // Replace "World" with "X"
        editor.text_selection_start = Some(CursorPosition { x: 6, y: 0 });
        editor.text_selection_end = Some(CursorPosition { x: 11, y: 0 });
        editor.cursor.x = 6;
        editor.cursor.y = 0;
        editor
            .handle_input_action(InputAction::WriteChar('X'))
            .unwrap();

        // Undo
        editor.handle_input_action(InputAction::UNDO).unwrap();
        assert_eq!(editor.editor_content[0], "Hello World");

        // New action: type 'Y' at a different position
        editor.cursor.x = 0;
        editor
            .handle_input_action(InputAction::WriteChar('Y'))
            .unwrap();
        assert_eq!(editor.editor_content[0], "YHello World");

        // Redo should fail (redo stack cleared)
        let result = editor.handle_input_action(InputAction::REDO);
        // Should still have "YHello World", not go back to "Hello X"
        assert_eq!(editor.editor_content[0], "YHello World");
        assert!(result.is_err());
    }

    // ========== Edge Cases ==========

    #[test]
    fn test_replace_with_special_characters() {
        let mut editor = create_editor_with_content(vec!["café"]);
        editor.text_selection_start = Some(CursorPosition { x: 1, y: 0 });
        editor.text_selection_end = Some(CursorPosition { x: 3, y: 0 });
        editor.cursor.x = 1;
        editor.cursor.y = 0;

        // Replace "af" with "ø"
        editor
            .handle_input_action(InputAction::WriteChar('ø'))
            .unwrap();
        assert_eq!(editor.editor_content[0], "cøé");

        // Undo
        editor.handle_input_action(InputAction::UNDO).unwrap();
        assert_eq!(editor.editor_content[0], "café");

        // Redo
        editor.handle_input_action(InputAction::REDO).unwrap();
        assert_eq!(editor.editor_content[0], "cøé");
    }

    #[test]
    fn test_replace_empty_selection() {
        let mut editor = create_editor_with_content(vec!["test"]);
        editor.text_selection_start = Some(CursorPosition { x: 2, y: 0 });
        editor.text_selection_end = Some(CursorPosition { x: 2, y: 0 });
        editor.cursor.x = 2;
        editor.cursor.y = 0;

        // This should just insert, not replace
        editor
            .handle_input_action(InputAction::WriteChar('X'))
            .unwrap();
        assert_eq!(editor.editor_content[0], "teXst");

        // Undo
        editor.handle_input_action(InputAction::UNDO).unwrap();
        assert_eq!(editor.editor_content[0], "test");
    }
}

#[cfg(test)]
mod editor_paste_tests {
    use calliglyph::core::editor::Editor;
    use calliglyph::input::input_action::InputAction;

    fn editor_with(lines: Vec<&str>) -> Editor {
        let mut e = Editor::new();
        e.editor_content = lines.into_iter().map(|l| l.to_string()).collect();
        e
    }

    fn set_clipboard(editor: &mut Editor, contents: Vec<&str>) {
        editor.clipboard.copied_text = contents.into_iter().map(|s| s.to_string()).collect();
    }

    // --- SINGLE LINE PASTE ---

    #[test]
    fn test_paste_single_line_via_input_action() {
        let mut editor = editor_with(vec!["Hello world"]);
        editor.cursor.x = 6;
        editor.cursor.y = 0;

        set_clipboard(&mut editor, vec!["amazing "]);

        editor.handle_input_action(InputAction::PASTE).unwrap();

        assert_eq!(editor.editor_content[0], "Hello amazing world");
    }

    #[test]
    fn test_paste_single_line_into_empty_line() {
        let mut editor = editor_with(vec![""]);
        editor.cursor.x = 0;
        editor.cursor.y = 0;

        set_clipboard(&mut editor, vec!["Hello"]);

        editor.handle_input_action(InputAction::PASTE).unwrap();

        assert_eq!(editor.editor_content, vec!["Hello"]);
    }

    // --- MULTI-LINE PASTE ---

    #[test]
    fn test_paste_multi_line_middle_via_input_action() {
        let mut editor = editor_with(vec!["Hello world"]);
        editor.cursor.x = 6;
        editor.cursor.y = 0;

        set_clipboard(&mut editor, vec!["AAA", "BBB", "CCC"]);

        editor.handle_input_action(InputAction::PASTE).unwrap();

        assert_eq!(
            editor.editor_content,
            vec![
                "Hello AAA".to_string(),
                "BBB".to_string(),
                "CCCworld".to_string(),
            ]
        );
    }

    #[test]
    fn test_paste_multi_line_at_start_of_line_via_input_action() {
        let mut editor = editor_with(vec!["Hello"]);
        editor.cursor.x = 0;
        editor.cursor.y = 0;

        set_clipboard(&mut editor, vec!["X", "Y"]);

        editor.handle_input_action(InputAction::PASTE).unwrap();

        assert_eq!(editor.editor_content, vec!["X", "YHello",]);
    }

    #[test]
    fn test_paste_multi_line_at_end_of_line_via_input_action() {
        let mut editor = editor_with(vec!["Hello"]);
        editor.cursor.x = 5;
        editor.cursor.y = 0;

        set_clipboard(&mut editor, vec!["X", "Y"]);

        editor.handle_input_action(InputAction::PASTE).unwrap();

        assert_eq!(editor.editor_content, vec!["HelloX", "Y",]);
    }

    // --- ERROR CASE ---

    #[test]
    fn test_paste_with_empty_clipboard_reports_error() {
        let mut editor = editor_with(vec!["Hello"]);
        editor.cursor.x = 3;
        editor.cursor.y = 0;

        let result = editor.handle_input_action(InputAction::PASTE);

        assert!(result.is_err());
    }
}

#[cfg(test)]
mod editor_cut_tests {
    use calliglyph::core::cursor::CursorPosition;
    use calliglyph::core::editor::Editor;
    use calliglyph::input::input_action::InputAction;

    /// Helper to create an editor with some starting text.
    fn create_editor_with_content(lines: Vec<&str>) -> Editor {
        let mut editor = Editor::new();
        editor.editor_content = lines.into_iter().map(String::from).collect();
        editor.editor_height = 10;
        editor
    }
    #[test]
    fn test_multiline_cut_till_end() {
        let mut editor = create_editor_with_content(vec!["Hello World", "AAA", "BBB", "CCC"]);
        editor.text_selection_start = Some(CursorPosition { x: 0, y: 1 });
        editor.text_selection_end = Some(CursorPosition { x: 3, y: 3 });

        editor.handle_input_action(InputAction::CUT).unwrap();
        assert_eq!(editor.editor_content[0], "Hello World");
        assert_eq!(editor.editor_content.len(), 1);
    }

    #[test]
    fn test_multiline_cut_middle_of_editor() {
        let mut editor =
            create_editor_with_content(vec!["Hello World", "AAA", "BBB", "CCC", "End"]);
        editor.text_selection_start = Some(CursorPosition { x: 0, y: 1 });
        editor.text_selection_end = Some(CursorPosition { x: 3, y: 3 });

        editor.handle_input_action(InputAction::CUT).unwrap();
        assert_eq!(editor.editor_content[0], "Hello World");
        assert_eq!(editor.editor_content[1], "End");
        assert_eq!(editor.editor_content.len(), 2);
    }
}
