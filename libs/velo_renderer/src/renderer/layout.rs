use super::Renderer;
use crate::types::{rect::Rect, result_item::ResultItem};

pub struct LayoutItem {
    pub rect: Rect,
    pub text_pos: (f32, f32),
    pub selected: bool,
    pub text: String,
}

impl Renderer {
    pub(super) fn compute_layout(&self, items: &[ResultItem]) -> Vec<LayoutItem> {
        let mut y = self.dims.input_height + self.dims.padding;

        items
            .iter()
            .map(|item| {
                let rect = Rect {
                    x: self.dims.padding,
                    y,
                    w: self.dims.width,
                    h: self.dims.item_height,
                };

                let layout = LayoutItem {
                    rect,
                    text_pos: (rect.x + 10.0, rect.y + 6.0),
                    selected: item.selected,
                    text: item.title.clone(),
                };

                y += self.dims.item_height + self.dims.spacing;
                layout
            })
            .collect()
    }
}
