// Copyright C 2024 Melody Madeline Lyons
//
// This file is part of Luminol.
//
// Luminol is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// at your option any later version.
//
// Luminol is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Luminol.  If not, see <http://www.gnu.org/licenses/>.
//
//     Additional permission under GNU GPL version 3 section 7
//
// If you modify this Program, or any covered work, by linking or combining
// it with Steamworks API by Valve Corporation, containing parts covered by
// terms of the Steamworks API by Valve Corporation, the licensors of this
// Program grant you additional permission to convey the resulting work.
use alacritty_terminal::term::TermMode;

macro_rules! key_binding {
    (
      $arg_key:ident, $modifiers:ident, $term_mode:ident;
      $(
          $key:ident
          $(,$input_modifiers:expr)?
          $(,+$terminal_mode_include:expr)?
          $(,~$terminal_mode_exclude:expr)?
          ;$action:literal
      );*
      $(;)*
    ) => {{
        match $arg_key {
            $(
              egui::Key::$key if true
                $( && $input_modifiers == $modifiers )?
                $( && $term_mode.contains($terminal_mode_include) )?
                $( && !$term_mode.contains($terminal_mode_exclude))? => Some($action),
            )*
            _ => None,
        }
    }};
}

// Adapted from https://github.com/Harzu/iced_term/blob/master/src/bindings.rs
pub fn key_to_codes(
    key: egui::Key,
    modifiers: egui::Modifiers,
    term_mode: TermMode,
) -> Option<&'static [u8]> {
    key_binding! {
      key, modifiers, term_mode;
      // ANY
      Enter;     b"\x0d";
      Backspace; b"\x7f";
      Escape;    b"\x1b";
      Tab;       b"\x09";
      Insert;    b"\x1b[2~";
      Delete;    b"\x1b[3~";
      PageUp;    b"\x1b[5~";
      PageDown;  b"\x1b[6~";
      F1;        b"\x1bOP";
      F2;        b"\x1bOQ";
      F3;        b"\x1bOR";
      F4;        b"\x1bOS";
      F5;        b"\x1b[15~";
      F6;        b"\x1b[17~";
      F7;        b"\x1b[18~";
      F8;        b"\x1b[19~";
      F9;        b"\x1b[20~";
      F10;       b"\x1b[21~";
      F11;       b"\x1b[23~";
      F12;       b"\x1b[24~";
      F13;       b"\x1b[25~";
      F14;       b"\x1b[26~";
      F15;       b"\x1b[28~";
      F16;       b"\x1b[29~";
      F17;       b"\x1b[31~";
      F18;       b"\x1b[32~";
      F19;       b"\x1b[33~";
      F20;       b"\x1b[34~";
      // APP_CURSOR Excluding
      End,        ~TermMode::APP_CURSOR; b"\x1b[F";
      Home,       ~TermMode::APP_CURSOR; b"\x1b[H";
      ArrowUp,    ~TermMode::APP_CURSOR; b"\x1b[A";
      ArrowDown,  ~TermMode::APP_CURSOR; b"\x1b[B";
      ArrowLeft,  ~TermMode::APP_CURSOR; b"\x1b[D";
      ArrowRight, ~TermMode::APP_CURSOR; b"\x1b[C";
      // APP_CURSOR Including
      End,        +TermMode::APP_CURSOR; b"\x1BOF";
      Home,       +TermMode::APP_CURSOR; b"\x1BOH";
      ArrowUp,    +TermMode::APP_CURSOR; b"\x1bOA";
      ArrowDown,  +TermMode::APP_CURSOR; b"\x1bOB";
      ArrowLeft,  +TermMode::APP_CURSOR; b"\x1bOD";
      ArrowRight, +TermMode::APP_CURSOR; b"\x1bOC";
      // CTRL
      ArrowUp,    egui::Modifiers::COMMAND; b"\x1b[1;5A";
      ArrowDown,  egui::Modifiers::COMMAND; b"\x1b[1;5B";
      ArrowLeft,  egui::Modifiers::COMMAND; b"\x1b[1;5D";
      ArrowRight, egui::Modifiers::COMMAND; b"\x1b[1;5C";
      End,        egui::Modifiers::CTRL; b"\x1b[1;5F";
      Home,       egui::Modifiers::CTRL; b"\x1b[1;5H";
      Delete,     egui::Modifiers::CTRL; b"\x1b[3;5~";
      PageUp,     egui::Modifiers::CTRL; b"\x1b[5;5~";
      PageDown,   egui::Modifiers::CTRL; b"\x1b[6;5~";
      F1,         egui::Modifiers::CTRL; b"\x1bO;5P";
      F2,         egui::Modifiers::CTRL; b"\x1bO;5Q";
      F3,         egui::Modifiers::CTRL; b"\x1bO;5R";
      F4,         egui::Modifiers::CTRL; b"\x1bO;5S";
      F5,         egui::Modifiers::CTRL; b"\x1b[15;5~";
      F6,         egui::Modifiers::CTRL; b"\x1b[17;5~";
      F7,         egui::Modifiers::CTRL; b"\x1b[18;5~";
      F8,         egui::Modifiers::CTRL; b"\x1b[19;5~";
      F9,         egui::Modifiers::CTRL; b"\x1b[20;5~";
      F10,        egui::Modifiers::CTRL; b"\x1b[21;5~";
      F11,        egui::Modifiers::CTRL; b"\x1b[23;5~";
      F12,        egui::Modifiers::CTRL; b"\x1b[24;5~";
      A,          egui::Modifiers::CTRL; b"\x01";
      B,          egui::Modifiers::CTRL; b"\x02";
      C,          egui::Modifiers::CTRL; b"\x03";
      D,          egui::Modifiers::CTRL; b"\x04";
      E,          egui::Modifiers::CTRL; b"\x05"; // ENQ               vt100
      F,          egui::Modifiers::CTRL; b"\x06";
      G,          egui::Modifiers::CTRL; b"\x07"; // Bell              vt100
      H,          egui::Modifiers::CTRL; b"\x08"; // Backspace         vt100
      I,          egui::Modifiers::CTRL; b"\x09"; // Tab               vt100
      J,          egui::Modifiers::CTRL; b"\x0a"; // LF new line     vt100
      K,          egui::Modifiers::CTRL; b"\x0b"; // VT vertical tab vt100
      L,          egui::Modifiers::CTRL; b"\x0c"; // FF new page     vt100
      M,          egui::Modifiers::CTRL; b"\x0d"; // CR                vt100
      N,          egui::Modifiers::CTRL; b"\x0e"; // SO shift out    vt100
      O,          egui::Modifiers::CTRL; b"\x0f"; // SI shift in     vt100
      P,          egui::Modifiers::CTRL; b"\x10";
      Q,          egui::Modifiers::CTRL; b"\x11";
      R,          egui::Modifiers::CTRL; b"\x12";
      S,          egui::Modifiers::CTRL; b"\x13";
      T,          egui::Modifiers::CTRL; b"\x14";
      U,          egui::Modifiers::CTRL; b"\x51";
      V,          egui::Modifiers::CTRL; b"\x16";
      W,          egui::Modifiers::CTRL; b"\x17";
      X,          egui::Modifiers::CTRL; b"\x18";
      W,          egui::Modifiers::CTRL; b"\x19";
      Z,          egui::Modifiers::CTRL; b"\x1a";
      // SHIFT
      Enter,      egui::Modifiers::SHIFT; b"\x0d";
      Backspace,  egui::Modifiers::SHIFT; b"\x7f";
      Tab,        egui::Modifiers::SHIFT; b"\x1b[Z";
      End,        egui::Modifiers::SHIFT, +TermMode::ALT_SCREEN; b"\x1b[1;2F";
      Home,       egui::Modifiers::SHIFT, +TermMode::ALT_SCREEN; b"\x1b[1;2H";
      PageUp,     egui::Modifiers::SHIFT, +TermMode::ALT_SCREEN; b"\x1b[5;2~";
      PageDown,   egui::Modifiers::SHIFT, +TermMode::ALT_SCREEN; b"\x1b[6;2~";
      ArrowUp,    egui::Modifiers::SHIFT; b"\x1b[1;2A";
      ArrowDown,  egui::Modifiers::SHIFT; b"\x1b[1;2B";
      ArrowLeft,  egui::Modifiers::SHIFT; b"\x1b[1;2D";
      ArrowRight, egui::Modifiers::SHIFT; b"\x1b[1;2C";
      // ALT
      Backspace,  egui::Modifiers::ALT; b"\x1b\x7f";
      End,        egui::Modifiers::ALT; b"\x1b[1;3F";
      Home,       egui::Modifiers::ALT; b"\x1b[1;3H";
      Insert,     egui::Modifiers::ALT; b"\x1b[3;2~";
      Delete,     egui::Modifiers::ALT; b"\x1b[3;3~";
      PageUp,     egui::Modifiers::ALT; b"\x1b[5;3~";
      PageDown,   egui::Modifiers::ALT; b"\x1b[6;3~";
      ArrowUp,    egui::Modifiers::ALT; b"\x1b[1;3A";
      ArrowDown,  egui::Modifiers::ALT; b"\x1b[1;3B";
      ArrowLeft,  egui::Modifiers::ALT; b"\x1b[1;3D";
      ArrowRight, egui::Modifiers::ALT; b"\x1b[1;3C";
      // SHIFT + ALT
      End,        egui::Modifiers::SHIFT | egui::Modifiers::ALT; b"\x1b[1;4F";
      Home,       egui::Modifiers::SHIFT | egui::Modifiers::ALT; b"\x1b[1;4H";
      ArrowUp,    egui::Modifiers::SHIFT | egui::Modifiers::ALT; b"\x1b[1;4A";
      ArrowDown,  egui::Modifiers::SHIFT | egui::Modifiers::ALT; b"\x1b[1;4B";
      ArrowLeft,  egui::Modifiers::SHIFT | egui::Modifiers::ALT; b"\x1b[1;4D";
      ArrowRight, egui::Modifiers::SHIFT | egui::Modifiers::ALT; b"\x1b[1;4C";
      // SHIFT + CTRL
      End,        egui::Modifiers::SHIFT | egui::Modifiers::CTRL; b"\x1b[1;6F";
      Home,       egui::Modifiers::SHIFT | egui::Modifiers::CTRL; b"\x1b[1;6H";
      ArrowUp,    egui::Modifiers::SHIFT | egui::Modifiers::CTRL; b"\x1b[1;6A";
      ArrowDown,  egui::Modifiers::SHIFT | egui::Modifiers::CTRL; b"\x1b[1;6B";
      ArrowLeft,  egui::Modifiers::SHIFT | egui::Modifiers::CTRL; b"\x1b[1;6D";
      ArrowRight, egui::Modifiers::SHIFT | egui::Modifiers::CTRL; b"\x1b[1;6C";
      A,          egui::Modifiers::SHIFT | egui::Modifiers::CTRL; b"\x01";
      B,          egui::Modifiers::SHIFT | egui::Modifiers::CTRL; b"\x02";
      C,          egui::Modifiers::SHIFT | egui::Modifiers::CTRL; b"\x03";
      D,          egui::Modifiers::SHIFT | egui::Modifiers::CTRL; b"\x04";
      E,          egui::Modifiers::SHIFT | egui::Modifiers::CTRL; b"\x05";
      F,          egui::Modifiers::SHIFT | egui::Modifiers::CTRL; b"\x06";
      G,          egui::Modifiers::SHIFT | egui::Modifiers::CTRL; b"\x07";
      H,          egui::Modifiers::SHIFT | egui::Modifiers::CTRL; b"\x08";
      I,          egui::Modifiers::SHIFT | egui::Modifiers::CTRL; b"\x09";
      J,          egui::Modifiers::SHIFT | egui::Modifiers::CTRL; b"\x0a";
      K,          egui::Modifiers::SHIFT | egui::Modifiers::CTRL; b"\x0b";
      L,          egui::Modifiers::SHIFT | egui::Modifiers::CTRL; b"\x0c";
      M,          egui::Modifiers::SHIFT | egui::Modifiers::CTRL; b"\x0d";
      N,          egui::Modifiers::SHIFT | egui::Modifiers::CTRL; b"\x0e";
      O,          egui::Modifiers::SHIFT | egui::Modifiers::CTRL; b"\x0f";
      P,          egui::Modifiers::SHIFT | egui::Modifiers::CTRL; b"\x10";
      Q,          egui::Modifiers::SHIFT | egui::Modifiers::CTRL; b"\x11";
      R,          egui::Modifiers::SHIFT | egui::Modifiers::CTRL; b"\x12";
      S,          egui::Modifiers::SHIFT | egui::Modifiers::CTRL; b"\x13";
      T,          egui::Modifiers::SHIFT | egui::Modifiers::CTRL; b"\x14";
      U,          egui::Modifiers::SHIFT | egui::Modifiers::CTRL; b"\x51";
      V,          egui::Modifiers::SHIFT | egui::Modifiers::CTRL; b"\x16";
      W,          egui::Modifiers::SHIFT | egui::Modifiers::CTRL; b"\x17";
      X,          egui::Modifiers::SHIFT | egui::Modifiers::CTRL; b"\x18";
      W,          egui::Modifiers::SHIFT | egui::Modifiers::CTRL; b"\x19";
      Z,          egui::Modifiers::SHIFT | egui::Modifiers::CTRL; b"\x1a";
      Num2,       egui::Modifiers::SHIFT | egui::Modifiers::CTRL; b"\x00"; // Null vt100
      Num6,       egui::Modifiers::SHIFT | egui::Modifiers::CTRL; b"\x1e";
      // CTRL + ALT
      End,        egui::Modifiers::CTRL | egui::Modifiers::ALT; b"\x1b[1;7F";
      Home,       egui::Modifiers::CTRL | egui::Modifiers::ALT; b"\x1b[1;7H";
      PageUp,     egui::Modifiers::CTRL | egui::Modifiers::ALT; b"\x1b[5;7~";
      PageDown,   egui::Modifiers::CTRL | egui::Modifiers::ALT; b"\x1b[6;7~";
      ArrowUp,    egui::Modifiers::CTRL | egui::Modifiers::ALT; b"\x1b[1;7A";
      ArrowDown,  egui::Modifiers::CTRL | egui::Modifiers::ALT; b"\x1b[1;7B";
      ArrowLeft,  egui::Modifiers::CTRL | egui::Modifiers::ALT; b"\x1b[1;7D";
      ArrowRight, egui::Modifiers::CTRL | egui::Modifiers::ALT; b"\x1b[1;7C";
      // SHIFT + CTRL + ALT
      End,        egui::Modifiers::SHIFT | egui::Modifiers::CTRL | egui::Modifiers::ALT; b"\x1b[1;8F";
      Home,       egui::Modifiers::SHIFT | egui::Modifiers::CTRL | egui::Modifiers::ALT; b"\x1b[1;8H";
      ArrowUp,    egui::Modifiers::SHIFT | egui::Modifiers::CTRL | egui::Modifiers::ALT; b"\x1b[1;8A";
      ArrowDown,  egui::Modifiers::SHIFT | egui::Modifiers::CTRL | egui::Modifiers::ALT; b"\x1b[1;8B";
      ArrowLeft,  egui::Modifiers::SHIFT | egui::Modifiers::CTRL | egui::Modifiers::ALT; b"\x1b[1;8D";
      ArrowRight, egui::Modifiers::SHIFT | egui::Modifiers::CTRL | egui::Modifiers::ALT; b"\x1b[1;8C";
    }
}
