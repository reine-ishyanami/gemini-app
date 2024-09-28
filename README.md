<!--
 Copyright (C) 2024 reine-ishyanami

 This program is free software: you can redistribute it and/or modify
 it under the terms of the GNU Affero General Public License as
 published by the Free Software Foundation, either version 3 of the
 License, or (at your option) any later version.

 This program is distributed in the hope that it will be useful,
 but WITHOUT ANY WARRANTY; without even the implied warranty of
 MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 GNU Affero General Public License for more details.

 You should have received a copy of the GNU Affero General Public License
 along with this program.  If not, see <https://www.gnu.org/licenses/>.
-->

# Gemini Api For Tui

This is a Rust library for interacting with the Gemini API. It provides a simple interface for sending and receiving Gemini messages. and also provides a simple terminal ui for interacte with Google Gemini.

## Usage

1. Go to [Google AI Studio](https://aistudio.google.com/app/apikey) to generate you api key.

2. set the environment variable `GEMINI_KEY` with your api key. (Optional: you can also set the api key when application started)

3. download the application from release page or build it from source.

4. enter something and press enter to ask the gemini.

5. press tab to focus other components. like `input`, `esc`, `set` and so on.

6. input a image path or url in the `input` textfield and press f4 to store the image for sending the request with the image in next time.

7. press f4 to clear the image when setted the image.

8. press tab to switch focus next component.

## ToDo

- [ ] history chat

- [ ] fix the textfield bug

- [ ] fix the textarea bug
