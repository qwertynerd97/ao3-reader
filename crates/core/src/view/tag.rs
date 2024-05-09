use chrono::offset;

use super::{Bus, Event, Hub, Id, RenderData, RenderQueue, View, ID_FEEDER};
use crate::color::{TEXT_BUMP_SMALL, TEXT_INVERTED_SOFT};
use crate::context::Context;
use crate::device::CURRENT_DEVICE;
use crate::font::{font_from_style, Fonts, RenderPlan, Style};
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::geom::{Point, Rectangle};
use crate::gesture::GestureEvent;
use crate::helpers::unicode_split;
use std::fmt;

#[derive(Clone, Debug)]
pub struct TagInfo {
    pub text: String,
    pub location: Option<String>,
    pub style: Style,
}

#[derive(Clone, Debug)]
pub struct TagElement{
    text: String,
    pt: Point
}

impl TagInfo {
    pub fn new(text: String, location: Option<String>, style: Style) -> TagInfo {
        TagInfo {
            text,
            location,
            style,
        }
    }
}

impl TagElement {
    pub fn new(text: String, pt: Point) -> TagElement {
        TagElement {
            text,
            pt
        }
    }
}

#[derive(Clone)]
pub struct Tag {
    id: Id,
    active: bool,
    rect: Rectangle,
    pub rects: Vec<Rectangle>,
    children: Vec<Box<dyn View>>,
    elements: Vec<TagElement>,
    has_loc: bool,
    pub info: TagInfo,
}

impl fmt::Debug for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tag [{}, {:?}]", self.info.text, self.info.location)
    }
}

impl Tag {
    pub fn new(
        rect: Rectangle,
        info: TagInfo,
        wrap_width: i32,
        offset: i32,
        fonts: &mut Fonts,
    ) -> Tag {
        let mut elements = Vec::new();
        let mut rects = Vec::new();
        let mut our_rect = rect;
        let dpi = CURRENT_DEVICE.dpi;

        let height = rect.height() as i32;
        let mut width = rect.width() as i32;
        let font = font_from_style(fonts, &info.style, dpi);

        let mut plan = font.plan(&info.text, None, None);
        let mut lines = 1;

        let line_height = font.line_height();
        let line_diff = (height - line_height) / 3;
        let mut start_x = rect.min.x + (line_diff * 2);
        let mut start_y = rect.min.y;
        let mut has_loc = false;
        let mut pt;

        if let Some(_loc) = info.location.clone() {
            has_loc = true;
        }

        let mut text = info.text.clone();

        loop {
            if plan.width > width {
                let (index, usable_width) = font.cut_point(&plan, width);
                let temp_plan = plan.split_off(index, usable_width);
                let (og_text, temp_text) = unicode_split(&mut text, index);
                text = og_text;
                pt = pt!(start_x, start_y + line_height + line_diff);
                elements.push(TagElement::new(text, pt));
                if lines == 1 {
                    rects.push(rect![
                        start_x - (line_diff * 2),
                        start_y + line_diff,
                        start_x + plan.width + line_diff,
                        start_y + height
                    ]);
                } else {
                    rects.push(rect![
                        start_x - line_diff,
                        start_y + line_diff,
                        start_x + plan.width + line_diff,
                        start_y + height
                    ]);
                }
                plan = temp_plan;
                text = temp_text;
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
        elements.push(TagElement::new(text, pt));

        if lines > 1 {
            our_rect = rect![
                offset,
                rect.min.y,
                offset + wrap_width,
                rect.min.y + (lines * height)
            ];
            rects.push(rect![
                start_x - (line_diff * 2),
                start_y + line_diff,
                start_x + plan.width + (line_diff * 2),
                start_y + height
            ]);
        } else {
            rects.push(rect![
                start_x - line_diff,
                start_y + line_diff,
                start_x + plan.width + (line_diff * 2),
                start_y + height
            ]);
        }

        Tag {
            id: ID_FEEDER.next(),
            active: false,
            rects,
            rect: our_rect,
            children: vec![],
            elements,
            info,
            has_loc,
        }
    }

    pub fn end_point(&self) -> Point {
        let rect = self.rects[self.rects.len() - 1].clone();
        pt![rect.max.x, rect.max.y]
    }

    pub fn lines(&self) -> usize {
        self.rects.len()
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

    pub fn vertical_shift(&mut self, index: usize, offset: i32) {
        self.elements[index].pt.y -= offset;
        self.rects[index].max.y -= offset;
        self.rects[index].min.y -= offset;
    }

    pub fn split(&mut self, height: i32, offset: i32) -> Tag {
        let mut line = 0;
        for rect in &self.rects {
            if rect.max.y > height {
                break;
            }
            line += 1;
        }

        let rects = self.rects.split_off(line);
        let elements = self.elements.split_off(line);

        let mut new_tag = Tag {
            info: self.info.clone(),
            id: ID_FEEDER.next(),
            active: self.active.clone(),
            rect: self.rect.clone(),
            rects,
            children: vec![],
            elements,
            has_loc: self.has_loc.clone(),
        };

        for i in 0..new_tag.elements.len() {
            new_tag.vertical_shift(i, offset);
        }
        return new_tag;
    }



    pub fn get_plan(&self, text: String, fonts: &mut Fonts) -> RenderPlan {
        let dpi = CURRENT_DEVICE.dpi;
        let font = font_from_style(fonts, &self.info.style, dpi);
        font.plan(text, None, None)
    }
}

impl View for Tag {
    fn handle_event(
        &mut self,
        evt: &Event,
        _hub: &Hub,
        bus: &mut Bus,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) -> bool {
        match *evt {
            Event::Gesture(GestureEvent::Tap(center)) if self.in_rects(center) => {
                rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                self.active = true;
                bus.push_back(Event::LoadIndex(self.info.location.clone().unwrap()));
                true
            }
            _ => false,
        }
    }

    fn render(&self, fb: &mut dyn Framebuffer, _rect: Rectangle, fonts: &mut Fonts) {
        let scheme = if self.active {
            TEXT_INVERTED_SOFT
        } else {
            TEXT_BUMP_SMALL
        };
        let dpi = CURRENT_DEVICE.dpi;

        if self.has_loc {
            for rect in &self.rects {
                fb.draw_rectangle(rect, scheme[0]);
            }
        }

        for el in &self.elements {
            let plan = &self.get_plan(el.text.to_string(), fonts);

            let font = font_from_style(fonts, &self.info.style, dpi);
    
            font.render(fb, scheme[1], &plan, el.pt);
        }

    }

    fn resize(
        &mut self,
        rect: Rectangle,
        _hub: &Hub,
        _rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
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
