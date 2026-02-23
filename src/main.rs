use std::{u8, io};
use rand::prelude::*;
use ratatui::{DefaultTerminal, Frame };
use ratatui::buffer::Buffer;
use ratatui::layout::{Layout, Rect, Constraint};
use ratatui::style::Color;
use ratatui::layout::Alignment;
use ratatui::style::{Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap, Widget};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Override the word to guess (for testing)
    #[arg(short, long)]
    word: String,

    /// Override the maximum number of guesses (for testing or as a difficulty setting)
    #[arg(short, long, default_value_t = 6)]
    max_guesses: u8,
}

struct WordGame {
    word: String,
    guess_buffer: String,
    message: String,
    guessed_words: Vec<String>,
    possible_words: Vec<String>,
    max_guesses: u8,
    exit: bool,
}

impl WordGame {
    fn new(word: String, possible_words: Vec<String>, max_guesses: u8) -> Self {
        WordGame {
            word,
            guess_buffer: String::new(),
            message: String::new(),
            guessed_words: Vec::new(),
            possible_words,
            max_guesses,
            exit: false,
        }
    }

    fn guess(&mut self, guess: String) -> Result<bool, String> {
        if self.possible_words.contains(&guess) {
            if self.guessed_words.contains(&guess) {
                return Err("You've already guessed that word".into());
            }
            let res = guess == self.word;
            self.guessed_words.push(guess);
            return Ok(res);
        }
        Err("Not a valid word".into())
    }

    fn is_game_over(&self) -> bool {
        self.exit || self.guessed_words.len() as u8 >= self.max_guesses || self.guessed_words.contains(&self.word.to_string())
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        if self.is_game_over() {
            frame.render_widget(GameOverWidget { word_game: self }, frame.area());
        } else {
            frame.render_widget(GameWidget { word_game: self }, frame.area());
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.modifiers {
            event::KeyModifiers::CONTROL => {
                // Handle control key combinations if needed
                if key_event.code == KeyCode::Char('c') {
                    self.exit = true; // Exit on Ctrl+C
                }
            }
            _ => {}
        }
        match key_event.code {
            KeyCode::Char(c) => {
                // Handle character input for guessing
                // For simplicity, we will just print the character here.
                // In a real application, you would want to build up the guess string and submit it when the user presses Enter.
                //println!("You pressed: {}", c);
                self.guess_buffer.push(c);
            }
            KeyCode::Backspace => {
                // Handle backspace for editing the guess
                self.guess_buffer.pop();
            }
            KeyCode::Enter => {
                // Handle guess submission
                // You would typically check the current guess against the word here.
                //println!("Guess submitted: {}", self.guess_buffer);
                let guess = self.guess_buffer.clone();
                self.guess_buffer.clear();
                match self.guess(guess) {
                    Ok(true) => self.message = format!("Congratulations! You've guessed {} in {} guesses.", self.word, self.guessed_words.len()),
                    Ok(false) => self.message = format!("Incorrect. You have {} guesses left.", self.max_guesses - self.guessed_words.len() as u8),
                    Err(e) => self.message = format!("{}. You have {} guesses left.", e, self.max_guesses - self.guessed_words.len() as u8),
                }
            }
            KeyCode::Esc => {
                // Handle quitting the game
                self.exit = true;
            }
            _ => {}
        }
    }
}

struct GameOverWidget<'a> {
    word_game: &'a WordGame,
}

impl<'a> Widget for GameOverWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().title("Game Over").borders(Borders::ALL);
        let message = if self.word_game.guessed_words.contains(&self.word_game.word) {
            vec![
                Line::raw(self.word_game.message.clone()),
                Line::raw("Press ESC to quit.")
            ]
        } else {
            vec![
                Line::raw(format!("Game Over! The word was: {}." , self.word_game.word)),
                Line::raw("Press ESC to quit.")
            ]
        };
        let paragraph = Paragraph::new(message)
            .alignment(Alignment::Center).wrap(Wrap { trim: true })
            .block(block);
        paragraph.render(area, buf);
    }
}


struct GameWidget<'a> {
    word_game: &'a WordGame,
}


impl<'a> Widget for GameWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let main_layout = Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]);
        let [top, bottom] = main_layout.areas(area);
        // Bottom layout
        let bottom_layout = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
        let [bottom_left, bottom_right] = bottom_layout.areas(bottom);

        // Message at the top
        let message_block = Block::default().title("Message").borders(Borders::ALL);


        let message_paragraph = Paragraph::new(self.word_game.message.clone())
            .alignment(Alignment::Left).wrap(Wrap { trim: true })
            .block(message_block);
        message_paragraph.render(top, buf);

        let guesses_widget = GuessesWidget { word_game: &self.word_game };
        guesses_widget.render(bottom_left, buf);

        let alphabet_widget = AlphabetWidget { word_game: &self.word_game };
        alphabet_widget.render(bottom_right, buf);
    }
}

struct GuessesWidget<'a> {
    word_game: &'a WordGame,
}

impl<'a> Widget for GuessesWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().title("Guesses").borders(Borders::ALL);
        let mut text = self.word_game.guessed_words.iter().map(|guess| {
            let mut letter_count = std::collections::HashMap::new();
            self.word_game.word.chars().for_each(|c| {
                *letter_count.entry(c).or_insert(0) += 1;
            });
            let line = guess.chars().enumerate().map(|(i, c)| {
                if c == self.word_game.word.chars().nth(i).unwrap() {
                    // decrement the letter count for this character
                    *letter_count.get_mut(&c).unwrap() -= 1;
                    Span::styled(c.to_string(), Style::default().fg(Color::Green))
                } else if self.word_game.word.contains(c) {
                    letter_count.get_mut(&c).map_or(Span::styled(c.to_string(), Style::default().fg(Color::Red)), |count| {
                        if *count > 0 {
                            *count -= 1;
                            Span::styled(c.to_string(), Style::default().fg(Color::LightYellow))
                        } else {
                            Span::styled(c.to_string(), Style::default().fg(Color::DarkGray))
                        }
                    })
                } else {
                    Span::styled(c.to_string(), Style::default().fg(Color::DarkGray))
                }
            }).collect::<Vec<Span>>();
            Line::from(line)
        }).collect::<Vec<Line>>();
        text.push(Line::from(Span::raw(self.word_game.guess_buffer.clone())));
        let paragraph = Paragraph::new(text)
            .alignment(Alignment::Left).wrap(Wrap { trim: true })
            .block(block);
        paragraph.render(area, buf);
    }
}

struct AlphabetWidget<'a> {
    word_game: &'a WordGame,
}

impl<'a> Widget for AlphabetWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().title("Alphabet").borders(Borders::ALL);
        // Generate an alphabet vector with default styling
        let mut alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().map(|c| {
           Span::raw(c.to_string())
        }).collect::<Vec<Span>>();
        // For each guessed word we will update the styling of the letters in the alphabet
        self.word_game.guessed_words.iter().for_each(|guess| {
            // For each letter, update the styling
            guess.chars().enumerate().for_each(|(i, c)| {
                // Convert to uppercase for indexing into the alphabet vector
                let c_upper = c.to_ascii_uppercase();
                // Calculate the index for the alphabet vector (A=0, B=1, ..., Z=25)
                let idx = (c_upper as u8 - b'A') as usize;
                if c_upper == self.word_game.word.chars().nth(i).unwrap().to_ascii_uppercase() {
                    // Correct letter and position
                    alphabet[idx] = Span::styled(c_upper.to_string(), Style::default().fg(Color::Green));
                } else if self.word_game.word.contains(c) {
                    // Correct letter but wrong position
                    // Do not update the color to yellow if it is already green (correct position)
                    if alphabet[idx].style.fg != Some(Color::Green) {
                        alphabet[idx] = Span::styled(c_upper.to_string(), Style::default().fg(Color::LightYellow));
                    }
                } else {
                    alphabet[idx] = Span::styled(c_upper.to_string(), Style::default().fg(Color::DarkGray));
                }
            });
        });
        let paragraph = Paragraph::new(Line::from(alphabet))
            .alignment(Alignment::Left).wrap(Wrap { trim: true })
            .block(block);
        paragraph.render(area, buf);
    }
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    // List of allowed guess words
    let valid_words: Vec<String> = include_str!("valid-words.txt").lines().map(|line| line.to_string()).collect();
    // List of words that can be chosen as the word to guess
    let guess_words: Vec<String> = include_str!("words.txt").lines().map(|line| line.to_string()).collect();
    // Get an RNG:
    let mut rng = rand::rng();
    let word = if args.word.len() > 0 {
        args.word.clone()
    } else {
        // Choose a random word from the list of guess words:
        let word_index: u32 = rng.next_u32() % guess_words.len() as u32;
        guess_words[word_index as usize].clone()
    };
    // Initialize the game
    let mut game = WordGame::new(word, valid_words, args.max_guesses);
    // Run the game loop
    ratatui::run(|terminal| game.run(terminal))
}
