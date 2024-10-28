<div align="center">

**&gt; English &lt;** | [简体中文](README_zh.md)

</div>

# Gemini Client Based on Command Line Keyboard Operations

## Introduction

This is a terminal command-line tool for interacting with the Gemini API, providing a simple terminal interface for engaging with the Gemini model.

## Installation and Usage

1. Go to [Google AI Studio](https://aistudio.google.com/app/apikey) to generate an API key.

2. In the command line, enter `./gemini` to run the Gemini client, input your API key when prompted, and press Enter to confirm.

3. Start using the client.

## Key Functions

### Chat Interface

#### Common Key Functions

| Key     | Function                     |
|---------|------------------------------|
| `Esc`   | Exit the program             |
| `Tab`   | Switch focus between components |
| `F3/Ctrl+s` | Show or hide the sidebar     |

#### Unique Key Functions

1. When focused on the input box:

    | Key          | Function                              |
    |--------------|---------------------------------------|
    | `Enter`      | Send message                          |
    | `F1/Ctrl+t`  | Edit title                            |
    | `F4/Ctrl+i`  | Insert an image corresponding to the input box path or delete image |
    | `Backspace`  | Delete the character before the cursor |
    | `Delete`     | Delete the character after the cursor |
    | `Left`       | Move cursor left                      |
    | `Right`      | Move cursor right                     |
    | `Home`       | Move cursor to the beginning of the line |
    | `End`        | Move cursor to the end of the line   |
    | `Character Key` | Input character                    |

2. When focused on the chat content display area:

    | Key     | Function                      |
    |---------|-------------------------------|
    | `F1/Ctrl+t` | Edit title                  |
    | `Up`    | Scroll messages up            |
    | `Down`  | Scroll messages down          |

3. When focused on the new chat button:

    | Key     | Function                      |
    |---------|-------------------------------|
    | `Enter` | Start a new chat              |

4. When focused on the chat list:

    | Key     | Function                      |
    |---------|-------------------------------|
    | `Up`    | Previous chat record          |
    | `Down`  | Next chat record              |
    | `Delete` | Delete chat record (requires confirmation) |
    | `Enter` | Load chat record               |

5. When focused on the settings button:

    | Key     | Function                      |
    |---------|-------------------------------|
    | `Enter` | Enter settings menu            |

### Settings Interface

| Key     | Function                      |
|---------|-------------------------------|
| `Esc`   | Exit settings menu            |
| `Tab`   | Switch focus between components |
| `F2/Ctrl+s` | Save settings and exit menu |
| `Enter` | Insert new line (if applicable) |
| `Backspace` | Delete the character before the cursor |
| `Delete` | Delete the character after the cursor |
| `Left`   | Move cursor left              |
| `Right`  | Move cursor right             |
| `Home`   | Move cursor to the beginning of the line |
| `End`    | Move cursor to the end of the line |
| `Character Key` | Input character        |
