
#[cfg(test)]
mod app_tests {
    use crate::app::*; // Access app.rs logic

    //init functions
    fn create_app_with_editor_content(vec: Vec<String>) -> App {
        // Replace `YourEditorStruct` with your editor struct's name
        let mut app = App::new();
        app.editor_content = vec;
        app

    }

    //WRITING CHARS IN EDITOR

    #[test]
    fn test_write_char_in_editor() {
        let mut app = App::new();
        app.write_char_in_editor('a');
        assert_eq!(app.editor_content[0], "a");
        assert_eq!(app.cursor_x, 1);
    }

    #[test]
    fn test_write_char_in_editor_normal_characters() {
        let mut app = App::new();
        app.write_char_in_editor('a');
        app.write_char_in_editor('b');
        app.write_char_in_editor('c');
        app.write_char_in_editor('d');
        assert_eq!(app.editor_content[0], "abcd");
        assert_eq!(app.cursor_x, 4);
    }

    #[test]
    fn test_write_char_in_editor_special_characters() {
        let mut app = App::new();
        app.write_char_in_editor('ᚠ');
        app.write_char_in_editor('Ω');
        app.write_char_in_editor('₿');
        app.write_char_in_editor('😎');
        assert_eq!(app.editor_content[0], "ᚠΩ₿😎");
        assert_eq!(app.cursor_x, 4);
    }

    #[test]
    fn test_write_char_in_editor_at_line_10() {
        let mut app = App::new();
        app.cursor_y = 10;
        app.write_char_in_editor('a');
        assert_eq!(app.editor_content[10], "a");
        assert_eq!(app.cursor_x, 1);
    }

    #[test]
    fn test_write_char_in_editor_at_100_x() {
        let mut app = App::new();
        app.cursor_x = 100;
        app.write_char_in_editor('a');
        assert_eq!(app.editor_content[0], "a");
        assert_eq!(app.cursor_x, 1);
    }



    //BACKSPACE IN EDITOR
    #[test]
    fn test_backspace_in_editor() {
        let mut app = create_app_with_editor_content(vec!['a'.to_string()]);
        app.cursor_x = 1;
        app.backspace_in_editor();
        assert_eq!(app.editor_content[0], "");
        assert_eq!(app.cursor_x, 0);
    }

    #[test]
    fn test_backspace_in_editor_special_characters() {
        let mut app = create_app_with_editor_content(vec!["ᚠΩ₿😎".to_string()]);
        app.cursor_x = 4;
        app.backspace_in_editor();
        assert_eq!(app.editor_content[0], "ᚠΩ₿");
        assert_eq!(app.cursor_x, 3);
    }

    #[test]
    fn test_backspace_in_editor_should_go_to_previous_line() {
        let mut app = create_app_with_editor_content(vec!["a".to_string(), "b".to_string()]);
        app.cursor_y = 1;
        app.cursor_x = 0;
        app.backspace_in_editor();
        assert_eq!(app.editor_content[0], "ab");
        assert_eq!(app.editor_content.len(), 1);
        assert_eq!(app.cursor_x, 1);
        assert_eq!(app.cursor_y, 0);

    }


    //DELETE IN EDITOR
    #[test]
    fn test_delete_in_editor() {
        let mut app = create_app_with_editor_content(vec!["ab".to_string()]);
        app.cursor_x = 0;
        app.delete_in_editor();
        assert_eq!(app.editor_content[0], "a");
        assert_eq!(app.cursor_x, 0);
    }

    #[test]
    fn test_delete_in_editor_special_characters() {
        let mut app = create_app_with_editor_content(vec!["ᚠΩ₿😎".to_string(),]);
        app.cursor_x = 2;
        app.delete_in_editor();
        assert_eq!(app.editor_content[0], "ᚠΩ₿");
        assert_eq!(app.cursor_x, 2);
    }

    #[test]
    fn test_delete_in_editor_should_go_to_previous_line() {
        let mut app = create_app_with_editor_content(vec!["a".to_string(), "b".to_string()]);
        app.cursor_x = 1;
        app.delete_in_editor();
        assert_eq!(app.editor_content[0], "ab");
        assert_eq!(app.editor_content.len(), 1);
        assert_eq!(app.cursor_x, 1);
    }

    //ENTER in editor

    #[test]
    fn test_enter_in_editor_at_end_of_line() {
        let mut app = create_app_with_editor_content(vec!["Hello World".to_string()]);
        app.cursor_x = app.editor_content[0].len() as i16; // Set cursor to end of line
        app.enter_in_editor();

        assert_eq!(app.cursor_y, 1); // Cursor should move to the next line
        assert_eq!(app.editor_content.len(), 2); // New line added
        assert_eq!(app.editor_content[1], ""); // New line should be empty
    }

    #[test]
    fn test_enter_in_editor_mid_line() {
        let mut app = create_app_with_editor_content(vec!["Hello World".to_string()]);
        app.cursor_x = 5; // Split the line at index 5
        app.enter_in_editor();

        assert_eq!(app.cursor_y, 1); // Cursor should move to next line
        assert_eq!(app.cursor_x, 0); // Cursor resets to start of new line
        assert_eq!(app.editor_content[0], "Hello"); // Line before cursor is kept intact
        assert_eq!(app.editor_content[1], " World"); // Line after cursor is moved to new line
    }



}