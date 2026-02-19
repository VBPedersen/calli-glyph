#[cfg(test)]
mod integration_app_tests {
    use calliglyph::app_config::AppLaunchConfig;
    use calliglyph::config::Config;
    use calliglyph::core::app::*;
    use calliglyph::core::command_line::command_binds::command_binds::*;
    use calliglyph::input::actions::InputAction;
    use calliglyph::ui::popups::popup::PopupResult;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::NamedTempFile;

    //init functions
    fn create_app() -> App {
        let app = App::new(Config::default(), AppLaunchConfig::default());
        app
    }

    fn test_save_path(filename: &str) -> PathBuf {
        PathBuf::from(format!("test_saves/{}", filename))
    }

    #[test]
    fn test_no_pending_states_does_nothing() {
        let mut app = create_app();
        app.handle_confirmation_popup_response();
        assert!(app.pending_states.is_empty());
    }

    #[test]
    fn test_save_confirmation_saves_file_and_removes_state() {
        let mut app = create_app();
        let save_path = test_save_path("file1.txt");
        app.editor.editor_content = vec![String::from("test")];

        app.pending_states
            .push_back(PendingState::Saving(save_path.clone()));
        app.popup_result = PopupResult::Bool(true);

        app.handle_confirmation_popup_response();

        assert!(Path::new(&save_path).exists());
        assert!(app.pending_states.is_empty());
        assert_eq!(app.popup_result, PopupResult::None);
        assert!(app.popup.is_none());

        // Cleanup test file
        fs::remove_file(&save_path).ok();
    }

    #[test]
    fn test_save_rejection_closes_popup_but_does_not_save() {
        let mut app = create_app();
        let save_path = test_save_path("file2.txt");
        app.editor.editor_content = vec![String::from("test")];

        app.pending_states
            .push_back(PendingState::Saving(save_path.clone()));
        app.popup_result = PopupResult::Bool(false);

        app.handle_confirmation_popup_response();

        assert!(!Path::new(&save_path).exists());
        assert_eq!(app.popup_result, PopupResult::None);
        assert!(app.popup.is_none());
    }

    #[test]
    fn test_quit_state_calls_quit() {
        let mut app = create_app();
        app.pending_states.push_back(PendingState::QuittingAbsolute);

        app.handle_confirmation_popup_response();
        assert!(app.pending_states.is_empty()); // Ensuring quit state was processed
    }

    #[test]
    fn test_save_then_quit_calls_save_then_quit() {
        let mut app = create_app();
        let save_path = test_save_path("file3.txt");
        app.editor.editor_content = vec![String::from("test")];
        app.pending_states
            .push_back(PendingState::Saving(save_path.clone()));
        app.pending_states.push_back(PendingState::QuittingAbsolute);
        app.popup_result = PopupResult::Bool(true);

        app.handle_confirmation_popup_response();
        app.handle_confirmation_popup_response();

        assert!(app.pending_states.is_empty());
        assert!(Path::new(&save_path).exists());

        // Cleanup test file
        fs::remove_file(&save_path).ok();
    }

    fn create_app_with_editor_content(vec: Vec<String>) -> App {
        let mut app = App::new(Config::default(), AppLaunchConfig::default());
        app.editor.editor_content = vec;
        app
    }
    #[test]
    fn test_save_with_specified_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mut app = create_app_with_editor_content(vec!["Test content".to_string()]);
        app.file_path = None;
        app.active_area = ActiveArea::CommandLine;
        app.command_line.input = ":".to_owned()
            + COMMAND_SAVE_DONT_EXIT
            + " "
            + temp_file.path().to_str().unwrap()
            + " --force";
        app.process_input_action(InputAction::ENTER);

        let saved_content = fs::read_to_string(file_path).unwrap();
        assert_eq!(saved_content, "Test content");
    }

    #[test]
    fn test_save_with_existing_file_path() {
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_path_buf();

        let mut app = create_app_with_editor_content(vec!["New content".to_string()]);
        app.file_path = Some(file_path.clone());
        app.active_area = ActiveArea::CommandLine;
        app.command_line.input = ":".to_owned() + COMMAND_SAVE_DONT_EXIT + " --force";
        app.process_input_action(InputAction::ENTER);

        let saved_content = fs::read_to_string(file_path).unwrap();
        assert_eq!(saved_content, "New content");
    }

    #[test]
    fn test_save_with_no_file_path_defaults_to_untitled() {
        let mut app = create_app_with_editor_content(vec!["Default content".to_string()]);

        app.active_area = ActiveArea::CommandLine;
        app.command_line.input = ":".to_owned() + COMMAND_SAVE_DONT_EXIT + " --force";
        app.process_input_action(InputAction::ENTER);

        let saved_content = fs::read_to_string("untitled").unwrap();
        assert_eq!(saved_content, "Default content");

        fs::remove_file("untitled").unwrap(); // Clean up after test
    }

    #[test]
    fn test_does_not_save_if_no_changes() {
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_path_buf();
        fs::write(&file_path, "Unchanged content").unwrap();
        let mut app = create_app_with_editor_content(vec!["Unchanged content".to_string()]);
        app.file_path = Some(file_path.clone());
        app.active_area = ActiveArea::CommandLine;
        app.command_line.input = ":".to_owned()
            + COMMAND_SAVE_DONT_EXIT
            + " "
            + temp_file.path().to_str().unwrap()
            + " --force";
        app.process_input_action(InputAction::ENTER);

        let saved_content = fs::read_to_string(file_path).unwrap();
        assert_eq!(saved_content, "Unchanged content"); // No overwrite happened
    }

    #[test]
    fn test_save_creates_new_file_if_missing() {
        let temp_file_path = "new_test_file.txt".to_string();
        let mut app = create_app_with_editor_content(vec!["Hello World!".to_string()]);
        app.file_path = None;
        app.active_area = ActiveArea::CommandLine;
        app.command_line.input =
            ":".to_owned() + COMMAND_SAVE_DONT_EXIT + " " + temp_file_path.as_str() + " --force";
        app.process_input_action(InputAction::ENTER);

        let saved_content = fs::read_to_string(&temp_file_path).unwrap();
        assert_eq!(saved_content, "Hello World!");

        fs::remove_file(temp_file_path).unwrap(); // Clean up
    }
}
