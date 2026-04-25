use std::{
    io::{IsTerminal, Write, stdout},
    time::Duration,
};

use crossterm::{
    ExecutableCommand, QueueableCommand, cursor,
    style::{Color, Print, ResetColor, SetForegroundColor, Stylize},
    terminal::{Clear, ClearType, size},
};
use tokio::{sync::watch, task::JoinHandle};

const ORB_FRAMES: &[&str] = &["◌", "◍", "◎", "◉", "●", "◉", "◎", "◍"];
const SPARKLES: &[&str] = &["✦", "✧", "✩", "✶", "·", "˚", " "];
const MESSAGES: &[&str] = &[
    "light gathers behind the glass...",
    "blue sparks drift through the veil...",
    "violet threads find their pattern...",
    "pink fireflies orbit the answer...",
    "cyan mist clears from the crystal...",
    "the orb listens for a useful truth...",
];
const GRADIENT: &[Color] = &[
    Color::Rgb {
        r: 75,
        g: 179,
        b: 255,
    },
    Color::Rgb {
        r: 104,
        g: 118,
        b: 255,
    },
    Color::Rgb {
        r: 173,
        g: 105,
        b: 255,
    },
    Color::Rgb {
        r: 237,
        g: 94,
        b: 255,
    },
    Color::Rgb {
        r: 255,
        g: 112,
        b: 199,
    },
    Color::Rgb {
        r: 74,
        g: 226,
        b: 255,
    },
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
                    .and_then(|out| out.queue(Clear(ClearType::CurrentLine)));

                let _ = write_shifted_line(&mut out, &line, tick).and_then(|_| out.flush());

                tick = tick.wrapping_add(1);

                tokio::select! {
                    _ = should_stop.changed() => {}
                    _ = tokio::time::sleep(Duration::from_millis(140)) => {}
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

pub fn print_answer(answer: &str) -> std::io::Result<()> {
    if !stdout().is_terminal() {
        println!("{answer}");
        return Ok(());
    }

    let mut out = stdout();
    let width = frame_width(answer);
    let inner_width = width.saturating_sub(4);
    let border = "─".repeat(width.saturating_sub(2));

    out.queue(SetForegroundColor(GRADIENT[0]))?
        .queue(Print(format!("╭{border}╮\n")))?;

    let title = "✦ crystal ball ✦";
    write_centered(&mut out, title, inner_width, GRADIENT[3])?;

    let orb_lines = ["     ✦     ", "   ◌ ◉ ◌   ", " ✧  ◍ ◎  ✧ ", "   ◌ ◉ ◌   "];
    for (idx, line) in orb_lines.iter().enumerate() {
        write_centered(
            &mut out,
            line,
            inner_width,
            GRADIENT[(idx + 1) % GRADIENT.len()],
        )?;
    }

    write_empty(&mut out, inner_width)?;

    for line in render_markdown(answer, inner_width) {
        write_answer_line(&mut out, &line, inner_width)?;
    }

    out.queue(SetForegroundColor(GRADIENT[4]))?
        .queue(Print(format!("╰{border}╯\n")))?
        .queue(ResetColor)?
        .flush()
}

fn write_shifted_line<W: Write>(out: &mut W, text: &str, tick: usize) -> std::io::Result<()> {
    let color = GRADIENT[(tick / 2) % GRADIENT.len()];
    out.queue(SetForegroundColor(color))?
        .queue(Print(text))?
        .queue(ResetColor)?;

    Ok(())
}

fn frame_width(answer: &str) -> usize {
    let terminal_width = size().map(|(width, _)| width as usize).unwrap_or(80);
    let longest_line = answer.lines().map(str::len).max().unwrap_or(0);
    let desired = longest_line.clamp(42, 76) + 4;

    desired.min(terminal_width.saturating_sub(2)).max(42)
}

fn write_centered<W: Write>(
    out: &mut W,
    text: &str,
    width: usize,
    color: Color,
) -> std::io::Result<()> {
    let text_width = text.chars().count();
    let left = width.saturating_sub(text_width) / 2;
    let right = width.saturating_sub(text_width + left);

    out.queue(SetForegroundColor(color))?
        .queue(Print("│ "))?
        .queue(Print(" ".repeat(left)))?
        .queue(Print(text.bold()))?
        .queue(Print(" ".repeat(right)))?
        .queue(Print(" │\n"))?
        .queue(ResetColor)?;

    Ok(())
}

fn write_empty<W: Write>(out: &mut W, width: usize) -> std::io::Result<()> {
    out.queue(SetForegroundColor(GRADIENT[2]))?
        .queue(Print("│ "))?
        .queue(Print(" ".repeat(width)))?
        .queue(Print(" │\n"))?
        .queue(ResetColor)?;

    Ok(())
}

struct AnswerLine {
    text: String,
    color: Color,
    bold: bool,
}

fn render_markdown(markdown: &str, width: usize) -> Vec<AnswerLine> {
    let mut lines = Vec::new();
    let mut in_code_block = false;

    for raw_line in markdown.lines() {
        let trimmed = raw_line.trim();

        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_code_block = !in_code_block;
            let text = if in_code_block {
                "╭─ code"
            } else {
                "╰─────"
            };
            push_wrapped(&mut lines, text, "", "", width, GRADIENT[1], in_code_block);
            continue;
        }

        if in_code_block {
            push_wrapped(
                &mut lines,
                raw_line,
                "  ",
                "  ",
                width,
                Color::Rgb {
                    r: 188,
                    g: 226,
                    b: 255,
                },
                false,
            );
            continue;
        }

        if trimmed.is_empty() {
            if !lines
                .last()
                .is_some_and(|line: &AnswerLine| line.text.is_empty())
            {
                lines.push(AnswerLine {
                    text: String::new(),
                    color: GRADIENT[5],
                    bold: false,
                });
            }
            continue;
        }

        if is_rule(trimmed) {
            lines.push(AnswerLine {
                text: "─".repeat(width.min(42)),
                color: GRADIENT[2],
                bold: false,
            });
            continue;
        }

        if let Some(heading) = parse_heading(trimmed) {
            push_wrapped(
                &mut lines,
                &format!("✦ {}", strip_inline_markdown(heading)),
                "",
                "  ",
                width,
                GRADIENT[3],
                true,
            );
            continue;
        }

        if let Some(quote) = trimmed.strip_prefix('>') {
            push_wrapped(
                &mut lines,
                &strip_inline_markdown(quote.trim_start()),
                "│ ",
                "│ ",
                width,
                GRADIENT[2],
                false,
            );
            continue;
        }

        if let Some(item) = parse_bullet(trimmed) {
            push_wrapped(
                &mut lines,
                &strip_inline_markdown(item),
                "• ",
                "  ",
                width,
                GRADIENT[4],
                false,
            );
            continue;
        }

        if let Some((number, item)) = parse_ordered_item(trimmed) {
            let prefix = format!("{number}. ");
            let continuation = " ".repeat(prefix.chars().count());
            push_wrapped(
                &mut lines,
                &strip_inline_markdown(item),
                &prefix,
                &continuation,
                width,
                GRADIENT[4],
                false,
            );
            continue;
        }

        push_wrapped(
            &mut lines,
            &strip_inline_markdown(trimmed),
            "",
            "",
            width,
            Color::White,
            false,
        );
    }

    if lines.is_empty() {
        lines.push(AnswerLine {
            text: String::new(),
            color: Color::White,
            bold: false,
        });
    }

    lines
}

fn write_answer_line<W: Write>(
    out: &mut W,
    line: &AnswerLine,
    width: usize,
) -> std::io::Result<()> {
    out.queue(SetForegroundColor(GRADIENT[5]))?
        .queue(Print("│ "))?
        .queue(SetForegroundColor(line.color))?;

    if line.bold {
        out.queue(Print(line.text.as_str().bold()))?;
    } else {
        out.queue(Print(&line.text))?;
    }

    out.queue(ResetColor)?
        .queue(Print(
            " ".repeat(width.saturating_sub(visible_len(&line.text))),
        ))?
        .queue(SetForegroundColor(GRADIENT[5]))?
        .queue(Print(" │\n"))?;

    Ok(())
}

fn push_wrapped(
    lines: &mut Vec<AnswerLine>,
    text: &str,
    first_prefix: &str,
    continuation_prefix: &str,
    width: usize,
    color: Color,
    bold: bool,
) {
    let available = width.saturating_sub(visible_len(first_prefix)).max(1);
    let continuation_available = width
        .saturating_sub(visible_len(continuation_prefix))
        .max(1);

    for (idx, wrapped_line) in wrap_words(text, available, continuation_available)
        .into_iter()
        .enumerate()
    {
        let prefix = if idx == 0 {
            first_prefix
        } else {
            continuation_prefix
        };

        lines.push(AnswerLine {
            text: format!("{prefix}{wrapped_line}"),
            color,
            bold,
        });
    }
}

fn wrap_words(text: &str, first_width: usize, continuation_width: usize) -> Vec<String> {
    if text.is_empty() {
        return vec![String::new()];
    }

    let mut lines = Vec::new();
    let mut current = String::new();
    let mut current_width = first_width;

    for word in text.split_whitespace() {
        let word_len = visible_len(word);
        let next_len = if current.is_empty() {
            word_len
        } else {
            visible_len(&current) + 1 + word_len
        };

        if next_len > current_width && !current.is_empty() {
            lines.push(current);
            current = String::new();
            current_width = continuation_width;
        }

        if word_len > current_width {
            if !current.is_empty() {
                lines.push(current);
                current = String::new();
                current_width = continuation_width;
            }

            lines.extend(split_long_word(word, current_width));
            continue;
        }

        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }

    if !current.is_empty() {
        lines.push(current);
    }

    lines
}

fn split_long_word(word: &str, width: usize) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();

    for ch in word.chars() {
        if visible_len(&current) >= width {
            parts.push(current);
            current = String::new();
        }
        current.push(ch);
    }

    if !current.is_empty() {
        parts.push(current);
    }

    parts
}

fn parse_heading(line: &str) -> Option<&str> {
    let hashes = line.chars().take_while(|ch| *ch == '#').count();

    if (1..=6).contains(&hashes) && line.chars().nth(hashes) == Some(' ') {
        Some(line[hashes + 1..].trim())
    } else {
        None
    }
}

fn parse_bullet(line: &str) -> Option<&str> {
    line.strip_prefix("- ")
        .or_else(|| line.strip_prefix("* "))
        .or_else(|| line.strip_prefix("+ "))
}

fn parse_ordered_item(line: &str) -> Option<(&str, &str)> {
    let dot = line.find('.')?;
    let number = &line[..dot];
    let rest = line.get(dot + 1..)?;

    if number.chars().all(|ch| ch.is_ascii_digit()) && rest.starts_with(' ') {
        Some((number, rest.trim_start()))
    } else {
        None
    }
}

fn is_rule(line: &str) -> bool {
    let without_spaces = line.replace(' ', "");
    let first = without_spaces.chars().next();

    without_spaces.len() >= 3
        && matches!(first, Some('-') | Some('*') | Some('_'))
        && without_spaces.chars().all(|ch| Some(ch) == first)
}

fn strip_inline_markdown(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut output = String::new();
    let mut idx = 0;

    while idx < chars.len() {
        match chars[idx] {
            '`' => {
                idx += 1;
                while idx < chars.len() && chars[idx] != '`' {
                    output.push(chars[idx]);
                    idx += 1;
                }
                idx += usize::from(idx < chars.len());
            }
            '*' | '_' => {
                idx += 1;
                if idx < chars.len() && chars[idx] == chars[idx - 1] {
                    idx += 1;
                }
            }
            '!' if chars.get(idx + 1) == Some(&'[') => {
                idx += 1;
            }
            '[' => {
                if let Some((label, url, next_idx)) = parse_link(&chars, idx) {
                    output.push_str(&label);
                    if !url.is_empty() {
                        output.push_str(" (");
                        output.push_str(&url);
                        output.push(')');
                    }
                    idx = next_idx;
                } else {
                    output.push(chars[idx]);
                    idx += 1;
                }
            }
            ch => {
                output.push(ch);
                idx += 1;
            }
        }
    }

    output
}

fn parse_link(chars: &[char], start: usize) -> Option<(String, String, usize)> {
    let close_label = chars[start..].iter().position(|ch| *ch == ']')? + start;
    if chars.get(close_label + 1) != Some(&'(') {
        return None;
    }

    let close_url = chars[close_label + 2..].iter().position(|ch| *ch == ')')? + close_label + 2;
    let label = chars[start + 1..close_label].iter().collect();
    let url = chars[close_label + 2..close_url].iter().collect();

    Some((label, url, close_url + 1))
}

fn visible_len(text: &str) -> usize {
    text.chars().count()
}
