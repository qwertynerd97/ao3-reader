use crate::device::CURRENT_DEVICE;
use crate::font::{Fonts, font_from_style, Style, RenderPlan};
use super::{View, Event, Hub, Bus, Id, ID_FEEDER, RenderQueue, RenderData, Align};
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::geom::Rectangle;
use crate::color::{TEXT_NORMAL};
use crate::app::Context;

#[derive(Clone)]
pub struct Text {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    color: u8,
    background: u8,
    text: String,
    align: Align,
    font: Style,
    plan: Option<RenderPlan>
}

impl Text {
    pub fn new(rect: Rectangle, text: String, align: Align, font: Style) -> Text {
        Text {
            id: ID_FEEDER.next(),
            rect,
            children: vec![],
            color: TEXT_NORMAL[1],
            background: TEXT_NORMAL[0],
            text,
            align,
            font,
            plan: None
        }
    }

    pub fn set_colors(mut self, color: Option<u8>, background: Option<u8>) {
        if let Some(text_color) = color {
            self.color = text_color;
        }

        if let Some(bg_color) = background {
            self.background = bg_color;
        }
    }

    pub fn update(&mut self, text: &str, rq: &mut RenderQueue) {
        if self.text != text {
            self.text = text.to_string();
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
        }
    }

    pub fn get_plan(&self, fonts: &mut Fonts) -> RenderPlan {
        let dpi = CURRENT_DEVICE.dpi;
        let font = font_from_style(fonts, &self.font, dpi);
        let padding = font.em() as i32;
        let max_width = self.rect.width() as i32 - padding;
        font.plan(&self.text, Some(max_width), None)
    }
}

impl View for Text {
    fn handle_event(&mut self, evt: &Event, _hub: &Hub, bus: &mut Bus, rq: &mut RenderQueue, _context: &mut Context) -> bool {
        match *evt {
            _ => false,
        }
    }

    fn render(&self, fb: &mut dyn Framebuffer, _rect: Rectangle, fonts: &mut Fonts) {
        let dpi = CURRENT_DEVICE.dpi;

        fb.draw_rectangle(&self.rect, self.background);

        let plan = &self.get_plan(fonts);

        let font = font_from_style(fonts, &self.font, dpi);
        let x_height = font.x_heights.0 as i32;

        let dx = self.align.offset(plan.width, self.rect.width() as i32);
        let dy = (self.rect.height() as i32 - x_height) / 2;
        let pt = pt!(self.rect.min.x + dx, self.rect.max.y - dy);

        font.render(fb, self.color, &plan, pt);
    }

    fn resize(&mut self, rect: Rectangle, _hub: &Hub, _rq: &mut RenderQueue, _context: &mut Context) {
        self.rect = rect;
    }

    fn rect(&self) -> &Rectangle {
        &self.rect
    }

    fn rect_mut(&mut self) -> &mut Rectangle {
        &mut self.rect
    }

    fn children(&self) -> &Vec<Box<dyn View>> {
        &self.children
    }

    fn children_mut(&mut self) -> &mut Vec<Box<dyn View>> {
        &mut self.children
    }

    fn id(&self) -> Id {
        self.id
    }
}