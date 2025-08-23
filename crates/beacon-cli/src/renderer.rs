//! Terminal rendering module for rich markdown output
//!
//! This module provides terminal rendering capabilities using termimad
//! for rich markdown display with optional fallback to plain text.

use anyhow::Result;
use termimad::{crossterm::style::Color, MadSkin};

/// Terminal renderer that can switch between rich and plain text output
pub struct TerminalRenderer {
    rich_enabled: bool,
    skin: MadSkin,
}

impl TerminalRenderer {
    /// Create a new terminal renderer
    pub fn new(rich_enabled: bool) -> Self {
        let mut skin = MadSkin::default();

        // Configure termimad skin for better appearance
        skin.set_headers_fg(Color::Blue);
        skin.bold.set_fg(Color::Yellow);
        skin.italic.set_fg(Color::Magenta);
        skin.code_block.set_bg(Color::AnsiValue(238));
        skin.inline_code.set_bg(Color::AnsiValue(238));

        // Keep standard header configuration for inline text styling
        // We'll manually handle hash symbols in the render method

        Self { rich_enabled, skin }
    }

    /// Render markdown text to terminal
    pub fn render(&self, markdown: &str) -> Result<()> {
        if self.rich_enabled {
            // Process line by line to show hash symbols for headers
            for line in markdown.lines() {
                if line.starts_with('#') {
                    print!("\x1b[34m{line}\x1b[0m");
                    println!();
                } else {
                    // For non-header lines, use regular rendering
                    self.skin.print_inline(line);
                    println!();
                }
            }
        } else {
            print!("{}", markdown);
        }
        Ok(())
    }
}

impl Default for TerminalRenderer {
    fn default() -> Self {
        Self::new(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_renderer() {
        let renderer = TerminalRenderer::new(false);
        assert!(!renderer.rich_enabled);
    }

    #[test]
    fn test_rich_renderer() {
        let renderer = TerminalRenderer::new(true);
        assert!(renderer.rich_enabled);
    }

    #[test]
    fn test_default_is_rich() {
        let renderer = TerminalRenderer::default();
        assert!(renderer.rich_enabled);
    }
}
