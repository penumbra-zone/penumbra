use anyhow::Result;

use penumbra_crypto::keys::{
    SeedPhrase, BIP39_MAX_WORD_LENGTH, BIP39_WORDS, SEED_PHRASE_NUM_WORDS as NUM_WORDS,
    SEED_PHRASE_WORDS_PER_LINE as WORDS_PER_LINE,
};

const WORDS_PER_COLUMN: usize = NUM_WORDS / WORDS_PER_LINE;

/// Interactively prompt the user for a seed phrase and return it when finished.
pub fn interactive() -> Result<String> {
    let mut state: State = State::start()?;

    let phrase = loop {
        if let Some(phrase) = state.full_phrase() {
            break phrase;
        }

        state.read_key()?;
    };

    Ok(phrase)
}

/// The state of the interactive seed phrase import process.
///
/// This steps forward one step for each character read. The language accepted by this state machine
/// is a superset of both space-separated seed phrases and numbered seed phrases, and accepts any
/// column format and numbering order (or lack thereof).
struct State {
    read_state: String,
    jump_state: Option<usize>,
    number: usize,
    partial_phrase: [Option<&'static str>; NUM_WORDS],
    term: console::Term,
}

impl State {
    /// Initialize the state machine and print the starting grid.
    fn start() -> Result<Self> {
        let this = Self {
            read_state: String::new(),
            jump_state: None,
            number: 0,
            partial_phrase: [None; NUM_WORDS],
            term: console::Term::stdout(),
        };

        // Print the initial layout and move the cursor into position
        this.term.write_str(&SeedPhrase::format_redacted(' '))?;
        this.term.move_cursor_left(usize::MAX)?;
        this.term.move_cursor_up(5)?;
        this.term.move_cursor_right(4)?;

        Ok(this)
    }

    /// Attempt to extract the full phrase from the state.
    fn full_phrase(&self) -> Option<String> {
        let mut phrase = String::new();
        for (i, word) in self.partial_phrase.iter().enumerate() {
            if let Some(word) = word {
                phrase.push_str(word);
                if i < NUM_WORDS - 1 {
                    phrase.push(' ');
                }
            } else {
                return None;
            }
        }
        Some(phrase)
    }

    /// Read a single key and process it.
    fn read_key(&mut self) -> Result<()> {
        let key = self.term.read_key()?;

        // Reset the jump state to `None`, unless the user presses a numeric key next
        let jump_state = self.jump_state.take();

        match key {
            console::Key::Char(' ') => {
                // We don't advance the cursor when the word hasn't been finalized here yet,
                // because this allows you to copy/paste a numbered seed phrase, and the
                // multiple spaces in a row won't interfere. If you want to manually skip, you
                // can use enter/tab/arrow keys, like a spreadsheet.
                self.try_commit();
                if self.partial_phrase[self.number].is_some() {
                    self.move_to(self.number.saturating_add(1))?;
                }
            }

            // Arrow keys move around the way you would expect:
            console::Key::ArrowLeft => {
                if let Some(n) = self.number.checked_sub(WORDS_PER_COLUMN) {
                    self.move_to(n)?;
                }
            }
            console::Key::ArrowRight | console::Key::Tab => {
                self.move_to(self.number.saturating_add(WORDS_PER_COLUMN))?
            }
            console::Key::ArrowUp => self.move_to(self.number.saturating_sub(1))?,
            console::Key::ArrowDown | console::Key::Enter => {
                self.move_to(self.number.saturating_add(1))?
            }

            // Home and end go to the first and last entries:
            console::Key::Home => self.move_to(0)?,
            console::Key::End => self.move_to(NUM_WORDS - 1)?,

            // Backspace deletes one character of the current word, or one digit of the number being
            // jumped to, and immediately jumps if necessary:
            console::Key::Backspace => {
                // If we're in the middle of a numeric jump, interpret the backspace key as going
                // back one digit in the number.
                if let Some(n) = jump_state {
                    let n = n / 10;
                    if n != 0 {
                        self.move_to(n)?;
                        self.jump_state = Some(n);
                    }
                    return Ok(());
                }

                // Delete the last character of the current word being typed
                if self.read_state.pop().is_none() {
                    // If the word is empty, go back one word after setting this word to `None` in
                    // the underlying phrase (this means you can clear words by revisiting them and
                    // backspacing through them)
                    self.partial_phrase[self.number] = None;
                    self.term.write_str("        ")?;
                    self.term.move_cursor_left(8)?;
                    self.move_to(self.number.saturating_sub(1))?;
                } else {
                    // Erase the deleted character from the screen
                    self.term.move_cursor_left(1)?;
                    self.term.write_str(" ")?;
                    self.term.move_cursor_left(1)?;

                    // If the read state is now empty, render the right background
                    if self.read_state.is_empty() {
                        if self.partial_phrase[self.number].is_some() {
                            self.term.write_str("████████")?;
                            self.term.move_cursor_left(8)?;
                        }
                    }
                }
            }
            console::Key::Char(c) => {
                match c {
                    // Digit characters initiate an immediate jump to another cell, and multiple
                    // digits in a row jump to the combined value of the digits, unless it exceeds
                    // the size of the table, in which case only the most recent is considered
                    '0'..='9' => {
                        let digit = c.to_digit(10).unwrap() as usize;
                        let n = if let Some(n) = jump_state {
                            let n = n * 10 + digit;
                            if n <= NUM_WORDS {
                                n
                            } else {
                                digit
                            }
                        } else {
                            digit
                        };
                        self.move_to(n.saturating_sub(1))?;
                        self.jump_state = Some(n);
                    }
                    // All other characters are tentatively pushed onto the word, and then the word
                    // is checked for prefix inclusion in the word set, before permitting the
                    // character to be added
                    _ => {
                        let mut tentative = self.read_state.clone();
                        tentative.push(c);

                        // Determine if the tentative word is a valid prefix (we only let the user type
                        // valid prefixes of BIP39 words)
                        let valid_prefix = match BIP39_WORDS.binary_search(&tentative.as_str()) {
                            Ok(_) => true,
                            Err(i) => {
                                if let Some(full_word) = BIP39_WORDS.get(i) {
                                    full_word.starts_with(&tentative)
                                } else {
                                    false
                                }
                            }
                        };

                        if valid_prefix {
                            if self.read_state.is_empty() {
                                self.term.write_str("        ")?;
                                self.term.move_cursor_left(8)?;
                            }
                            self.term.write_str(&c.to_string())?;
                            self.read_state = tentative;
                        }
                    }
                }
            }
            _ => { /* ignore anything else */ }
        }

        Ok(())
    }

    // Move the cursor to the given index in the grid, and commit the word if possible.
    fn move_to(&mut self, n: usize) -> Result<()> {
        if n == self.number {
            return Ok(()); // bail if trying to go to where we already are
        }

        self.finish_word()?;

        let from_line = self.number % WORDS_PER_COLUMN;
        let from_column = self.number / WORDS_PER_COLUMN;
        let to_line = n % WORDS_PER_COLUMN;
        let to_column = n / WORDS_PER_COLUMN;

        if n >= NUM_WORDS {
            return Ok(()); // bail if trying to move off the grid
        }

        self.number = n;

        if to_line > from_line {
            self.term.move_cursor_down(to_line - from_line)?;
        } else {
            self.term.move_cursor_up(from_line - to_line)?;
        }

        if to_column > from_column {
            self.term
                .move_cursor_right((to_column - from_column) * (BIP39_MAX_WORD_LENGTH + 5))?;
        } else {
            self.term
                .move_cursor_left((from_column - to_column) * (BIP39_MAX_WORD_LENGTH + 5))?;
        }

        Ok(())
    }

    /// Commit the current word, or don't, displaying as appropriate.
    fn finish_word(&mut self) -> Result<()> {
        self.try_commit();

        self.term.move_cursor_left(self.read_state.len())?;
        if self.partial_phrase[self.number].is_some() {
            self.term.write_str("████████")?;
        } else {
            self.term.write_str("        ")?;
        }
        self.term.move_cursor_left(8)?;

        // Reset the read state because we moved away from the word
        self.read_state = String::new();

        Ok(())
    }

    /// Try to commit the current word.
    fn try_commit(&mut self) {
        if let Ok(index) = BIP39_WORDS.binary_search(&self.read_state.as_str()) {
            self.partial_phrase[self.number] = Some(BIP39_WORDS[index]);
        }
    }
}
