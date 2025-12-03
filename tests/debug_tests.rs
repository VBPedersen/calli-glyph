#[cfg(test)]
mod debug_logger_tests {
    use calliglyph::core::debug::{DebugLogger, LogEntry, LogLevel};
    use std::time::Instant;

    #[test]
    fn test_logger_push_entry() {
        let mut logger = DebugLogger::new(10);
        logger.push(LogEntry {
            timestamp: Instant::now(),
            level: LogLevel::Info,
            message: "Test message".to_string(),
            context: None,
        });

        assert_eq!(logger.entries().len(), 1);
        assert_eq!(logger.entries()[0].message, "Test message");
    }

    #[test]
    fn test_logger_respects_max_entries() {
        let mut logger = DebugLogger::new(3);

        for i in 0..5 {
            logger.push(LogEntry {
                timestamp: Instant::now(),
                level: LogLevel::Info,
                message: format!("Message {}", i),
                context: None,
            });
        }

        assert_eq!(logger.entries().len(), 3);
        // Should have kept the last 3 messages (2, 3, 4)
        assert_eq!(logger.entries()[0].message, "Message 2");
        assert_eq!(logger.entries()[2].message, "Message 4");
    }

    #[test]
    fn test_logger_clear() {
        let mut logger = DebugLogger::new(10);
        logger.push(LogEntry {
            timestamp: Instant::now(),
            level: LogLevel::Info,
            message: "Test".to_string(),
            context: None,
        });

        assert_eq!(logger.entries().len(), 1);
        logger.clear();
        assert_eq!(logger.entries().len(), 0);
    }

    #[test]
    fn test_logger_filter_by_level() {
        let mut logger = DebugLogger::new(10);

        logger.push(LogEntry {
            timestamp: Instant::now(),
            level: LogLevel::Error,
            message: "Error msg".to_string(),
            context: None,
        });
        logger.push(LogEntry {
            timestamp: Instant::now(),
            level: LogLevel::Info,
            message: "Info msg".to_string(),
            context: None,
        });
        logger.push(LogEntry {
            timestamp: Instant::now(),
            level: LogLevel::Warn,
            message: "Warn msg".to_string(),
            context: None,
        });

        let errors = logger.filter_by_level(LogLevel::Error);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Error msg");

        let warns_and_above = logger.filter_by_level(LogLevel::Warn);
        assert_eq!(warns_and_above.len(), 2); // Error and Warn
    }

    #[test]
    fn test_logger_count_by_level() {
        let mut logger = DebugLogger::new(10);

        for _ in 0..3 {
            logger.push(LogEntry {
                timestamp: Instant::now(),
                level: LogLevel::Error,
                message: "Error".to_string(),
                context: None,
            });
        }
        for _ in 0..2 {
            logger.push(LogEntry {
                timestamp: Instant::now(),
                level: LogLevel::Warn,
                message: "Warn".to_string(),
                context: None,
            });
        }

        assert_eq!(logger.count_by_level(LogLevel::Error), 3);
        assert_eq!(logger.count_by_level(LogLevel::Warn), 2);
        assert_eq!(logger.count_by_level(LogLevel::Info), 0);
    }

    #[test]
    fn test_logger_empty_initially() {
        let logger = DebugLogger::new(10);
        assert_eq!(logger.entries().len(), 0);
    }
}

#[cfg(test)]
mod debug_metrics_tests {
    use calliglyph::core::debug::PerformanceMetrics;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_metrics_initialization() {
        let metrics = PerformanceMetrics::new();
        assert_eq!(metrics.event_count, 0);
        assert_eq!(metrics.render_count, 0);
        assert_eq!(metrics.frame_times.len(), 0);
    }

    #[test]
    fn test_metrics_tick_increments_render_count() {
        let mut metrics = PerformanceMetrics::new();
        metrics.tick();
        assert_eq!(metrics.render_count, 1);
        metrics.tick();
        assert_eq!(metrics.render_count, 2);
    }

    #[test]
    fn test_metrics_tick_records_frame_time() {
        let mut metrics = PerformanceMetrics::new();
        thread::sleep(Duration::from_millis(10));
        metrics.tick();

        assert_eq!(metrics.frame_times.len(), 1);
        assert!(metrics.frame_times[0] >= Duration::from_millis(10));
    }

    #[test]
    fn test_metrics_respects_max_frame_times() {
        let mut metrics = PerformanceMetrics::new();

        // Add 121 frames (max is 120)
        for _ in 0..121 {
            metrics.tick();
        }

        assert_eq!(metrics.frame_times.len(), 120);
    }

    #[test]
    fn test_metrics_avg_frame_time() {
        let mut metrics = PerformanceMetrics::new();

        // Manually add frame times
        metrics.frame_times.push_back(Duration::from_millis(10));
        metrics.frame_times.push_back(Duration::from_millis(20));
        metrics.frame_times.push_back(Duration::from_millis(30));

        let avg = metrics.avg_frame_time();
        assert_eq!(avg, Duration::from_millis(20));
    }

    #[test]
    fn test_metrics_avg_frame_time_empty() {
        let metrics = PerformanceMetrics::new();
        assert_eq!(metrics.avg_frame_time(), Duration::from_secs(0));
    }

    #[test]
    fn test_metrics_record_event() {
        let mut metrics = PerformanceMetrics::new();
        metrics.record_event();
        assert_eq!(metrics.event_count, 1);
        metrics.record_event();
        assert_eq!(metrics.event_count, 2);
    }

    #[test]
    fn test_metrics_min_max_frame_time() {
        let mut metrics = PerformanceMetrics::new();
        metrics.frame_times.push_back(Duration::from_millis(10));
        metrics.frame_times.push_back(Duration::from_millis(30));
        metrics.frame_times.push_back(Duration::from_millis(20));

        assert_eq!(metrics.min_frame_time(), Duration::from_millis(10));
        assert_eq!(metrics.max_frame_time(), Duration::from_millis(30));
    }

    #[test]
    fn test_metrics_reset() {
        let mut metrics = PerformanceMetrics::new();
        metrics.tick();
        metrics.record_event();
        metrics.record_event();

        assert_eq!(metrics.render_count, 1);
        assert_eq!(metrics.event_count, 2);
        assert_eq!(metrics.frame_times.len(), 1);

        metrics.reset();

        assert_eq!(metrics.render_count, 0);
        assert_eq!(metrics.event_count, 0);
        assert_eq!(metrics.frame_times.len(), 0);
    }
}

#[cfg(test)]
mod debug_snapshot_tests {
    use calliglyph::core::app::ActiveArea;
    use calliglyph::core::cursor::Cursor;
    use calliglyph::core::debug::{CaptureMode, DebugState, SnapshotTrigger};
    use std::path::PathBuf;

    fn create_test_debug_state() -> DebugState {
        DebugState::new()
    }

    #[test]
    fn test_snapshot_history_initialization() {
        let debug_state = create_test_debug_state();
        assert_eq!(debug_state.snapshots.len(), 0);
        assert!(debug_state.snapshots.is_empty());
    }

    #[test]
    fn test_capture_manual_snapshot() {
        let mut debug_state = create_test_debug_state();
        debug_state.enabled = true;

        debug_state.capture_manual_snapshot(
            ActiveArea::Editor,
            Cursor { x: 0, y: 0 },
            None,
            vec!["test".to_string()],
            0,
            vec![],
            vec![],
            vec![],
            None,
        );

        assert_eq!(debug_state.snapshots.len(), 1);
        let snapshot = debug_state.snapshots.latest().unwrap();
        assert!(matches!(snapshot.trigger, SnapshotTrigger::Manual));
    }

    #[test]
    fn test_snapshot_not_captured_when_disabled() {
        let mut debug_state = create_test_debug_state();
        debug_state.enabled = false; // Disabled

        debug_state.update_and_maybe_snapshot(
            ActiveArea::Editor,
            Some(SnapshotTrigger::Error("test".to_string())),
            Cursor { x: 0, y: 0 },
            None,
            vec![],
            0,
            vec![],
            vec![],
            vec![],
            None,
        );

        assert_eq!(debug_state.snapshots.len(), 0);
    }

    #[test]
    fn test_capture_mode_none_prevents_snapshots() {
        let mut debug_state = create_test_debug_state();
        debug_state.enabled = true;
        debug_state.capture_mode = CaptureMode::None;

        debug_state.update_and_maybe_snapshot(
            ActiveArea::Editor,
            Some(SnapshotTrigger::Error("test".to_string())),
            Cursor { x: 0, y: 0 },
            None,
            vec![],
            0,
            vec![],
            vec![],
            vec![],
            None,
        );

        assert_eq!(debug_state.snapshots.len(), 0);
    }

    #[test]
    fn test_capture_mode_on_event_captures_with_trigger() {
        let mut debug_state = create_test_debug_state();
        debug_state.enabled = true;
        debug_state.capture_mode = CaptureMode::OnEvent;

        debug_state.update_and_maybe_snapshot(
            ActiveArea::Editor,
            Some(SnapshotTrigger::Command("test".to_string())),
            Cursor { x: 0, y: 0 },
            None,
            vec![],
            0,
            vec![],
            vec![],
            vec![],
            None,
        );

        assert_eq!(debug_state.snapshots.len(), 1);
    }

    #[test]
    fn test_capture_mode_on_event_ignores_without_trigger() {
        let mut debug_state = create_test_debug_state();
        debug_state.enabled = true;
        debug_state.capture_mode = CaptureMode::OnEvent;

        debug_state.update_and_maybe_snapshot(
            ActiveArea::Editor,
            None, // No trigger
            Cursor { x: 0, y: 0 },
            None,
            vec![],
            0,
            vec![],
            vec![],
            vec![],
            None,
        );

        assert_eq!(debug_state.snapshots.len(), 0);
    }

    #[test]
    fn test_capture_mode_manual_only_captures_manual_trigger() {
        let mut debug_state = create_test_debug_state();
        debug_state.enabled = true;
        debug_state.capture_mode = CaptureMode::Manual;

        // Try error trigger - should not capture
        debug_state.update_and_maybe_snapshot(
            ActiveArea::Editor,
            Some(SnapshotTrigger::Error("test".to_string())),
            Cursor { x: 0, y: 0 },
            None,
            vec![],
            0,
            vec![],
            vec![],
            vec![],
            None,
        );
        assert_eq!(debug_state.snapshots.len(), 0);

        // Try manual trigger - should capture
        debug_state.update_and_maybe_snapshot(
            ActiveArea::Editor,
            Some(SnapshotTrigger::Manual),
            Cursor { x: 0, y: 0 },
            None,
            vec![],
            0,
            vec![],
            vec![],
            vec![],
            None,
        );
        assert_eq!(debug_state.snapshots.len(), 1);
    }

    #[test]
    fn test_capture_mode_every_frame_always_captures() {
        let mut debug_state = create_test_debug_state();
        debug_state.enabled = true;
        debug_state.capture_mode = CaptureMode::EveryFrame;

        // Without trigger
        debug_state.update_and_maybe_snapshot(
            ActiveArea::Editor,
            None,
            Cursor { x: 0, y: 0 },
            None,
            vec![],
            0,
            vec![],
            vec![],
            vec![],
            None,
        );
        assert_eq!(debug_state.snapshots.len(), 1);

        // With trigger
        debug_state.update_and_maybe_snapshot(
            ActiveArea::Editor,
            Some(SnapshotTrigger::Error("test".to_string())),
            Cursor { x: 0, y: 0 },
            None,
            vec![],
            0,
            vec![],
            vec![],
            vec![],
            None,
        );
        assert_eq!(debug_state.snapshots.len(), 2);
    }

    #[test]
    fn test_snapshot_history_respects_max() {
        let mut debug_state = create_test_debug_state();
        debug_state.enabled = true;

        // Capture 51 snapshots (max is 50)
        for i in 0..51 {
            debug_state.capture_manual_snapshot(
                ActiveArea::Editor,
                Cursor { x: i, y: i },
                None,
                vec![format!("line {}", i)],
                0,
                vec![],
                vec![],
                vec![],
                None,
            );
        }

        assert_eq!(debug_state.snapshots.len(), 50);
        // Should have dropped the first one
        let first = debug_state.snapshots.get(0).unwrap();
        let pos = (first.cursor_pos.x, first.cursor_pos.y);
        assert_eq!(pos, (1, 1)); // Started from 1, not 0
    }

    #[test]
    fn test_snapshot_clear() {
        let mut debug_state = create_test_debug_state();
        debug_state.enabled = true;

        debug_state.capture_manual_snapshot(
            ActiveArea::Editor,
            Cursor { x: 0, y: 0 },
            None,
            vec![],
            0,
            vec![],
            vec![],
            vec![],
            None,
        );
        assert_eq!(debug_state.snapshots.len(), 1);

        debug_state.clear_snapshots();
        assert_eq!(debug_state.snapshots.len(), 0);
    }

    #[test]
    fn test_snapshot_captures_state_correctly() {
        let mut debug_state = create_test_debug_state();
        debug_state.enabled = true;

        debug_state.capture_manual_snapshot(
            ActiveArea::CommandLine,
            Cursor { x: 5, y: 10 },
            None,
            vec!["line1".to_string(), "line2".to_string()],
            2,
            vec!["clip1".to_string()],
            vec![],
            vec![],
            Some(PathBuf::from("/path/to/file")),
        );

        let snapshot = debug_state.snapshots.latest().unwrap();
        let pos = (snapshot.cursor_pos.x, snapshot.cursor_pos.y);
        assert_eq!(pos, (5, 10));
        assert_eq!(snapshot.buffer_lines, 2);
        assert_eq!(snapshot.buffer_content[0], "line1");
        assert_eq!(snapshot.clipboard_size, 1);
        assert_eq!(snapshot.file_path, Some(PathBuf::from("/path/to/file")));
    }
}

#[cfg(test)]
mod debug_state_tests {
    use calliglyph::core::debug::{CaptureMode, DebugState, LogLevel};

    #[test]
    fn test_debug_state_initialization() {
        let debug_state = DebugState::new();
        assert!(!debug_state.enabled);
        assert_eq!(debug_state.capture_mode, CaptureMode::OnEvent);
        assert_eq!(debug_state.logger.entries().len(), 0);
        assert_eq!(debug_state.snapshots.len(), 0);
    }

    #[test]
    fn test_debug_state_log_when_enabled() {
        let mut debug_state = DebugState::new();
        debug_state.enabled = true;

        debug_state.log(LogLevel::Info, "Test message");
        assert_eq!(debug_state.logger.entries().len(), 1);
    }

    #[test]
    fn test_debug_state_log_when_disabled() {
        let mut debug_state = DebugState::new();
        debug_state.enabled = false;

        debug_state.log(LogLevel::Info, "Test message");
        assert_eq!(debug_state.logger.entries().len(), 0);
    }

    #[test]
    fn test_debug_state_tick_frame_when_enabled() {
        let mut debug_state = DebugState::new();
        debug_state.enabled = true;

        debug_state.tick_frame();
        assert_eq!(debug_state.metrics.render_count, 1);
    }

    #[test]
    fn test_debug_state_tick_frame_when_disabled() {
        let mut debug_state = DebugState::new();
        debug_state.enabled = false;

        debug_state.tick_frame();
        assert_eq!(debug_state.metrics.render_count, 0);
    }

    #[test]
    fn test_debug_state_set_capture_mode() {
        let mut debug_state = DebugState::new();
        debug_state.enabled = true;

        debug_state.set_capture_mode(CaptureMode::EveryFrame);
        assert_eq!(debug_state.capture_mode, CaptureMode::EveryFrame);
        assert_eq!(debug_state.logger.entries().len(), 1); // Should log the change
    }

    #[test]
    fn test_debug_state_clear_logs() {
        let mut debug_state = DebugState::new();
        debug_state.enabled = true;

        debug_state.log(LogLevel::Info, "Test 1");
        debug_state.log(LogLevel::Info, "Test 2");
        assert_eq!(debug_state.logger.entries().len(), 2);

        debug_state.clear_logs();
        assert_eq!(debug_state.logger.entries().len(), 0);
    }
}

#[cfg(test)]
mod debug_view_tests {
    use calliglyph::ui::debug::{DebugTab, DebugView};

    // TODO
    // #[test]
    // fn test_debug_view_initialization() {
    //     let view = DebugView::new();
    //     assert_eq!(view.active_tab, DebugTab::Overview);
    //     assert_eq!(view.scroll_offset, 0);
    //     assert_eq!(view.selected_snapshot, None);
    //     assert!(!view.viewing_snapshot);
    // }

    #[test]
    fn test_debug_view_next_tab() {
        let mut view = DebugView::new();
        assert_eq!(view.active_tab, DebugTab::Overview);

        view.next_tab();
        assert_eq!(view.active_tab, DebugTab::Logs);

        view.next_tab();
        assert_eq!(view.active_tab, DebugTab::Clipboard);
    }

    #[test]
    fn test_debug_view_prev_tab() {
        let mut view = DebugView::new();
        view.active_tab = DebugTab::Clipboard;

        view.prev_tab();
        assert_eq!(view.active_tab, DebugTab::Logs);

        view.prev_tab();
        assert_eq!(view.active_tab, DebugTab::Overview);
    }

    #[test]
    fn test_debug_view_tab_cycling() {
        let mut view = DebugView::new();
        assert_eq!(view.active_tab, DebugTab::Overview);

        // Cycle through all tabs
        view.next_tab(); // Logs
        view.next_tab(); // Clipboard
        view.next_tab(); // History
        view.next_tab(); // Snapshots
        view.next_tab(); // Performance
        view.next_tab(); // Back to Overview
        assert_eq!(view.active_tab, DebugTab::Overview);
    }

    #[test]
    fn test_debug_view_scroll_up() {
        let mut view = DebugView::new();
        view.scroll_offset = 5;

        view.scroll_up();
        assert_eq!(view.scroll_offset, 4);
    }

    #[test]
    fn test_debug_view_scroll_up_at_zero() {
        let mut view = DebugView::new();
        view.scroll_offset = 0;

        view.scroll_up();
        assert_eq!(view.scroll_offset, 0); // Shouldn't go negative
    }

    #[test]
    fn test_debug_view_scroll_down() {
        let mut view = DebugView::new();
        view.scroll_offset = 0;

        view.scroll_down();
        assert_eq!(view.scroll_offset, 1);
    }
    // TODO
    // #[test]
    // fn test_debug_view_select_next_snapshot() {
    //     let mut view = DebugView::new();
    //     assert_eq!(view.selected_snapshot, None);
    //
    //     view.select_next_snapshot(10);
    //     assert_eq!(view.selected_snapshot, Some(0));
    //
    //     view.select_next_snapshot(10);
    //     assert_eq!(view.selected_snapshot, Some(1));
    // }
    // TODO
    // #[test]
    // fn test_debug_view_select_next_snapshot_respects_max() {
    //     let mut view = DebugView::new();
    //     view.selected_snapshot = Some(9);
    //
    //     view.select_next_snapshot(9); // Max is 9
    //     assert_eq!(view.selected_snapshot, Some(9)); // Shouldn't exceed
    // }
    // TODO
    // #[test]
    // fn test_debug_view_select_prev_snapshot() {
    //     let mut view = DebugView::new();
    //     view.selected_snapshot = Some(5);
    //
    //     view.select_prev_snapshot();
    //     assert_eq!(view.selected_snapshot, Some(4));
    //
    //     view.select_prev_snapshot();
    //     assert_eq!(view.selected_snapshot, Some(3));
    // }
    // TODO
    // #[test]
    // fn test_debug_view_select_prev_snapshot_at_zero() {
    //     let mut view = DebugView::new();
    //     view.selected_snapshot = Some(0);
    //
    //     view.select_prev_snapshot();
    //     assert_eq!(view.selected_snapshot, None); // Goes to None
    // }
    //
    // TODO
    // #[test]
    // fn test_debug_view_open_snapshot_viewer() {
    //     let mut view = DebugView::new();
    //     view.selected_snapshot = Some(5);
    //
    //     view.open_snapshot_viewer();
    //     assert!(view.viewing_snapshot);
    // }
    // TODO
    // #[test]
    // fn test_debug_view_open_snapshot_viewer_without_selection() {
    //     let mut view = DebugView::new();
    //     view.selected_snapshot = None;
    //
    //     view.open_snapshot_viewer();
    //     assert!(!view.viewing_snapshot); // Shouldn't open without selection
    // }

    // TODO
    // #[test]
    // fn test_debug_view_close_snapshot_viewer() {
    //     let mut view = DebugView::new();
    //     view.viewing_snapshot = true;
    //
    //     view.close_snapshot_viewer();
    //     assert!(!view.viewing_snapshot);
    // }

    #[test]
    fn test_debug_view_tab_change_resets_scroll() {
        let mut view = DebugView::new();
        view.scroll_offset = 10;

        view.next_tab();
        assert_eq!(view.scroll_offset, 0);
    }
}

#[cfg(test)]
mod debug_integration_tests {
    use calliglyph::core::app::ActiveArea;
    use calliglyph::core::cursor::Cursor;
    use calliglyph::core::debug::{CaptureMode, DebugState, LogLevel, SnapshotTrigger};

    #[test]
    fn test_full_debug_workflow() {
        let mut debug_state = DebugState::new();

        // Enable debugging
        debug_state.enabled = true;

        // Log some events
        debug_state.log(LogLevel::Info, "Application started");
        debug_state.log(LogLevel::Debug, "Loading file");
        debug_state.log(LogLevel::Error, "File not found");

        assert_eq!(debug_state.logger.entries().len(), 3);
        assert_eq!(debug_state.logger.count_by_level(LogLevel::Error), 1);

        // Tick some frames
        debug_state.tick_frame();
        debug_state.tick_frame();
        assert_eq!(debug_state.metrics.render_count, 2);

        // Capture a snapshot on error
        debug_state.update_and_maybe_snapshot(
            ActiveArea::Editor,
            Some(SnapshotTrigger::Error("File not found".to_string())),
            Cursor { x: 0, y: 0 },
            None,
            vec!["".to_string()],
            0,
            vec![],
            vec![],
            vec![],
            None,
        );

        assert_eq!(debug_state.snapshots.len(), 1);

        // Capture manual snapshot
        debug_state.capture_manual_snapshot(
            ActiveArea::Editor,
            Cursor { x: 5, y: 10 },
            None,
            vec!["Hello".to_string()],
            0,
            vec![],
            vec![],
            vec![],
            None,
        );

        assert_eq!(debug_state.snapshots.len(), 2);
    }

    #[test]
    fn test_debug_mode_switching() {
        let mut debug_state = DebugState::new();
        debug_state.enabled = true;

        // Start in OnEvent mode
        assert_eq!(debug_state.capture_mode, CaptureMode::OnEvent);

        // Switch to None - should not capture
        debug_state.set_capture_mode(CaptureMode::None);
        debug_state.update_and_maybe_snapshot(
            ActiveArea::Editor,
            Some(SnapshotTrigger::Error("test".to_string())),
            Cursor { x: 0, y: 0 },
            None,
            vec![],
            0,
            vec![],
            vec![],
            vec![],
            None,
        );
        assert_eq!(debug_state.snapshots.len(), 0);

        // Switch to EveryFrame - should capture
        debug_state.set_capture_mode(CaptureMode::EveryFrame);
        debug_state.update_and_maybe_snapshot(
            ActiveArea::Editor,
            None,
            Cursor { x: 0, y: 0 },
            None,
            vec![],
            0,
            vec![],
            vec![],
            vec![],
            None,
        );
        assert_eq!(debug_state.snapshots.len(), 1);
    }

    #[test]
    fn test_snapshot_history_ordering() {
        let mut debug_state = DebugState::new();
        debug_state.enabled = true;

        for i in 0..5 {
            debug_state.capture_manual_snapshot(
                ActiveArea::Editor,
                Cursor { x: i, y: i },
                None,
                vec![format!("line {}", i)],
                0,
                vec![],
                vec![],
                vec![],
                None,
            );
        }

        // Latest should be the last one added
        let latest = debug_state.snapshots.latest().unwrap();
        let pos = (latest.cursor_pos.x, latest.cursor_pos.y);
        assert_eq!(pos, (4, 4));

        // First should be the oldest
        let first = debug_state.snapshots.get(0).unwrap();
        let pos = (first.cursor_pos.x, first.cursor_pos.y);
        assert_eq!(pos, (0, 0));
    }
}
