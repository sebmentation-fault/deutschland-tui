use clap::Parser;
use csv::ReaderBuilder;
use rand::Rng;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Alignment, Constraint, Rect},
    style::{Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{
        block::{Position, Title},
        Block, Cell, Paragraph, Row, Table, TableState, Widget,
    },
    DefaultTerminal, Frame,
};
use std::{
    error::Error,
    fs::{self, File},
    io,
};

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
    verb: Option<String>,

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
pub struct App {
    cur_question: u8,
    total_questions: u8,
    total_correct: u8,   // Total correct answers
    total_incorrect: u8, // Total incorrect answers

    // if None, then show the select screen. Can choose to be specific or to be open to all
    table_state: TableState,
    verbs: Vec<String>, // the list of all the verbs (german + english)
    verb: Option<Verb>, // the chosen verb

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

    fn draw(&mut self, frame: &mut Frame) {
        // if we are rendering table we pass in different arguments than to render_widget
        if self.verb.is_none() {
            self.render_verbs_table(frame);
            return;
        }

        frame.render_widget(&*self, frame.area());
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
        if self.verb.is_none() {
            self.handle_key_event_select_verb(key_event);
            return;
        }

        if self.exit.is_some() {
            self.handle_key_event_game_over(key_event);
            return;
        }

        self.handle_key_event_learning(key_event);
    }

    fn handle_key_event_select_verb(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => self.exit = Some(true),
            KeyCode::Enter => {
                // set the verb
                if let Some(i) = self.table_state.selected() {
                    self.verb = Some(Verb::from_str(
                        self.verbs
                            .get(i)
                            .expect("Selected verb could not be getted"),
                    ));
                } else {
                    panic!("No verb selected, but is being selected")
                }

                self.conjugations = parse_conjugations(&self.verb.unwrap())
                    .expect("Could not parse the conjugations");
                self.cur_conjugation = rand::thread_rng().gen_range(0..self.conjugations.len());
            }
            KeyCode::Up => self.previous_table_item(),
            KeyCode::Char('k') => self.previous_table_item(),
            KeyCode::Down => self.next_table_item(),
            KeyCode::Char('j') => self.next_table_item(),
            _ => {}
        }
    }

    fn handle_key_event_learning(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Enter => {
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

    fn handle_key_event_game_over(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Enter => {
                self.cur_question = 0;
                self.total_correct = 0;
                self.total_incorrect = 0;
                self.exit = None;
            }
            KeyCode::Esc => self.exit = Some(true),
            // select a new verb and go again :)
            _ => {
                self.cur_question = 0;
                self.total_correct = 0;
                self.total_incorrect = 0;
                self.verb = None;
                self.exit = None;
            }
        }
    }

    pub fn next_table_item(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.verbs.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn previous_table_item(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.verbs.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
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
            Line::from(vec!["English: ".into(), conj.english.to_string().blue()]),
            Line::from(vec![
                "Your input: ".into(),
                self.cur_response.to_string().yellow(),
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
            Line::from(vec!["English: ".into(), conj.english.to_string().blue()]),
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
            Line::from(vec!["English: ".into(), conj.english.to_string().blue()]),
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

    fn render_verbs_table(&mut self, frame: &mut Frame) {
        let title = Title::from(" Select a Verb ".bold());
        let instructions = Title::from(Line::from(vec![
            " Prev ".into(),
            "<Up> ".blue().bold(),
            " Next ".into(),
            "<Down> ".blue().bold(),
        ]));
        let block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .border_set(border::THICK);

        let rows: Vec<Row> = self
            .verbs
            .iter()
            .map(|s| Row::new(vec![Cell::from(s.as_str())]))
            .collect();
        let widths = [Constraint::Length(20)];

        let table = Table::new(rows, widths)
            .header(Row::new(vec![Cell::from("Verbs")]))
            .highlight_style(Style::new().reversed())
            .highlight_symbol(">>")
            .block(block);

        frame.render_stateful_widget(table, frame.area(), &mut self.table_state)
    }

    fn render_score(&self, area: Rect, buf: &mut Buffer) {
        let title = Title::from(" Lesson Completed ".bold());
        let instructions = Title::from(Line::from(vec![
            " Exit ".into(),
            "<ESC> ".blue().bold(),
            " Attempt Again ".into(),
            "<Enter> ".blue().bold(),
            " Select New Verb ".into(),
            "<Anything> ".blue().bold(),
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
    let verb = args.verb.map(|v| Verb::from_str(&v));

    // 2. Get the possible verbs
    // get all the file names in the ./verbs directory
    let verb_files = fs::read_dir("./verbs")
        .expect("Could not find/read the verbs directory")
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
        .expect("Could not collect the files");
    // strip the file extension and ./verbs prefix if exists
    let verbs = verb_files
        .iter()
        .map(|p| {
            p.file_name()
                .expect("Could not get the file name")
                .to_str()
                .expect("Could not convert the file name to a string")
                .replace(".csv", "")
        })
        .collect::<Vec<String>>();

    // 3. Init ratatui
    let mut terminal = ratatui::init();

    // 4. Loop for each question
    let mut app = App {
        cur_question: 0,
        total_questions: n,
        total_correct: 0,
        total_incorrect: 0,
        table_state: TableState::default().with_selected(0),
        verbs,
        verb,
        cur_conjugation: usize::MAX, // so that things definitely panic if not updated
        conjugations: vec![],
        cur_response: String::new(),
        cur_response_incorrect: None,
        exit: None,
    };
    let _ = app.run(&mut terminal).expect("App failed to run");
    ratatui::restore();

    // 5. Exit
    Ok(())
}
