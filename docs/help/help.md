---
id: help
title: Help
summary: How to use the help system
tags: help, :help, h, topics, search
---

# Help

The help system provides documentation for all built-in commands, features,
and plugins in calli-glyph.

## Commands

| Command           | Description                                      |
|-------------------|--------------------------------------------------|
| `:help`           | Open the help popup (all topics)                 |
| `:help <topic>`   | Jump directly to a topic, e.g. `:help debug`     |
| `:h`              | Alias for `:help`                                |
| `:<cmd> help`     | Show help for a specific command, e.g. `:debug help` |

## Navigation

| Key        | Action                              |
|------------|-------------------------------------|
| `↑` / `↓`  | Move through the topic list         |
| `Ctrl+u`   | Scroll content up                   |
| `Ctrl+d`   | Scroll content down                 |
| `/`        | Activate search bar                 |
| `Esc`      | Close search / close help           |

## Search

Press `/` to activate the search bar. Type to filter topics in real time.
Search matches against topic titles, summaries, and tags.

Press `Esc` once to clear the search, and again to close the popup.

## Topics

| Topic     | Description                                      |
|-----------|--------------------------------------------------|
| `editor`  | Core editing, movement, and file operations      |
| `debug`   | Developer console and debug overlay              |
| `config`  | Configuration file and available options         |
| `plugins` | Plugin system and management                     |
| `help`    | This page                                        |
