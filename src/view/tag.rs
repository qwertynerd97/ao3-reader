use crate::device::CURRENT_DEVICE;
use crate::font::{Fonts, Style, font_from_style, RenderPlan};
use super::{View, Event, Hub, Bus, Id, ID_FEEDER, RenderQueue, RenderData};
use crate::gesture::GestureEvent;
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::geom::{Rectangle, Point};
use crate::document::Location;
use crate::color::{TEXT_BUMP_SMALL, TEXT_INVERTED_SOFT};
use crate::app::Context;

#[derive(Clone)]
pub struct Tag {
    id: Id,
    active: bool,
    rect: Rectangle,
    rects: Vec<Rectangle>,
    children: Vec<Box<dyn View>>,
    elements: Vec<(RenderPlan, Point)>,
    loc: Option<String>,
    style: Style,
    has_loc: bool
}

impl Tag {
    pub fn new(rect: Rectangle, text: String, wrap_width: i32, offset: i32, loc: Option<String>, fonts: &mut Fonts, style: Style) -> Tag {
        let mut elements = Vec::new();
        let mut rects = Vec::new();
        let mut our_rect = rect;
        let dpi = CURRENT_DEVICE.dpi;

        let height = rect.height() as i32;
        let mut width = rect.width() as i32;
        let font = font_from_style(fonts, &style, dpi);

        let mut plan = font.plan(&text, None, None);
        let mut lines = 1;
        let max_lines = 5;

        let line_height = font.line_height();
        let line_diff = (height - line_height) / 3;
        let mut start_x = rect.min.x + (line_diff * 2);
        let mut start_y = rect.min.y;
        let mut has_loc = false;

        if let Some(_loc) = loc.clone() {
            has_loc = true;
        }

        while lines < max_lines {
            if plan.width > width {
                let (index, usable_width) = font.cut_point(&plan, width);
                let temp_plan = plan.split_off(index, usable_width);
                let pt = pt!(start_x,
                    start_y + line_height + line_diff);
                elements.push((plan.clone(), pt));
                if lines == 1 {
                    rects.push(rect![start_x - (line_diff * 2), start_y + line_diff, start_x + plan.width + line_diff, start_y + height]);
                } else {
                    rects.push(rect![start_x - line_diff, start_y + line_diff, start_x + plan.width + line_diff, start_y + height]);
                }
                plan = temp_plan;
                if lines == 1 {
                    width = wrap_width;
                    start_x = offset + line_diff;
                }
                lines += 1;
                start_y += height;
            } else {
                break;
            }
        }
        
        // Handle the last line
        if plan.width > width {
            font.crop_right(&mut plan, width);
        }

        let pt = pt!(start_x, start_y + line_height + line_diff);
        elements.push((plan.clone(), pt));
        
        if lines > 1 {
            our_rect = rect![offset, rect.min.y, offset + wrap_width, rect.min.y + (lines * height)];
            rects.push(rect![start_x - (line_diff * 2), start_y + line_diff, start_x + plan.width + (line_diff * 2), start_y + height]);
        } else {
            rects.push(rect![start_x - line_diff, start_y + line_diff, start_x + plan.width + (line_diff * 2), start_y + height]);
        }  
        
        Tag {
            id: ID_FEEDER.next(),
            active: false,
            rects,
            rect: our_rect,
            children: vec![],
            elements,
            loc,
            style,
            has_loc
        }
    }

    pub fn end_point(&self) -> Point {
        let rect = self.rects[self.rects.len() - 1].clone();
        pt![rect.max.x, rect.max.y]
    }

    pub fn lines(&self) -> usize {
        self.rects.len()
    }

    pub fn shift_vertical(&mut self, offset: i32) {
        for rect in &mut self.rects {
            rect.min.y =- offset;
            rect.max.y =- offset;
        }

        for el in &mut self.elements {
            el.1.y =- offset;
        }

    }

    fn in_rects(&self, pt: Point) -> bool {
        if self.has_loc {
            for rect in &self.rects {
                if rect.includes(pt) {
                    return true;
                }
            }
        }
        return false;

    }

    pub fn split(&mut self, line: usize) -> Tag {
        let rects = self.rects.split_off(line);
        let elements = self.elements.split_off(line);
        Tag {
            id: ID_FEEDER.next(),
            active: self.active.clone(),
            rects,
            rect: self.rect.clone(),
            children: vec![],
            elements,
            loc: self.loc.clone(),
            style: self.style.clone(),
            has_loc: self.has_loc.clone()
        }
    }

}

impl View for Tag {
    fn handle_event(&mut self, evt: &Event, _hub: &Hub, bus: &mut Bus, rq: &mut RenderQueue, _context: &mut Context) -> bool {
        match *evt {
            Event::Gesture(GestureEvent::Tap(center)) if self.in_rects(center) => {
                rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                self.active = true;
                bus.push_back(Event::LoadIndex(self.loc.clone().unwrap()));
                true
            },
            _ => false,
        }
    }

    fn render(&self, fb: &mut dyn Framebuffer, _rect: Rectangle, fonts: &mut Fonts) {
        let scheme = if self.active {
            TEXT_INVERTED_SOFT
        } else {
            TEXT_BUMP_SMALL
        };

        if self.has_loc {
            for rect in &self.rects {
                fb.draw_rectangle(rect, scheme[0]);
            }
        }

        let dpi = CURRENT_DEVICE.dpi;
        let font = font_from_style(fonts, &self.style, dpi);
        for (plan, pt) in &self.elements {
            font.render(fb, scheme[1], &plan, *pt);
        }
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
