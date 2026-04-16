use iced::{
    Element,
    advanced::widget::Text,
    widget::{
        column,
        markdown::{self, Item},
    },
};
use regex::Regex;

use crate::config::font_settings::markdown_settings;

#[derive(Debug)]
/// Denotes what the markdown is, standard markdown text or a parsed image
pub enum MarkdownElement {
    Standard(String),
    Image(ParsedImage),
}

#[derive(Debug)]
/// Data extracted from image markdown of the following form:
/// ```text
/// ![no_image_text](path "hover text"){width=100 height=100}
/// ```
/// Note the only required field is the path, so all of the following are valid:
/// ```text
/// ![](path)
/// ![no_image_text](path)
/// ![](path "hover text"){width=100 height=100}
/// ```
pub struct ParsedImage {
    no_image_text: String,
    path: String,
    hover_text: Option<String>,
    dimensions: Option<(usize, usize)>,
}

#[derive(Debug)]
/// Parsed markdown that is ready to be rendered
pub enum ParsedMarkdown {
    Iced(Vec<Item>),
    Image(ParsedImage),
}

impl ParsedImage {
    /// Returns the indexes in the markdown that contain either fenced or inline code blocks
    fn get_ignored_ranges(markdown: &str) -> Vec<(usize, usize)> {
        let mut ignored_ranges = Vec::new();

        let code_block_regex =
            Regex::new(r#"```[^\n]*\n[\s\S]*?\n```|~~~[^\n]*\n[\s\S]*?\n~~~|`[^`\n]+`"#)
                .expect("bad regex");

        for found_match in code_block_regex.find_iter(markdown) {
            ignored_ranges.push((found_match.start(), found_match.end()));
        }

        ignored_ranges
    }

    /// Splits a markdown string into substrings that either contain standard markdown or image markdown. Images that
    /// are inside code blocks (see get_ignored_ranges) are not considered image markdown, and will end up in the
    /// standard markdown substrings
    fn split_on_image(markdown: &str) -> Vec<String> {
        let mut split_strings = Vec::new();

        let image_regex = Regex::new(r#"!\[([^\]]*)\]\(\s*([^\s)]+)\s*(?:["]([^"]*)["])?\s*\)(?:\s*\{\s*width\s*=\s*(\d+)\s+height\s*=\s*(\d+)\s*\})?"#).expect("bad regex");

        let ignored_ranges = Self::get_ignored_ranges(markdown);

        let mut current_last_char = 0;

        for found_match in image_regex.find_iter(markdown) {
            let is_ignored = ignored_ranges
                .iter()
                .any(|(start, end)| found_match.start() >= *start && found_match.end() <= *end);

            if is_ignored {
                continue;
            }

            let before_match = markdown[current_last_char..found_match.start()].to_string();

            if !before_match.is_empty() {
                split_strings.push(before_match);
            }

            let match_string = found_match.as_str().to_string();

            split_strings.push(match_string);

            current_last_char = found_match.end();
        }

        let end_string = markdown[current_last_char..].to_string();

        if !end_string.is_empty() {
            split_strings.push(end_string);
        }

        split_strings
    }

    /// Attempts to parse the markdown for information required in a ParsedImage
    fn parse_markdown_image(image_text: &str) -> Option<ParsedImage> {
        // TODO: use same regex as the splitter: detect false hits and deal with them
        let image_regex = Regex::new(
        r#"^!\[([^\]]*)\]\(\s*([^\s)]+)\s*(?:["]([^"]*)["])?\s*\)(?:\s*\{\s*width\s*=\s*(\d+)\s+height\s*=\s*(\d+)\s*\})?$"#
    ).ok()?;

        let captures = image_regex.captures(image_text)?;

        let no_image_text = captures.get(1)?.as_str().to_string();
        let path = captures.get(2)?.as_str().to_string();
        let hover_text = captures.get(3).map(|mat| mat.as_str().to_string());

        let dimensions = match (captures.get(4), captures.get(5)) {
            (Some(width), Some(height)) => {
                Some((width.as_str().parse().ok()?, height.as_str().parse().ok()?))
            }
            _ => None,
        };

        Some(Self {
            no_image_text,
            path,
            hover_text,
            dimensions,
        })
    }
}

/// Determines a collection of the markdown
fn classify(markdown: &str) -> Vec<MarkdownElement> {
    let split = ParsedImage::split_on_image(markdown);

    let classified = split
        .iter()
        .map(|text| {
            let parsed = ParsedImage::parse_markdown_image(text);

            if let Some(image) = parsed {
                MarkdownElement::Image(image)
            } else {
                MarkdownElement::Standard(text.to_string())
            }
        })
        .collect::<Vec<MarkdownElement>>();

    let mut simplified_parsed = Vec::new();

    let mut accumulated_markdown = String::new();

    for element in classified {
        match element {
            MarkdownElement::Standard(markdown) => {
                accumulated_markdown.push_str(&markdown);
            }
            MarkdownElement::Image(markdown_image) => {
                if !accumulated_markdown.is_empty() {
                    simplified_parsed.push(MarkdownElement::Standard(accumulated_markdown.clone()));
                    accumulated_markdown.clear();
                }

                simplified_parsed.push(MarkdownElement::Image(markdown_image));
            }
        }
    }

    if !accumulated_markdown.is_empty() {
        simplified_parsed.push(MarkdownElement::Standard(accumulated_markdown));
    }

    simplified_parsed
}

/// Parses a markdown string into a collection of ParsedMarkdown ready to be rendered
pub fn parse(markdown: &str) -> Vec<ParsedMarkdown> {
    let elements = classify(markdown);

    elements
        .into_iter()
        .map(|e| match e {
            MarkdownElement::Standard(markdown) => {
                ParsedMarkdown::Iced(markdown::parse(&markdown).collect::<Vec<Item>>())
            }
            MarkdownElement::Image(markdown_image) => ParsedMarkdown::Image(markdown_image),
        })
        .collect()
}

/// Renders the given parsed markdown
pub fn build_markdown<'a, M: 'a + Clone>(
    to_render: &'a Vec<ParsedMarkdown>,
    markdown_message: M,
) -> Element<'a, M> {
    let mut rendered = column![];

    for parsed in to_render {
        match parsed {
            ParsedMarkdown::Iced(items) => {
                let message = markdown_message.clone();
                rendered = rendered.push(
                    markdown::view(items, markdown_settings()).map(move |_link| message.clone()),
                );
            }
            ParsedMarkdown::Image(markfown_image) => {
                rendered = rendered.push(Text::new(format!("{:#?}", markfown_image)).size(16));
            }
        }
    }

    rendered.into()
}
