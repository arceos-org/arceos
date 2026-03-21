// Minimal line editor for raw serial console (QEMU -nographic).
//
// Features:
//   - Character echo (including multibyte UTF-8 / Chinese)
//   - Left / Right arrow cursor movement (wide-char aware)
//   - Home / End
//   - Backspace / Delete (UTF-8-safe, CJK display-width aware)
//   - Insert in the middle of the line with redraw
//   - Command history (Up / Down arrows)
//
// No external crate dependencies — works on ArceOS Hermit target.

use std::io::{self, Read, Write};

// ---------------------------------------------------------------------------
// Display width helpers
// ---------------------------------------------------------------------------

/// Returns the display width of a character (1 for ASCII, 2 for CJK, 0 for
/// zero-width / control chars).
fn char_width(c: char) -> usize {
    let cp = c as u32;
    // Control characters
    if cp < 0x20 || cp == 0x7F {
        return 0;
    }
    // Common CJK ranges (width 2)
    if (0x1100..=0x115F).contains(&cp)      // Hangul Jamo
        || (0x2E80..=0x303E).contains(&cp)  // CJK Radicals, Kangxi, CJK Symbols
        || (0x3040..=0x33BF).contains(&cp)  // Hiragana, Katakana, CJK Compat
        || (0x3400..=0x4DBF).contains(&cp)  // CJK Ext A
        || (0x4E00..=0x9FFF).contains(&cp)  // CJK Unified Ideographs
        || (0xA000..=0xA4CF).contains(&cp)  // Yi
        || (0xAC00..=0xD7AF).contains(&cp)  // Hangul Syllables
        || (0xF900..=0xFAFF).contains(&cp)  // CJK Compat Ideographs
        || (0xFE30..=0xFE6F).contains(&cp)  // CJK Compat Forms
        || (0xFF01..=0xFF60).contains(&cp)  // Fullwidth Forms
        || (0xFFE0..=0xFFE6).contains(&cp)  // Fullwidth Signs
        || (0x20000..=0x2FA1F).contains(&cp) // CJK Ext B-F, Compat Supplement
        || (0x30000..=0x3134F).contains(&cp)
    // CJK Ext G
    {
        return 2;
    }
    1
}

/// Display width of a string.
fn str_width(s: &str) -> usize {
    s.chars().map(char_width).sum()
}

// ---------------------------------------------------------------------------
// ANSI helpers
// ---------------------------------------------------------------------------

fn write_bytes(out: &mut impl Write, bytes: &[u8]) {
    let _ = out.write_all(bytes);
}

fn move_cursor_left(out: &mut impl Write, n: usize) {
    if n > 0 {
        let _ = write!(out, "\x1b[{}D", n);
    }
}

fn move_cursor_right(out: &mut impl Write, n: usize) {
    if n > 0 {
        let _ = write!(out, "\x1b[{}C", n);
    }
}

fn clear_to_eol(out: &mut impl Write) {
    write_bytes(out, b"\x1b[K");
}

// ---------------------------------------------------------------------------
// Line buffer — stores the line as a String with a byte-position cursor
// ---------------------------------------------------------------------------

struct LineBuffer {
    buf: String,
    /// Cursor position as a **byte** offset into `buf`.
    cursor: usize,
}

impl LineBuffer {
    fn new() -> Self {
        Self {
            buf: String::new(),
            cursor: 0,
        }
    }

    fn from(s: &str) -> Self {
        let len = s.len();
        Self {
            buf: s.to_string(),
            cursor: len,
        }
    }

    /// Insert a character at the cursor.
    fn insert(&mut self, c: char) {
        self.buf.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    /// Delete the character before the cursor. Returns the deleted char.
    fn backspace(&mut self) -> Option<char> {
        if self.cursor == 0 {
            return None;
        }
        // Walk back to the start of the previous UTF-8 char
        let before = &self.buf[..self.cursor];
        let c = before.chars().next_back()?;
        self.cursor -= c.len_utf8();
        self.buf.remove(self.cursor);
        Some(c)
    }

    /// Delete the character at the cursor. Returns the deleted char.
    fn delete(&mut self) -> Option<char> {
        if self.cursor >= self.buf.len() {
            return None;
        }
        let c = self.buf[self.cursor..].chars().next()?;
        self.buf.remove(self.cursor);
        Some(c)
    }

    /// Move cursor one character to the left. Returns the char skipped.
    fn move_left(&mut self) -> Option<char> {
        if self.cursor == 0 {
            return None;
        }
        let before = &self.buf[..self.cursor];
        let c = before.chars().next_back()?;
        self.cursor -= c.len_utf8();
        Some(c)
    }

    /// Move cursor one character to the right. Returns the char skipped.
    fn move_right(&mut self) -> Option<char> {
        if self.cursor >= self.buf.len() {
            return None;
        }
        let c = self.buf[self.cursor..].chars().next()?;
        self.cursor += c.len_utf8();
        Some(c)
    }

    /// Move cursor to start. Returns display-width moved.
    fn move_home(&mut self) -> usize {
        let w = str_width(&self.buf[..self.cursor]);
        self.cursor = 0;
        w
    }

    /// Move cursor to end. Returns display-width moved.
    fn move_end(&mut self) -> usize {
        let w = str_width(&self.buf[self.cursor..]);
        self.cursor = self.buf.len();
        w
    }

    /// Text after cursor (for redraw).
    fn after_cursor(&self) -> &str {
        &self.buf[self.cursor..]
    }

    fn is_at_end(&self) -> bool {
        self.cursor >= self.buf.len()
    }

    fn as_str(&self) -> &str {
        &self.buf
    }

    fn clear(&mut self) {
        self.buf.clear();
        self.cursor = 0;
    }
}

// ---------------------------------------------------------------------------
// Escape sequence parser (state machine)
// ---------------------------------------------------------------------------

enum EscState {
    Normal,
    Esc,              // got \x1b
    Bracket,          // got \x1b[
    BracketDigit(u8), // got \x1b[<digit>
}

enum Key {
    Char(char),
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    Enter,
    CtrlC,
    CtrlD,
    CtrlA,
    CtrlE,
    CtrlU,
    CtrlK,
    CtrlL,
    Ignore,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub struct LineEditor {
    history: Vec<String>,
    max_history: usize,
}

impl LineEditor {
    pub fn new(max_history: usize) -> Self {
        Self {
            history: Vec::new(),
            max_history,
        }
    }

    /// Read one line from stdin with editing support.
    /// Returns `None` on EOF / Ctrl-D on empty line.
    pub fn read_line(&mut self, prompt: &str) -> Option<String> {
        let mut stdout = io::stdout();
        let stdin = io::stdin();

        // Print prompt
        write_bytes(&mut stdout, prompt.as_bytes());
        let _ = stdout.flush();

        let mut lb = LineBuffer::new();
        let mut esc = EscState::Normal;
        let mut utf8_buf: Vec<u8> = Vec::new();
        let mut utf8_remaining: usize = 0;

        // History navigation state
        let mut hist_idx: Option<usize> = None; // None = editing new line
        let mut saved_line = String::new(); // saved current input when browsing history

        let mut byte_buf = [0u8; 1];

        loop {
            match stdin.lock().read(&mut byte_buf) {
                Ok(0) => return None, // EOF
                Ok(_) => {}
                Err(_) => return None,
            }
            let b = byte_buf[0];

            // UTF-8 multi-byte accumulation
            if utf8_remaining > 0 {
                if b & 0xC0 == 0x80 {
                    utf8_buf.push(b);
                    utf8_remaining -= 1;
                    if utf8_remaining == 0 {
                        if let Ok(s) = std::str::from_utf8(&utf8_buf) {
                            for c in s.chars() {
                                self.handle_key(
                                    Key::Char(c),
                                    &mut lb,
                                    &mut stdout,
                                    &mut hist_idx,
                                    &mut saved_line,
                                    prompt,
                                );
                            }
                        }
                        utf8_buf.clear();
                    }
                    continue;
                } else {
                    // Invalid continuation — discard and fall through
                    utf8_buf.clear();
                    utf8_remaining = 0;
                }
            }

            // Escape sequence state machine
            match esc {
                EscState::Normal => {
                    if b == 0x1b {
                        esc = EscState::Esc;
                        continue;
                    }
                }
                EscState::Esc => {
                    if b == b'[' {
                        esc = EscState::Bracket;
                        continue;
                    }
                    // Unknown escape — ignore the Esc and reprocess this byte
                    esc = EscState::Normal;
                    // fall through to handle b normally
                }
                EscState::Bracket => {
                    esc = EscState::Normal;
                    match b {
                        b'A' => {
                            self.handle_key(
                                Key::Up,
                                &mut lb,
                                &mut stdout,
                                &mut hist_idx,
                                &mut saved_line,
                                prompt,
                            );
                            continue;
                        }
                        b'B' => {
                            self.handle_key(
                                Key::Down,
                                &mut lb,
                                &mut stdout,
                                &mut hist_idx,
                                &mut saved_line,
                                prompt,
                            );
                            continue;
                        }
                        b'C' => {
                            self.handle_key(
                                Key::Right,
                                &mut lb,
                                &mut stdout,
                                &mut hist_idx,
                                &mut saved_line,
                                prompt,
                            );
                            continue;
                        }
                        b'D' => {
                            self.handle_key(
                                Key::Left,
                                &mut lb,
                                &mut stdout,
                                &mut hist_idx,
                                &mut saved_line,
                                prompt,
                            );
                            continue;
                        }
                        b'H' => {
                            self.handle_key(
                                Key::Home,
                                &mut lb,
                                &mut stdout,
                                &mut hist_idx,
                                &mut saved_line,
                                prompt,
                            );
                            continue;
                        }
                        b'F' => {
                            self.handle_key(
                                Key::End,
                                &mut lb,
                                &mut stdout,
                                &mut hist_idx,
                                &mut saved_line,
                                prompt,
                            );
                            continue;
                        }
                        b'0'..=b'9' => {
                            esc = EscState::BracketDigit(b - b'0');
                            continue;
                        }
                        _ => continue, // unknown — ignore
                    }
                }
                EscState::BracketDigit(d) => {
                    esc = EscState::Normal;
                    if b == b'~' {
                        match d {
                            1 => {
                                self.handle_key(
                                    Key::Home,
                                    &mut lb,
                                    &mut stdout,
                                    &mut hist_idx,
                                    &mut saved_line,
                                    prompt,
                                );
                                continue;
                            }
                            3 => {
                                self.handle_key(
                                    Key::Delete,
                                    &mut lb,
                                    &mut stdout,
                                    &mut hist_idx,
                                    &mut saved_line,
                                    prompt,
                                );
                                continue;
                            }
                            4 => {
                                self.handle_key(
                                    Key::End,
                                    &mut lb,
                                    &mut stdout,
                                    &mut hist_idx,
                                    &mut saved_line,
                                    prompt,
                                );
                                continue;
                            }
                            _ => continue, // ignore
                        }
                    }
                    // Unknown digit sequence — ignore
                    continue;
                }
            }

            // Normal byte processing
            let key = match b {
                0x01 => Key::CtrlA,                                 // Ctrl-A → Home
                0x03 => Key::CtrlC,                                 // Ctrl-C
                0x04 => Key::CtrlD,                                 // Ctrl-D → EOF
                0x05 => Key::CtrlE,                                 // Ctrl-E → End
                0x08 | 0x7F => Key::Backspace,                      // BS or DEL
                0x0A | 0x0D => Key::Enter,                          // LF or CR
                0x0B => Key::CtrlK,                                 // Ctrl-K → kill to end
                0x0C => Key::CtrlL,                                 // Ctrl-L → redraw
                0x15 => Key::CtrlU,                                 // Ctrl-U → kill to start
                b if b >= 0x20 && b < 0x7F => Key::Char(b as char), // printable ASCII
                b if b >= 0xC0 => {
                    // UTF-8 lead byte
                    utf8_buf.clear();
                    utf8_buf.push(b);
                    if b < 0xE0 {
                        utf8_remaining = 1;
                    } else if b < 0xF0 {
                        utf8_remaining = 2;
                    } else {
                        utf8_remaining = 3;
                    }
                    Key::Ignore
                }
                _ => Key::Ignore,
            };

            match key {
                Key::Enter => {
                    write_bytes(&mut stdout, b"\r\n");
                    let _ = stdout.flush();
                    let line = lb.as_str().to_string();
                    // Add to history if non-empty and different from last
                    if !line.trim().is_empty() {
                        if self.history.last().map_or(true, |prev| prev != &line) {
                            self.history.push(line.clone());
                            if self.history.len() > self.max_history {
                                self.history.remove(0);
                            }
                        }
                    }
                    return Some(line);
                }
                Key::CtrlC => {
                    // Discard line, print ^C, start fresh
                    write_bytes(&mut stdout, b"^C\r\n");
                    write_bytes(&mut stdout, prompt.as_bytes());
                    let _ = stdout.flush();
                    lb.clear();
                    hist_idx = None;
                    saved_line.clear();
                }
                Key::CtrlD => {
                    if lb.as_str().is_empty() {
                        write_bytes(&mut stdout, b"\r\n");
                        let _ = stdout.flush();
                        return None; // EOF
                    }
                    // Non-empty line: treat as delete
                    self.handle_key(
                        Key::Delete,
                        &mut lb,
                        &mut stdout,
                        &mut hist_idx,
                        &mut saved_line,
                        prompt,
                    );
                }
                Key::Ignore => {}
                other => {
                    self.handle_key(
                        other,
                        &mut lb,
                        &mut stdout,
                        &mut hist_idx,
                        &mut saved_line,
                        prompt,
                    );
                }
            }
        }
    }

    fn handle_key(
        &self,
        key: Key,
        lb: &mut LineBuffer,
        out: &mut impl Write,
        hist_idx: &mut Option<usize>,
        saved_line: &mut String,
        prompt: &str,
    ) {
        match key {
            Key::Char(c) => {
                lb.insert(c);
                if lb.is_at_end() {
                    // Simple append — just echo the character
                    let mut tmp = [0u8; 4];
                    write_bytes(out, c.encode_utf8(&mut tmp).as_bytes());
                } else {
                    // Insert in the middle — redraw from cursor
                    let after = lb.after_cursor();
                    let after_width = str_width(after);
                    let mut tmp = [0u8; 4];
                    write_bytes(out, c.encode_utf8(&mut tmp).as_bytes());
                    write_bytes(out, after.as_bytes());
                    clear_to_eol(out);
                    // Move cursor back to the correct position
                    move_cursor_left(out, after_width);
                }
                let _ = out.flush();
            }
            Key::Backspace => {
                if let Some(c) = lb.backspace() {
                    let w = char_width(c);
                    if lb.is_at_end() {
                        // Simple erase at end: move back, clear
                        move_cursor_left(out, w);
                        clear_to_eol(out);
                    } else {
                        // Middle of line: move back, redraw rest
                        move_cursor_left(out, w);
                        let after = lb.after_cursor();
                        write_bytes(out, after.as_bytes());
                        clear_to_eol(out);
                        move_cursor_left(out, str_width(after));
                    }
                    let _ = out.flush();
                }
            }
            Key::Delete => {
                if let Some(_c) = lb.delete() {
                    // Redraw from cursor position
                    let after = lb.after_cursor();
                    write_bytes(out, after.as_bytes());
                    clear_to_eol(out);
                    move_cursor_left(out, str_width(after));
                    let _ = out.flush();
                }
            }
            Key::Left => {
                if let Some(c) = lb.move_left() {
                    move_cursor_left(out, char_width(c));
                    let _ = out.flush();
                }
            }
            Key::Right => {
                if let Some(c) = lb.move_right() {
                    move_cursor_right(out, char_width(c));
                    let _ = out.flush();
                }
            }
            Key::Home | Key::CtrlA => {
                let w = lb.move_home();
                move_cursor_left(out, w);
                let _ = out.flush();
            }
            Key::End | Key::CtrlE => {
                let w = lb.move_end();
                move_cursor_right(out, w);
                let _ = out.flush();
            }
            Key::Up => {
                if self.history.is_empty() {
                    return;
                }
                let new_idx = match *hist_idx {
                    None => {
                        *saved_line = lb.as_str().to_string();
                        self.history.len() - 1
                    }
                    Some(i) => {
                        if i == 0 {
                            return; // already at oldest
                        }
                        i - 1
                    }
                };
                *hist_idx = Some(new_idx);
                self.replace_line(lb, out, &self.history[new_idx].clone(), prompt);
            }
            Key::Down => {
                match *hist_idx {
                    None => return, // not in history mode
                    Some(i) => {
                        if i + 1 < self.history.len() {
                            *hist_idx = Some(i + 1);
                            self.replace_line(lb, out, &self.history[i + 1].clone(), prompt);
                        } else {
                            // Return to saved line
                            *hist_idx = None;
                            let s = saved_line.clone();
                            self.replace_line(lb, out, &s, prompt);
                        }
                    }
                }
            }
            Key::CtrlU => {
                // Kill from start to cursor
                let w = str_width(&lb.buf[..lb.cursor]);
                let rest = lb.after_cursor().to_string();
                lb.buf = rest;
                lb.cursor = 0;
                // Redraw: move to column 0 (after prompt), rewrite line
                move_cursor_left(out, w);
                write_bytes(out, lb.buf.as_bytes());
                clear_to_eol(out);
                move_cursor_left(out, str_width(&lb.buf));
                let _ = out.flush();
            }
            Key::CtrlK => {
                // Kill from cursor to end
                lb.buf.truncate(lb.cursor);
                clear_to_eol(out);
                let _ = out.flush();
            }
            Key::CtrlL => {
                // Redraw whole line
                write_bytes(out, b"\x1b[2J\x1b[H"); // clear screen, home
                write_bytes(out, prompt.as_bytes());
                write_bytes(out, lb.buf.as_bytes());
                // Move cursor to correct position
                let after_width = str_width(lb.after_cursor());
                move_cursor_left(out, after_width);
                let _ = out.flush();
            }
            _ => {}
        }
    }

    /// Replace the entire line buffer and redraw.
    fn replace_line(&self, lb: &mut LineBuffer, out: &mut impl Write, new: &str, _prompt: &str) {
        // Move cursor to start of input
        let cur_width = str_width(&lb.buf[..lb.cursor]);
        move_cursor_left(out, cur_width);
        // Clear old line
        clear_to_eol(out);
        // Write new content
        *lb = LineBuffer::from(new);
        write_bytes(out, lb.buf.as_bytes());
        let _ = out.flush();
    }
}
