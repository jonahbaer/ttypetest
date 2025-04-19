mod test_instance;
use test_instance::TestInstance;

use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    time::Duration,
};

use rand::seq::SliceRandom;

use futures::StreamExt;

use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind, KeyModifiers};

use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::{Line, Text},
    widgets::{Block, Padding, Paragraph, Widget, Wrap},
};

struct Ttypetest {
    test: TestInstance,
    corpus: Vec<String>,
    exit: bool,
}

impl Ttypetest {
    const FPS: f64 = 60.0;
    const WORD_SRC: &str = "words.txt";

    fn new() -> io::Result<Self> {
        let f = File::open(Self::WORD_SRC)?;
        let r = BufReader::new(f);
        let corpus: Result<Vec<String>, io::Error> = r.lines().collect();

        match corpus {
            Ok(mut corpus) => {
                let mut rng = rand::rng();
                corpus.shuffle(&mut rng);

                Ok(Self {
                    test: TestInstance::new(&corpus, 30),
                    corpus,
                    exit: false,
                })
            }
            Err(e) => Err(e),
        }
    }

    async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let period = Duration::from_secs_f64(1.0 / Self::FPS);
        let mut interval = tokio::time::interval(period);
        let mut events = EventStream::new();

        while !self.exit {
            tokio::select! {
                _ = interval.tick() => { terminal.draw(|frame| self.draw(frame))?; },
                Some(Ok(event)) = events.next() => self.handle_event(&event),
            }
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_event(&mut self, event: &Event) {
        if let Event::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Esc => self.exit = true,
                    KeyCode::Char(' ') => self.test.space(),
                    KeyCode::Char(c) => self.test.input(c),
                    KeyCode::Backspace => _ = self.test.backspace(),
                    KeyCode::Enter => {
                        let mut rng = rand::rng();
                        self.corpus.shuffle(&mut rng);
                        self.test = TestInstance::new(&self.corpus, 30);
                    }
                    _ => (),
                }
            }
        }
    }
}

impl Widget for &Ttypetest {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let timer = self
            .test
            .elapsed()
            .unwrap_or(Duration::from_secs(0))
            .as_secs_f64();
        let wpm = self.test.wpm().unwrap_or(0.0);
        let cpm = self.test.cpm().unwrap_or(0.0);
        let stat_line = Line::from(format!(
            "time: {:.2} ; cpm: {:.2} ; wpm: {:.2}",
            timer, cpm, wpm
        ));
        let header = Text::from_iter(["ttypetest".blue().bold().into(), "".into(), stat_line]);

        let vert_lout = Layout::vertical([
            Constraint::Length(header.height() as u16 + 2),
            Constraint::Percentage(100),
            Constraint::Length(3),
        ]);
        let [header_area, test_area, info_area] = vert_lout.areas(area);

        Paragraph::new(header)
            .centered()
            .block(Block::new().padding(Padding::proportional(1)))
            .render(header_area, buf);

        let test_text = Text::from(vec![self.test.rtui_line()]);

        Paragraph::new(test_text)
            .wrap(Wrap { trim: true })
            .block(
                Block::new()
                    .padding(Padding::proportional(2))
                    .light_yellow(),
            )
            .render(test_area, buf);

        Paragraph::new(Text::from("<esc> quit - <enter> restart"))
            .centered()
            .block(Block::new().padding(Padding::proportional(1)))
            .render(info_area, buf);
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = Ttypetest::new()?.run(&mut terminal).await;
    ratatui::restore();
    app_result
}
