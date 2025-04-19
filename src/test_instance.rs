use std::time::{Duration, Instant};

use ratatui::{
    style::Stylize,
    text::{Line, Span},
};

#[derive(Clone)]
enum LetterScore {
    NoInput,
    Correct,
    Incorrect,
}

struct Word {
    letters: String,
    input: Option<String>,
}

impl Word {
    fn new(letters: String) -> Self {
        Self {
            letters,
            input: None,
        }
    }

    fn input(&mut self, c: char) -> () {
        if let Some(ref mut s) = self.input {
            s.push(c);
        } else {
            self.input = Some(String::from(c));
        }
    }

    // returns true if a character was actually deleted
    // false if the word-input is empty
    fn backspace(&mut self) -> bool {
        if let Some(ref mut s) = self.input {
            s.pop();
            if s.len() == 0 {
                self.input = None;
            }
            true
        } else {
            false
        }
    }

    fn has_input(&self) -> bool {
        match self.input {
            Some(_) => true,
            None => false,
        }
    }

    fn is_correct(&self) -> bool {
        self.score().iter().all(|s| match s {
            LetterScore::Correct => true,
            _ => false,
        })
    }

    fn score(&self) -> Vec<LetterScore> {
        if let Some(ref input) = self.input {
            let mut v: Vec<LetterScore> = self
                .letters
                .chars()
                .zip(input.chars())
                .map(|(a, b)| {
                    if a == b {
                        LetterScore::Correct
                    } else {
                        LetterScore::Incorrect
                    }
                })
                .collect();

            if self.letters.len() > input.len() {
                while v.len() < self.letters.len() {
                    v.push(LetterScore::NoInput)
                }
            } else if input.len() > self.letters.len() {
                while v.len() < input.len() {
                    v.push(LetterScore::Incorrect)
                }
            }

            v
        } else {
            vec![LetterScore::NoInput; self.letters.len()]
        }
    }

    fn spanvec(&self) -> Vec<Span> {
        if let Some(ref input) = self.input {
            self.score()
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    if i < self.letters.len() {
                        let l = String::from(self.letters.as_bytes()[i] as char);
                        match s {
                            LetterScore::NoInput => l.gray().dim(),
                            LetterScore::Correct => l.green(),
                            LetterScore::Incorrect => l.red(),
                        }
                    } else {
                        match s {
                            LetterScore::Incorrect => {
                                String::from(input.as_bytes()[i] as char).red()
                            }
                            _ => panic!("Should be unreachable"),
                        }
                    }
                })
                .collect()
        } else {
            vec![self.letters.clone().gray().dim()]
        }
    }
}

#[derive(Clone, Copy)]
enum TestState {
    Paused,
    Running(Instant),
    End(Duration),
}

pub struct TestInstance {
    state: TestState,
    words: Vec<Word>,
    current_word: usize,
    input_ccount: u32,
}

impl TestInstance {
    /* assumes corpus is shuffled */
    pub fn new(corpus: &Vec<String>, len: usize) -> Self {
        Self {
            state: TestState::Paused,
            words: corpus
                .iter()
                .take(len)
                .map(|s| Word::new(s.clone()))
                .collect(),
            current_word: 0,
            input_ccount: 0,
        }
    }

    pub fn input(&mut self, c: char) -> () {
        self.state = if let TestState::Paused = self.state {
            TestState::Running(Instant::now())
        } else {
            self.state
        };

        if let TestState::Paused | TestState::Running(_) = self.state {
                self.words[self.current_word].input(c);
                self.input_ccount += 1;
                self.state;
        }
    }

    pub fn elapsed(&self) -> Option<Duration> {
        match self.state {
            TestState::Paused => None,
            TestState::Running(start) => Some(start.elapsed()),
            TestState::End(d) => Some(d),
        }
    }

    pub fn wpm(&self) -> Option<f64> {
        self.elapsed()
            .map(|d| self.current_word as f64 / d.as_secs_f64() * 60.)
    }

    pub fn cpm(&self) -> Option<f64> {
        self.elapsed()
            .map(|d| self.input_ccount as f64 / d.as_secs_f64() * 60.)
    }

    pub fn space(&mut self) -> () {
        if self.current_word < self.words.len() - 1 && self.words[self.current_word].has_input() {
            self.current_word += 1;
        } else if self.current_word == self.words.len() - 1 {
            self.state = match self.state {
                TestState::Paused => todo!("should be unreachable?"),
                TestState::Running(start) => TestState::End(start.elapsed()),
                TestState::End(d) => TestState::End(d),
            };
        }
    }

    pub fn backspace(&mut self) -> () {
        match self.state {
            TestState::Running(_) => {
                if !self.words[self.current_word].backspace() && self.current_word > 0 {
                    self.current_word -= 1;
                }

                if self.input_ccount > 0 {
                    self.input_ccount -= 1;
                }
            }
            _ => (),
        }
    }

    pub fn rtui_line(&self) -> Line {
        Line::default().spans(
            self.words
                .iter()
                .enumerate()
                .map(|(i, w)| {
                    let mut spans = w.spanvec();
                    if i < self.current_word && !w.is_correct() {
                        spans = spans.into_iter().map(|s| s.underlined()).collect();
                    }
                    spans.push(Span::raw(" "));
                    spans
                })
                .flatten(),
        )
    }
}
