---
id: githubtoken
title: GitHub Token
summary: Set or remove the GitHub token used when installing language grammars
tags: github, token, :githubtoken, :ghtoken, grammar, install, language, rate limit, authentication, credentials, dialog
---

# GitHub Token

Language grammars are installed by cloning their GitHub repositories. Without
a token GitHub treats these clones as anonymous and limits how many you can do.
A personal access token lifts that limit. It is optional.

## Commands

| Command                    | Description                                              |
|----------------------------|----------------------------------------------------------|
| `:githubtoken`             | Open the token dialog (recommended)                      |
| `:ghtoken`                 | Alias for `:githubtoken`                                 |
| `:githubtoken <token>`     | Set the token directly                                   |
| `:githubtoken clear`       | Remove the stored token                                  |
| Menu                       | Language, then GitHub token (key `g`)                    |

## The token dialog

The dialog shows whether a token is stored (`set ✓` or `none`), and an input
line for a new one.

| Key          | Action                                                     |
|--------------|------------------------------------------------------------|
| type / paste | Enter the token                                            |
| `Backspace`  | Delete the last character                                  |
| `Enter`      | Save the token and close                                   |
| `Tab`        | Remove the stored token                                    |
| `Esc`        | Cancel                                                     |

While you type, the token is hidden behind dots. Only the last four
characters are visible and the length is shown below, so you can check that a
paste was complete.

Whitespace around the token is ignored. If you press `Enter` with an empty
input, nothing is saved and you are asked to paste a token first.

## Creating a token

Create a personal access token in your GitHub account settings. No special
permissions are needed to clone public repositories.

## Where it is stored

The token is kept in a separate credentials file in your config directory. It
is not written to `config.toml`, so sharing your config does not share the
token. It is used for all future grammar installs.

## Why use the dialog

With `:githubtoken <token>` the token is briefly visible on the command line
while you type or paste it. The dialog masks it, so prefer the dialog unless
you are scripting.

## Related

| Topic        | Where                                               |
|--------------|-----------------------------------------------------|
| Plugins      | `:help plugins`                                     |
| Menu         | `:help menu`                                        |
