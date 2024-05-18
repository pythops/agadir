use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::NaiveDate;
use ed25519_dalek::SigningKey;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::text::Text;
use ratatui::Terminal;
use russh::server::*;
use russh::{Channel, ChannelId};
use russh_keys::key::PublicKey;
use tokio::sync::Mutex;
use tracing::info;

use crate::app::{App, FocusedBlock};
use crate::post::Post;
use crate::ui;

#[derive(Clone)]
struct TerminalHandle {
    handle: Handle,
    sink: Vec<u8>,
    channel_id: ChannelId,
}

impl std::io::Write for TerminalHandle {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.sink.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let handle = self.handle.clone();
        let channel_id = self.channel_id;
        let data = self.sink.clone().into();
        futures::executor::block_on(async move {
            let result = handle.data(channel_id, data).await;
            if result.is_err() {
                eprintln!("Failed to send data: {:?}", result);
            }
        });

        self.sink.clear();
        Ok(())
    }
}

type SshTerminal = Terminal<CrosstermBackend<TerminalHandle>>;

#[derive(Clone)]
pub struct AppServer {
    clients: Arc<Mutex<HashMap<usize, (SshTerminal, App<'static>)>>>,
    id: usize,
    pub posts: Vec<Post<'static>>,
    pub toc: Vec<(NaiveDate, Text<'static>)>,
    key: SigningKey,
}

impl AppServer {
    pub fn new(
        posts: Vec<Post<'static>>,
        toc: Vec<(NaiveDate, Text<'static>)>,
        key: SigningKey,
    ) -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            id: 0,
            posts,
            toc,
            key,
        }
    }

    pub async fn run(&mut self, port: u16) -> Result<(), anyhow::Error> {
        let clients = self.clients.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

                for (_, (terminal, app)) in clients.lock().await.iter_mut() {
                    terminal.draw(|frame| ui::render(app, frame)).unwrap();
                }
            }
        });

        let config = Config {
            inactivity_timeout: Some(std::time::Duration::from_secs(3600)),
            auth_rejection_time: std::time::Duration::from_secs(3),
            auth_rejection_time_initial: Some(std::time::Duration::from_secs(0)),
            keys: vec![russh_keys::key::KeyPair::Ed25519(self.key.clone())],
            ..Default::default()
        };

        info!("{}", format!("Running on port {}", port));
        self.run_on_address(Arc::new(config), ("0.0.0.0", port))
            .await?;
        Ok(())
    }
}

impl Server for AppServer {
    type Handler = Self;
    fn new_client(&mut self, _: Option<std::net::SocketAddr>) -> Self {
        let s = self.clone();
        self.id += 1;
        s
    }
}

#[async_trait]
impl Handler for AppServer {
    type Error = anyhow::Error;

    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        session: &mut Session,
    ) -> Result<bool, Self::Error> {
        let mut clients = self.clients.lock().await;
        let terminal_handle = TerminalHandle {
            handle: session.handle(),
            sink: Vec::new(),
            channel_id: channel.id(),
        };

        let backend = CrosstermBackend::new(terminal_handle.clone());
        let mut terminal = Terminal::new(backend)?;
        let app = App::new(self.posts.clone(), self.toc.clone());

        terminal.hide_cursor()?;
        terminal.clear()?;

        clients.insert(self.id, (terminal, app));

        info!(id = self.id, "New client is connected.");

        Ok(true)
    }

    async fn auth_publickey(&mut self, _: &str, _: &PublicKey) -> Result<Auth, Self::Error> {
        Ok(Auth::Accept)
    }

    async fn auth_none(&mut self, _: &str) -> Result<Auth, Self::Error> {
        Ok(Auth::Accept)
    }

    async fn auth_password(&mut self, _: &str, _: &str) -> Result<Auth, Self::Error> {
        Ok(Auth::Accept)
    }

    //TODO: refactor: convert to crossterm keys
    async fn data(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        match data {
            b"q" | [3] => {
                let mut clients = self.clients.lock().await;
                let (terminal, _) = clients.get_mut(&self.id).unwrap();
                terminal.clear()?;
                terminal.show_cursor()?;
                clients.remove(&self.id);
                session.close(channel);
                info!(id = self.id, "Client disconnected.");
                return Ok(());
            }

            b"j" => {
                let mut clients = self.clients.lock().await;
                let (_, app) = clients.get_mut(&self.id).unwrap();

                match &app.focused_block {
                    FocusedBlock::Toc => {
                        let i = match app.toc_state.selected() {
                            Some(i) => {
                                if i < app.posts.len() - 1 {
                                    i + 1
                                } else {
                                    i
                                }
                            }
                            None => 0,
                        };

                        app.toc_state.select(Some(i));
                    }
                    FocusedBlock::Post => {
                        app.scroll = app.scroll.saturating_add(1);
                    }
                }
            }

            b"k" => {
                let mut clients = self.clients.lock().await;
                let (_, app) = clients.get_mut(&self.id).unwrap();

                match &app.focused_block {
                    FocusedBlock::Toc => {
                        let i = match app.toc_state.selected() {
                            Some(i) => {
                                if i > 1 {
                                    i - 1
                                } else {
                                    0
                                }
                            }
                            None => 0,
                        };

                        app.toc_state.select(Some(i));
                    }
                    FocusedBlock::Post => {
                        app.scroll = app.scroll.saturating_sub(1);
                    }
                }
            }

            // Enter
            [13] => {
                let mut clients = self.clients.lock().await;
                let (_, app) = clients.get_mut(&self.id).unwrap();
                if app.focused_block == FocusedBlock::Toc {
                    app.focused_block = FocusedBlock::Post;
                }
            }

            // Esc
            [27] | [127] => {
                let mut clients = self.clients.lock().await;
                let (_, app) = clients.get_mut(&self.id).unwrap();
                if app.focused_block == FocusedBlock::Post {
                    app.focused_block = FocusedBlock::Toc;
                }
            }

            // G
            [71] => {
                let mut clients = self.clients.lock().await;
                let (_, app) = clients.get_mut(&self.id).unwrap();
                match app.focused_block {
                    FocusedBlock::Post => {
                        if let Some(i) = app.toc_state.selected() {
                            let post_title = &app.toc[i].1;

                            if let Some(post) = app.posts.iter().find(|p| &p.title == post_title) {
                                let length = post.content.height();
                                app.scroll = length as u16;
                            }
                        }
                    }
                    FocusedBlock::Toc => {
                        app.toc_state.select(Some(app.toc.len()));
                    }
                }
            }

            // gg
            [103] => {
                let mut clients = self.clients.lock().await;
                let (_, app) = clients.get_mut(&self.id).unwrap();
                if app.previous_key == vec![103] {
                    match app.focused_block {
                        FocusedBlock::Post => {
                            app.scroll = 0;
                        }
                        FocusedBlock::Toc => {
                            app.toc_state.select(Some(0));
                        }
                    }
                }
            }

            _ => {}
        }

        let mut clients = self.clients.lock().await;
        let (_, app) = clients.get_mut(&self.id).unwrap();
        app.previous_key = data.to_vec();

        Ok(())
    }

    // FIX: not working
    async fn window_change_request(
        &mut self,
        _: ChannelId,
        col_width: u32,
        row_height: u32,
        _: u32,
        _: u32,
        _: &mut Session,
    ) -> Result<(), Self::Error> {
        {
            let mut clients = self.clients.lock().await;
            let (terminal, _) = clients.get_mut(&self.id).unwrap();
            let rect = Rect {
                x: 0,
                y: 0,
                width: col_width as u16,
                height: row_height as u16,
            };
            terminal.resize(rect)?;
        }

        Ok(())
    }
}
