use crate::device::CURRENT_DEVICE;
use crate::font::{Fonts, font_from_style, NORMAL_STYLE};
use crate::color::{BLACK, WHITE};
use crate::geom::{Rectangle};
use crate::framebuffer::{Framebuffer, UpdateMode};
use super::{View, Event, Hub, Bus, Id, ID_FEEDER, RenderQueue, RenderData};
use crate::app::Context;

#[derive(Clone)]
pub struct WorksLabel {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    current_page: usize,
    works_count: usize,
    maxlines: usize,
}

impl WorksLabel {
    pub fn new(rect: Rectangle, current_page: usize, works_count: usize, maxlines: usize)  -> WorksLabel {
        WorksLabel {
            id: ID_FEEDER.next(),
            rect,
            children: vec![],
            current_page,
            works_count,
            maxlines,
        }
    }

    pub fn update(&mut self, current_page: usize, works_count: usize, maxlines: usize, rq: &mut RenderQueue) {
        let mut render = false;
        if self.current_page != current_page {
            self.current_page = current_page;
            render = true;
        }
        if self.works_count != works_count {
            self.works_count = works_count;
            render = true;
        }
        if self.maxlines != maxlines {
            self.maxlines = maxlines;
            render = true;
        }
        if render {
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
        }
    }

    pub fn text(&self, size: u8) -> String {
        if self.works_count == 0 {
            return "No works".to_string();
        }
        let start = self.maxlines * self.current_page;
        let end = if start + self.maxlines > self.works_count {self.works_count} else {start + self.maxlines};

        match size {
            0 => format!("{} - {} of {} works", start, end, self.works_count),
            1 => format!("{} - {} of {}", start, end, self.works_count),
            _ => format!("{} - {}", start, end)
        }
    }
}


impl View for WorksLabel {
    fn handle_event(&mut self, evt: &Event, _hub: &Hub, _bus: &mut Bus, _rq: &mut RenderQueue, _context: &mut Context) -> bool {
        match *evt {
            _ => false,
        }
    }

    fn render(&self, fb: &mut dyn Framebuffer, _rect: Rectangle, fonts: &mut Fonts) {
        let dpi = CURRENT_DEVICE.dpi;
        let font = font_from_style(fonts, &NORMAL_STYLE, dpi);
        let padding = font.em() as i32 / 2;
        let max_width = self.rect.width().saturating_sub(2 * padding as u32) as i32;
        let mut plan = font.plan(&self.text(0), None, None);
        for size in 1..=3 {
            if plan.width <= max_width {
                break;
            }
            plan = font.plan(&self.text(size), None, None);
        }
        font.crop_right(&mut plan, max_width);
        let dx = padding + (max_width - plan.width) / 2;
        let dy = (self.rect.height() as i32 - font.x_heights.0 as i32) / 2;
        let pt = pt!(self.rect.min.x + dx, self.rect.max.y - dy);
        fb.draw_rectangle(&self.rect, WHITE);
        font.render(fb, BLACK, &plan, pt);
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
