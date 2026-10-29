use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Widget},
};
use super::symbol_index::{CompletionItem, CompletionKind};

pub struct PopupWidget<'a> {
    items: &'a [CompletionItem],
    selected_index: usize,
}

impl<'a> PopupWidget<'a> {
    pub fn new(items: &'a [CompletionItem], selected_index: usize) -> Self {
        Self {
            items,
            selected_index,
        }
    }
}

impl<'a> Widget for PopupWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.items.is_empty() {
            let block = Block::default()
                .borders(Borders::ALL)
                .title(" @ References ")
                .style(Style::default().bg(Color::Reset).fg(Color::DarkGray));
            let paragraph = Paragraph::new("No matching references")
                .block(block)
                .style(Style::default().fg(Color::DarkGray));
            paragraph.render(area, buf);
            return;
        }

        let list_items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let is_selected = i == self.selected_index;

                let kind_str = match &item.kind {
                    CompletionKind::File(_) => "[file]",
                    CompletionKind::Function(_) => "[fn]",
                    CompletionKind::Struct(_) => "[struct]",
                    CompletionKind::Module(_) => "[mod]",
                };

                let content = format!(" {} {} ({} tok)", kind_str, item.display_text, item.token_cost);

                let style = if is_selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                ListItem::new(content).style(style)
            })
            .collect();

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" @ References (Tab/Enter) ")
            .style(Style::default().bg(Color::Black).fg(Color::White));

        let list = List::new(list_items).block(block);
        list.render(area, buf);
    }
}
