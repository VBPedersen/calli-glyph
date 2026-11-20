---
id: dialogs
title: Menus & Dialogs
summary: Shared keys for menus, lists and dialogs, plus text entry and key capture
tags: dialogs, modals, menus, navigation, keys, tab, enter, esc, text input, key capture, confirm, hotkeys, shortcuts
---

# Menus & Dialogs

The menu, settings, controls, theme browser, theme editor, file browser and
similar screens all share the same basic keys, so once you know one you know
them all.

## Common keys

| Key                    | Action                                                     |
|------------------------|------------------------------------------------------------|
| `↑` / `k`              | Move up                                                    |
| `↓` / `j`              | Move down                                                  |
| `Tab` / `Shift+Tab`    | Next / previous tab or category                            |
| `Enter`                | Confirm, run, or change the selected item                  |
| `Esc`                  | Close, or cancel what you are doing                        |
| A letter               | Entry-specific shortcut, shown as `[x]` on the row or in the footer |

The footer of each screen lists the keys that work there. Letter shortcuts
are case sensitive: `r` and `R` can mean different things.

## Destructive actions

Actions that reset a lot at once, such as restoring a built-in theme or
resetting a tab of key bindings, use `R` and ask you to press it a second time.
Any other key cancels the confirmation.

## Text entry

Some dialogs ask you to type, for example the GitHub token, a new theme name
or a color value. While one is open:

| Key                | Action                                                      |
|--------------------|-------------------------------------------------------------|
| any character      | Added to the input, including capital letters and symbols   |
| `Backspace`        | Delete the last character                                   |
| `Enter`            | Confirm                                                     |
| `Esc`              | Cancel the input                                            |

Letters do not trigger shortcuts while you are typing. Pasting works: the
pasted text is entered like typed characters.

## Key capture

When the controls screen asks you to press a key combination, the next key you
press is recorded as the new binding instead of running. `Esc` cancels. To
bind `Esc` itself, type its name instead, see `:help controls`.

## Stacked screens

Screens open on top of each other. Closing one with `Esc` returns you to what
was underneath, such as the welcome screen. Some screens hand over to another
when you pick an action, for example editing a theme from the theme browser
replaces the browser with the editor.

## Related

| Topic         | Where                                         |
|---------------|-----------------------------------------------|
| Menu          | `:help menu`                                  |
| Settings      | `:help settings`                              |
| Controls      | `:help controls`                              |
| Themes        | `:help themes`                                |
