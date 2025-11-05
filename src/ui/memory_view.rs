use tui::{
    style::Style,
    text::{Span, Spans},
    widgets::{Paragraph, Widget},
};

use crate::instruction::Address;

pub struct MemoryView<'a> {
    memory: &'a [u8],
    shown_address: u16,
    address_style: Style,
    data_style: Style,
    label_style: Style,
}

impl<'a> MemoryView<'a> {
    pub fn new(memory: &'a [u8]) -> Self {
        Self {
            memory,
            shown_address: 0,
            address_style: Style::default(),
            data_style: Style::default(),
            label_style: Style::default(),
        }
    }

    pub fn shown_address(mut self, address: Address) -> Self {
        self.shown_address = address.value();
        self
    }

    pub fn address_style(mut self, style: Style) -> Self {
        self.address_style = style;
        self
    }

    pub fn data_style(mut self, style: Style) -> Self {
        self.data_style = style;
        self
    }

    pub fn label_style(mut self, style: Style) -> Self {
        self.label_style = style;
        self
    }
}

impl<'a> Widget for MemoryView<'a> {
    fn render(self, mut area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        // Available length of characters to draw bytes to.
        let memory_area_width = area.width - 8;

        // This calculation takes into consideration that the last byte doesn't need to be followed
        // by a space (and is therefore only 2 characters wide).
        // Proof:
        // Let n = row_byte_count
        //     w = memory_area_width
        // in
        //   (n*2 + (n-1)) <= w
        //            3n-1 <= w
        //              3n <= w + 1
        //               n <= (w + 1) / 3
        // We want to maximize n, therefore
        //   n = floor((w + 1) / 3)
        let row_byte_count = (memory_area_width + 1) / 3;

        static MAX_ROW_BYTES: u16 = 16;
        let row_byte_count = row_byte_count.min(MAX_ROW_BYTES);

        // Draw first line
        Paragraph::new(Spans::from(
            [Span::styled("Offset", self.label_style), Span::raw("  ")]
                .into_iter()
                .chain(
                    (0..row_byte_count)
                        .map(|byte_index| {
                            [Span::styled(
                                format!("{:02x}", byte_index),
                                self.address_style,
                            )]
                        })
                        .collect::<Box<[_]>>()
                        .join(&Span::raw("  ")),
                )
                .collect::<Vec<_>>(),
        ))
        .render(area, buf);

        area.y += 1;
        area.height -= 1;

        let rows = area.height;
        let showable_span_len = rows * row_byte_count;

        let view_start_offset = self.shown_address.saturating_sub(showable_span_len / 2);

        for row_index in 0..rows {
            let offset = view_start_offset + row_index * row_byte_count;

            let mut row_area = area;
            row_area.height = 1;
            row_area.y += row_index;

            Paragraph::new(Spans::from(
                [
                    Span::raw("  "),
                    Span::styled(format!("{:04x}", offset), self.address_style),
                    Span::raw("  "),
                ]
                .into_iter()
                .chain(
                    (0..row_byte_count)
                        .map(|byte_index| {
                            [Span::styled(
                                format!("{:02x}", self.memory[(offset + byte_index) as usize]),
                                self.data_style,
                            )]
                        })
                        .collect::<Box<[_]>>()
                        .join(&Span::raw("  ")),
                )
                .collect::<Vec<_>>(),
            ))
            .render(row_area, buf);
        }
    }
}
