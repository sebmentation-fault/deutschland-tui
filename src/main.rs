use clap::Parser;
use csv::ReaderBuilder;
use rand::Rng;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Alignment, Rect},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{
        block::{Position, Title},
        Block, Paragraph, Widget,
    },
    DefaultTerminal, Frame,
};
use std::{error::Error, fs::File, io};

// --- Use the person, tense, verb structs ---
mod person;
mod tense;
mod verb;
use person::Person;
use tense::Tense;
use verb::Verb;

/// The possible arguments.
///
/// See clap docs: https://docs.rs/clap/latest/clap/
#[derive(Parser, Debug)]
#[command(author = "Sebastian K.", version, about = "A simple CLI tool to help test on German conjugations of common verbs", long_about = None)]
pub struct Args {
    /// Number of questions in the lesson
    #[arg(short, long, default_value_t = 10)]
    number: u8,

    /// The person to focus on
    #[arg(short, long)]
    person: Option<String>,

    /// The verb to focus on. Could extend so this is Option too.
    #[arg(short, long)]
    verb: String,

    /// The tense (to focus one specifically)
    #[arg(short, long)]
    tense: Option<String>,
}

#[derive(Debug, Clone)]
struct Conjugation {
    person: Person,
    tense: Tense,
    verb: Verb,
    english: String,
    german: String,
}

/// Loads and parses the conjugations for the verb
fn parse_conjugations(verb: &Verb) -> Result<Vec<Conjugation>, Box<dyn Error>> {
    let file_path = format!("./verbs/{}.csv", verb);
    let file = File::open(file_path)?;
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);

    let mut conjugations: Vec<Conjugation> = Vec::new();
    for result in rdr.records() {
        let record = result?;
        let tense = Tense::from_str(record.get(0).unwrap());
        let person = Person::from_str(record.get(1).unwrap());
        let english = record.get(2).unwrap().to_string();
        let german = record.get(3).unwrap().to_string();
        let con = Conjugation {
            verb: *verb,
            tense,
            person,
            english,
            german,
        };
        conjugations.push(con);
    }

    Ok(conjugations)
}

/// The application state
#[derive(Debug)]
pub struct App {
    cur_question: u8,
    total_questions: u8,
    total_correct: u8,   // Total correct answers
    total_incorrect: u8, // Total incorrect answers

    cur_conjugation: usize,         // Index to the conjugation that we are on
    conjugations: Vec<Conjugation>, // All the conjugations we are allowed to ask
    cur_response: String,           // The current response from the user
    cur_response_incorrect: Option<bool>, // If entered, then if the response was correct
    exit: Option<bool>,
}

// Based mostly off of the example in the ratatui repo:
// https://ratatui.rs/tutorials/counter-app/basic-app/
impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<u8> {
        while self.exit.is_none() || self.exit.is_some_and(|x| x == false) {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(self.total_correct)
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
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
        match key_event.code {
            KeyCode::Enter => {
                // user wants to repeat
                if self.exit.is_some() {
                    self.cur_question = 0;
                    self.total_correct = 0;
                    self.total_incorrect = 0;
                    self.exit = None;
                    return;
                }
                // user is still answering questions
                // if it is incorrect, we basically just wait for another enter key
                if self.cur_response_incorrect.is_none() {
                    self.check_answer();
                } else {
                    self.next_question();
                }
            }
            KeyCode::Esc => self.exit = Some(true),
            KeyCode::Backspace => {
                self.cur_response.pop();
            }
            KeyCode::Char(c) => self.cur_response.push(c),
            _ => {}
        }
    }

    /// Checks the current answer and updates the state accordingly
    /// It awaits another enter key, such that the next view will show
    /// the correct answer or a great job message.
    /// Does nothing if the input is empty
    fn check_answer(&mut self) {
        if self.cur_response.is_empty() {
            return;
        }

        let correct = self.conjugations.get(self.cur_conjugation).unwrap().german
            == self.cur_response.to_lowercase();

        if !correct {
            self.total_incorrect += 1;
            self.cur_response_incorrect = Some(true);
        } else {
            self.total_correct += 1;
            self.cur_response_incorrect = Some(false);
        }
    }

    /// Moves to the next question
    /// If there are no more questions, then it sets the exit flag to true
    /// so that the application will exit.
    fn next_question(&mut self) {
        self.cur_response.clear();
        self.cur_response_incorrect = None;
        self.cur_question += 1;
        if self.cur_question >= self.total_questions {
            self.exit = Some(false);
        }
        self.cur_conjugation = rand::thread_rng().gen_range(0..self.conjugations.len());
    }
}

impl Widget for &App {
    /// Renders the application state
    ///
    /// For new questions, it will show the current question and the total number of questions
    /// and also the current question number, and the currently entered response.
    ///
    /// For questions that have been answered, if the answer was correct, then it will show
    /// good job.
    ///
    /// If the answer was incorrect, then it will show the correct answer.
    fn render(self, area: Rect, buf: &mut Buffer) {
        // we are ready to exit, just have to wait for the user to acknowledge
        // the final score
        if self.exit.is_some() {
            self.render_score(area, buf);
            return;
        }

        match self.cur_response_incorrect {
            Some(true) => {
                self.render_incorrect(area, buf);
            }
            Some(false) => {
                self.render_correct(area, buf);
            }
            None => {
                self.render_unanswered_question(area, buf);
            }
        }
    }
}

impl App {
    fn render_unanswered_question(&self, area: Rect, buf: &mut Buffer) {
        let conj = self.conjugations.get(self.cur_conjugation).unwrap();
        let title = Title::from(
            format!(
                " {} | {} | {} | Q{}/{} ",
                conj.verb,
                conj.tense,
                conj.person,
                self.cur_question + 1,
                self.total_questions
            )
            .bold(),
        );
        let instructions = Title::from(Line::from(vec![
            " Input Answer ".into(),
            "<Chars> ".blue().bold(),
            " Submit ".into(),
            "<Enter> ".blue().bold(),
        ]));
        let block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .border_set(border::THICK);

        let text = Text::from(vec![
            Line::from(""),
            Line::from(""),
            Line::from(vec!["English: ".into(), conj.english.to_string().yellow()]),
            Line::from(vec![
                "Your input: ".into(),
                self.cur_response.to_string().blue(),
            ]),
        ]);

        Paragraph::new(text)
            .centered()
            .block(block)
            .render(area, buf);
    }

    fn render_correct(&self, area: Rect, buf: &mut Buffer) {
        let conj = self.conjugations.get(self.cur_conjugation).unwrap();
        let title = Title::from(
            format!(
                " {} | {} | {} | Q{}/{} ",
                conj.verb,
                conj.tense,
                conj.person,
                self.cur_question + 1,
                self.total_questions
            )
            .bold(),
        );
        let instructions = Title::from(Line::from(vec![
            " Continue ".into(),
            "<Enter> ".blue().bold(),
        ]));
        let block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .border_set(border::THICK);

        let text = Text::from(vec![
            Line::from(""),
            Line::from(""),
            Line::from(vec!["English: ".into(), conj.english.to_string().yellow()]),
            Line::from(vec![
                "Your input: ".into(),
                self.cur_response.to_string().green(),
            ]),
        ]);

        Paragraph::new(text)
            .centered()
            .block(block)
            .render(area, buf);
    }

    fn render_incorrect(&self, area: Rect, buf: &mut Buffer) {
        let conj = self.conjugations.get(self.cur_conjugation).unwrap();
        let title = Title::from(
            format!(
                " {} | {} | {} | Q{}/{} ",
                conj.verb,
                conj.tense,
                conj.person,
                self.cur_question + 1,
                self.total_questions
            )
            .bold(),
        );
        let instructions = Title::from(Line::from(vec![
            " Continue ".into(),
            "<Enter> ".blue().bold(),
        ]));
        let block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .border_set(border::THICK);

        let text = Text::from(vec![
            Line::from(""),
            Line::from(""),
            Line::from(vec!["English: ".into(), conj.english.to_string().yellow()]),
            Line::from(vec![
                "Your input: ".into(),
                self.cur_response.to_string().red(),
            ]),
            Line::from(vec![
                "Correct German: ".into(),
                conj.german.to_string().green(),
            ]),
        ]);

        Paragraph::new(text)
            .centered()
            .block(block)
            .render(area, buf);
    }

    fn render_score(&self, area: Rect, buf: &mut Buffer) {
        let title = Title::from(" Lesson Completed ".bold());
        let instructions = Title::from(Line::from(vec![
            " Exit ".into(),
            "<ESC> ".blue().bold(),
            " Attempt Again ".into(),
            "<Enter> ".blue().bold(),
        ]));
        let block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .border_set(border::THICK);

        let text = Text::from(vec![
            Line::from(""),
            Line::from(""),
            Line::from(format!(
                "You got {} correct out of {}!",
                self.total_correct, self.total_questions
            )),
        ]);

        Paragraph::new(text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}

fn main() -> Result<(), io::Error> {
    // 1. Santize the arguments
    let args = Args::parse();
    let n = args.number;
    if !(1..100).contains(&n) {
        panic!("n is either too small or too large");
    }
    let verb = Verb::from_str(&args.verb);

    // 2. Init ratatui
    let mut terminal = ratatui::init();

    // 3. Load the conjugations
    let conjugations = parse_conjugations(&verb).expect("Conjugations was not parsed");

    // 4. Loop for each question
    let mut app = App {
        cur_question: 0,
        total_questions: n,
        total_correct: 0,
        total_incorrect: 0,
        cur_conjugation: rand::thread_rng().gen_range(0..conjugations.len()),
        conjugations,
        cur_response: String::new(),
        cur_response_incorrect: None,
        exit: None,
    };
    let _ = app.run(&mut terminal).expect("App failed to run");
    ratatui::restore();

    // 5. Exit
    Ok(())
}
