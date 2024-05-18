use std::error;

use chrono::NaiveDate;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::Text,
    widgets::{Block, Clear, Padding, Paragraph, Row, Table, TableState, Wrap},
    Frame,
};

use crate::post::Post;

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Clone, PartialEq)]
pub enum FocusedBlock {
    Toc,
    Post,
}

pub struct App<'a> {
    pub running: bool,
    pub focused_block: FocusedBlock,
    pub posts: Vec<Post<'a>>,
    pub scroll: u16,
    pub toc: Vec<(NaiveDate, Text<'a>)>,
    pub toc_state: TableState,
    pub area_height: u16,
    pub previous_key: Vec<u8>,
}

impl<'a> App<'a> {
    pub fn new(posts: Vec<Post<'a>>, toc: Vec<(NaiveDate, Text<'a>)>) -> Self {
        let mut toc_state = TableState::default();
        if toc.is_empty() {
            toc_state.select(None);
        } else {
            toc_state.select(Some(0));
        }

        let focused_block = FocusedBlock::Toc;
        let scroll: u16 = 0;

        let area_height = 0;

        Self {
            running: true,
            focused_block,
            posts,
            scroll,
            toc,
            toc_state,
            area_height,
            previous_key: vec![0],
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        match self.focused_block {
            FocusedBlock::Toc => {
                let layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                            Constraint::Percentage(20),
                            Constraint::Min(self.toc.len() as u16 + 2),
                            Constraint::Percentage(20),
                        ]
                        .as_ref(),
                    )
                    .split(frame.size());

                let block = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        [
                            Constraint::Percentage(20),
                            Constraint::Fill(1),
                            Constraint::Percentage(20),
                        ]
                        .as_ref(),
                    )
                    .split(layout[1])[1];

                let widths = [Constraint::Length(20), Constraint::Fill(1)];
                let rows: Vec<Row> = self
                    .toc
                    .iter()
                    .map(|key| {
                        let date = key.0.format("%B %d %Y").to_string();
                        let title = key.1.to_string();
                        Row::new(vec![date, title]).style(Style::default().fg(Color::White))
                    })
                    .collect();

                let table = Table::new(rows, widths)
                    .block(
                        Block::default()
                            .title("Posts")
                            .title_style(Style::default().bold().fg(Color::Green))
                            .title_alignment(Alignment::Center)
                            .padding(Padding::top(2))
                            .style(Style::default()),
                    )
                    .highlight_symbol(Text::from(">>  ").style(Style::new().yellow().bold()));

                frame.render_widget(Clear, block);
                frame.render_stateful_widget(table, block, &mut self.toc_state.clone());
            }

            FocusedBlock::Post => {
                if let Some(i) = self.toc_state.selected() {
                    let post_title = &self.toc[i].1;

                    if let Some(post) = self.posts.iter().find(|p| &p.title == post_title) {
                        let paragraph = Paragraph::new(post.content.clone())
                            .wrap(Wrap { trim: false })
                            .block(Block::new().title_alignment(Alignment::Center))
                            .style(Style::default())
                            .scroll((self.scroll, 0));

                        frame.render_widget(paragraph, frame.size());
                    }
                };
            }
        }
    }
}
