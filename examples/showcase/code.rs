//! A tiny Rust highlighter: enough for builder chains, not a real lexer.

use egui::text::{LayoutJob, TextFormat};
use egui::{Color32, FontId};
use egui_styled::rgb;

use crate::fonts::{Weight, font};

const KEYWORD: Color32 = rgb(198, 120, 221);
const TYPE: Color32 = rgb(229, 192, 123);
const METHOD: Color32 = rgb(97, 175, 239);
const NUMBER: Color32 = rgb(209, 154, 102);
const STRING: Color32 = rgb(152, 195, 121);
const COMMENT: Color32 = rgb(110, 118, 129);
const PLAIN: Color32 = rgb(220, 223, 228);

pub fn highlight(src: &str, size: f32) -> LayoutJob {
    let mono = font(Weight::Mono, size);
    let mut job = LayoutJob::default();
    let mut push = |s: &str, color: Color32| {
        job.append(s, 0.0, TextFormat::simple(FontId::clone(&mono), color));
    };

    for (i, line) in src.lines().enumerate() {
        if i > 0 {
            push("\n", PLAIN);
        }
        if let Some(idx) = line.find("//") {
            highlight_code(&line[..idx], &mut push);
            push(&line[idx..], COMMENT);
        } else {
            highlight_code(line, &mut push);
        }
    }
    job
}

fn highlight_code(line: &str, push: &mut impl FnMut(&str, Color32)) {
    let chars: Vec<(usize, char)> = line.char_indices().collect();
    let mut i = 0;
    while i < chars.len() {
        let (start, c) = chars[i];
        let end_of = |j: usize| chars.get(j).map_or(line.len(), |&(b, _)| b);
        if c == '"' {
            let mut j = i + 1;
            while j < chars.len() && chars[j].1 != '"' {
                j += 1;
            }
            j = (j + 1).min(chars.len());
            push(&line[start..end_of(j)], STRING);
            i = j;
        } else if c.is_ascii_digit() {
            let mut j = i;
            while j < chars.len() && (chars[j].1.is_ascii_digit() || chars[j].1 == '.') {
                j += 1;
            }
            push(&line[start..end_of(j)], NUMBER);
            i = j;
        } else if c.is_alphabetic() || c == '_' {
            let mut j = i;
            while j < chars.len() && (chars[j].1.is_alphanumeric() || chars[j].1 == '_') {
                j += 1;
            }
            let word = &line[start..end_of(j)];
            let after_dot = i > 0 && chars[i - 1].1 == '.';
            let color = match word {
                "fn" | "let" | "move" | "impl" | "mut" | "pub" | "use" | "match" => KEYWORD,
                _ if after_dot => METHOD,
                _ if word.starts_with(char::is_uppercase) => TYPE,
                _ => PLAIN,
            };
            push(word, color);
            i = j;
        } else {
            push(&line[start..end_of(i + 1)], PLAIN);
            i += 1;
        }
    }
}
