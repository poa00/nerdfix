//! Parses font infomation from nerd font official cheat sheet.

use crate::{db::Icon, error};
use select::document::Document;

pub fn parse(s: &str) -> error::Result<Vec<Icon>> {
    let mut parser = Parser::new(s);
    parser.parse()?;
    Ok(parser.icons)
}

pub struct Parser<'a> {
    document: Document,
    icons: Vec<Icon>,
    _source: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            document: Document::from(s),
            icons: Default::default(),
            _source: s,
        }
    }

    pub fn parse(&mut self) -> error::Result<()> {
        use select::predicate::*;
        for node in self
            .document
            .find(Attr("id", "glyphCheatSheet").child(Element))
        {
            (|| {
                let name = node
                    .find(Class("class-name").child(Text))
                    .next()?
                    .as_text()?
                    .trim();
                let codepoint = node
                    .find(Class("codepoint").child(Text))
                    .next()?
                    .as_text()?;
                let codepoint = u32::from_str_radix(codepoint, 16).ok()?;
                let obsolete = (|| {
                    node.find(Class("corner-text").child(Text))
                        .next()?
                        .as_text()
                })();
                let obsolete = obsolete == Some("obsolete");
                self.icons.push(Icon {
                    name: name.to_owned(),
                    codepoint,
                    obsolete,
                });
                Some(())
            })();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // Author: Ryan L McIntyre
    // License: MIT
    // Upstream: https://github.com/ryanoasis/nerd-fonts/blob/gh-pages/_posts/2017-01-04-icon-cheat-sheet.md
    const CHEAT_SHEET: &str = r#"
<div id="glyphCheatSheet" class="nerd-font-cheat-sheet">
  <div class="column">
    <div class="nf nf-cod-account center"></div>
    <div class="class-name">nf-cod-account</div><div title="Copy Hex Code to Clipboard" class="codepoint">eb99</div>
  </div>
  <div class="column">
    <div class="nf nf-cod-activate_breakpoints center"></div>
    <div class="class-name">nf-cod-activate_breakpoints</div><div title="Copy Hex Code to Clipboard" class="codepoint">ea97</div>
  </div>
  <div class="column">
    <span class="corner-red"></span><span class="corner-text">obsolete</span>
    <div class="nf nf-mdi-access_point center"></div>
    <div class="class-name">nf-mdi-access_point</div><div title="Copy Hex Code to Clipboard" class="codepoint">f501</div>
  </div>
  <div class="column">
    <span class="corner-red"></span><span class="corner-text">obsolete</span>
    <div class="nf nf-mdi-access_point_network center"></div>
    <div class="class-name">nf-mdi-access_point_network</div><div title="Copy Hex Code to Clipboard" class="codepoint">f502</div>
  </div>
</div>"#;

    #[test]
    fn parser() {
        let icons = super::parse(CHEAT_SHEET).unwrap();
        let expected = vec![
            icon!("nf-cod-account", 0xeb99),
            icon!("nf-cod-activate_breakpoints", 0xea97),
            icon!("nf-mdi-access_point", 0xf501, true),
            icon!("nf-mdi-access_point_network", 0xf502, true),
        ];
        assert_eq!(icons, expected);
    }
}