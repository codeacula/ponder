use std::{
    io::{IsTerminal, Write, stdout},
    time::Duration,
};

use crossterm::{
    ExecutableCommand, QueueableCommand, cursor,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use tokio::{sync::watch, task::JoinHandle};

const ORB_FRAMES: &[&str] = &["◌", "◍", "◎", "◉", "◎", "◍"];
const SPARKLES: &[&str] = &["✦", "✧", "·", "*", "˚", " "];
const MESSAGES: &[&str] = &[
    "consulting the hidden lattice...",
    "tracing luminous threads...",
    "listening for improbable truths...",
    "distilling fragments into shape...",
    "reading the weather of symbols...",
    "asking the orb to blink twice...",
];

pub struct WaitUi {
    stop: Option<watch::Sender<bool>>,
    task: Option<JoinHandle<()>>,
}

impl WaitUi {
    pub fn start(show_orb: bool, show_mystical: bool) -> Self {
        if !stdout().is_terminal() {
            return Self {
                stop: None,
                task: None,
            };
        }

        let (stop, mut should_stop) = watch::channel(false);
        let task = tokio::spawn(async move {
            let mut tick = 0usize;
            let mut out = stdout();

            loop {
                if *should_stop.borrow() {
                    break;
                }

                let orb = if show_orb {
                    let left = SPARKLES[tick % SPARKLES.len()];
                    let frame = ORB_FRAMES[tick % ORB_FRAMES.len()];
                    let right = SPARKLES[(tick + 2) % SPARKLES.len()];
                    format!("{left} {frame} {right}")
                } else {
                    String::new()
                };

                let message = if show_mystical {
                    MESSAGES[(tick / 10) % MESSAGES.len()]
                } else {
                    "pondering..."
                };

                let line = if show_orb {
                    format!("{orb}  {message}")
                } else {
                    message.to_string()
                };

                let _ = out
                    .queue(cursor::MoveToColumn(0))
                    .and_then(|out| out.queue(Clear(ClearType::CurrentLine)))
                    .and_then(|out| out.queue(SetForegroundColor(Color::Magenta)))
                    .and_then(|out| out.queue(Print(line)))
                    .and_then(|out| out.queue(ResetColor))
                    .and_then(|out| out.flush());

                tick = tick.wrapping_add(1);

                tokio::select! {
                    _ = should_stop.changed() => {}
                    _ = tokio::time::sleep(Duration::from_millis(90)) => {}
                }
            }

            let _ = out
                .execute(cursor::MoveToColumn(0))
                .and_then(|out| out.execute(Clear(ClearType::CurrentLine)))
                .and_then(|out| out.flush());
        });

        Self {
            stop: Some(stop),
            task: Some(task),
        }
    }

    pub async fn stop(self) {
        if let Some(stop) = self.stop {
            let _ = stop.send(true);
        }

        if let Some(task) = self.task {
            let _ = task.await;
        }
    }
}
