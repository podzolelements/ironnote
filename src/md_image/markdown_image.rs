use iced::{
    Element,
    advanced::widget::Text,
    widget::{
        Image, column,
        image::Handle,
        markdown::{self, Item},
        tooltip,
    },
};
use image::{DynamicImage, imageops::FilterType};
use regex::Regex;
use std::{collections::HashMap, path::PathBuf, sync::LazyLock};

use crate::{config::font_settings::markdown_settings, ui::styling::TOOLTIP_DELAY};

#[derive(Debug)]
/// Denotes what the markdown is, standard markdown text or a parsed image
pub enum MarkdownElement {
    Standard(String),
    Image(ParsedImage),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    dimensions: Option<(u32, u32)>,
}

#[derive(Debug, Default)]
/// Stores raw and processed images in memory to avoid loading and processing every frame
pub struct ImageCache {
    disk_images: HashMap<String, DynamicImage>,
    proccessed_images: HashMap<ParsedImage, Handle>,
}

#[derive(Debug)]
/// Parsed markdown that is ready to be rendered
pub enum ParsedMarkdown {
    Iced(Vec<Item>),
    Image(ParsedImage),
}

static MARKDOWN_IMAGE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"!\[([^\]]*)\]\(\s*([^\s)]+)\s*(?:["]([^"]*)["])?\s*\)(?:\s*\{\s*width\s*=\s*(\d+)\s+height\s*=\s*(\d+)\s*\})?"#).expect("bad regex")
});
static CODE_BLOCK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"```[^\n]*\n[\s\S]*?\n```|~~~[^\n]*\n[\s\S]*?\n~~~|`[^`\n]+`"#).expect("bad regex")
});

impl ParsedImage {
    /// Returns the indexes in the markdown that contain either fenced or inline code blocks
    fn get_ignored_ranges(markdown: &str) -> Vec<(usize, usize)> {
        let mut ignored_ranges = Vec::new();

        for found_match in CODE_BLOCK_REGEX.find_iter(markdown) {
            ignored_ranges.push((found_match.start(), found_match.end()));
        }

        ignored_ranges
    }

    /// Splits a markdown string into substrings that either contain standard markdown or image markdown. Images that
    /// are inside code blocks (see get_ignored_ranges) are not considered image markdown, and will end up in the
    /// standard markdown substrings
    fn split_on_image(markdown: &str) -> Vec<String> {
        let mut split_strings = Vec::new();

        let ignored_ranges = Self::get_ignored_ranges(markdown);

        let mut current_last_char = 0;

        for found_match in MARKDOWN_IMAGE_REGEX.find_iter(markdown) {
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
        let is_text_only_an_image = MARKDOWN_IMAGE_REGEX
            .find(image_text)
            .map(|mat| mat.start() == 0 && mat.end() == image_text.len())
            .unwrap_or(false);

        if !is_text_only_an_image {
            return None;
        }

        let captures = MARKDOWN_IMAGE_REGEX.captures(image_text)?;

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
pub fn parse(markdown: &str, image_cache: &mut ImageCache) -> Vec<ParsedMarkdown> {
    let elements = classify(markdown);

    for element in &elements {
        match element {
            MarkdownElement::Standard(_) => {
                continue;
            }
            MarkdownElement::Image(parsed_image) => {
                let path_string = parsed_image.path.clone();

                let image_path = PathBuf::from(path_string.clone());

                if !image_cache.disk_images.contains_key(&path_string)
                    && let Ok(image) = image::open(image_path)
                {
                    let rgba8_image = DynamicImage::from(image.to_rgba8());

                    image_cache
                        .disk_images
                        .insert(path_string.clone(), rgba8_image);
                }

                if let Some(image) = image_cache.disk_images.get(&path_string)
                    && !image_cache.proccessed_images.contains_key(parsed_image)
                {
                    let proccessed_image = if let Some((width, height)) = parsed_image.dimensions {
                        image.resize(width, height, FilterType::Gaussian)
                    } else {
                        image.clone()
                    };
                    let handle = Handle::from_rgba(
                        proccessed_image.width(),
                        proccessed_image.height(),
                        proccessed_image.clone().into_bytes(),
                    );

                    image_cache
                        .proccessed_images
                        .insert(parsed_image.clone(), handle);
                }
            }
        }
    }

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
    image_cache: &ImageCache,
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
            ParsedMarkdown::Image(parsed_image) => {
                if let Some(handle) = image_cache.proccessed_images.get(parsed_image) {
                    let image = Image::new(handle);

                    let with_hover_text = if let Some(hover_text) = &parsed_image.hover_text {
                        column![
                            tooltip(
                                image,
                                Text::new(hover_text).size(15),
                                // TODO: disapear on mouse movement
                                tooltip::Position::FollowCursor,
                            )
                            .delay(TOOLTIP_DELAY)
                        ]
                    } else {
                        column![image]
                    };

                    rendered = rendered.push(with_hover_text);
                } else {
                    // TODO: draw broken image
                    let no_image_text = Text::new(parsed_image.no_image_text.clone());

                    rendered = rendered.push(no_image_text);
                }
            }
        }
    }

    rendered.into()
}
