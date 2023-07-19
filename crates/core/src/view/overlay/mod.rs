pub mod chapters;
pub mod works;
pub mod about;

use std::thread;
use crate::device::CURRENT_DEVICE;
use crate::geom::{ Rectangle, CornerSpec, BorderSpec, CycleDir};
use crate::font::Fonts;
use super::{View, Event, Hub, Bus, Id, ID_FEEDER, RenderQueue, ViewId, RenderData};
use super::{THICKNESS_LARGE, BORDER_RADIUS_MEDIUM, CLOSE_IGNITION_DELAY, SMALL_BAR_HEIGHT, BIG_BAR_HEIGHT};
use super::icon::{Icon, DisabledIcon};
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::gesture::GestureEvent;
use crate::color::{BLACK, WHITE};
use crate::unit::scale_by_dpi;
use crate::app::Context;

#[derive(Clone)]
pub struct Overlay {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    msg_area: Rectangle,
    view_id: ViewId,
    will_close: bool,
    current_page: usize,
    pages_count: usize,
    is_next_disabled: bool,
    is_prev_disabled: bool,
}

impl Overlay {
    pub fn new(view_id: ViewId, context: &mut Context) -> Overlay {
        let id = ID_FEEDER.next();
        let dpi = CURRENT_DEVICE.dpi;
        let (width, height) = context.display.dims;
        let mut children = Vec::new();
        let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;
        let padding = scale_by_dpi(BORDER_RADIUS_MEDIUM, dpi) as i32;

        let overlay_width = width as i32 - 4 * padding;
        let overlay_height = height as i32 - 3 * small_height - 2 * padding;
        let max_message_width = overlay_width - 2 * padding;
        let max_message_height = overlay_height - 2 * small_height;

        let pages_count = 0;
        let is_next_disabled = pages_count < 2;

        let dx = (width as i32 - overlay_width) / 2;
        let dy = small_height + padding;
        let rect = rect![dx, dy,
                         dx + overlay_width, dy + overlay_height];

        let rect_close = rect![rect.max.x - small_height - padding,
                                  rect.min.y + padding,
                                  rect.max.x - padding,
                                  rect.min.y + padding + small_height];
        let button_close = Icon::new("close", rect_close, Event::Validate);
        children.push(Box::new(button_close) as Box<dyn View>);

        let mid = width as i32 / 2;
        let rect_prev = rect![mid - small_height,
                                  rect.max.y - padding - small_height,
                                  mid,
                                  rect.max.y - padding];

        let rect_next = rect![mid,
                                  rect.max.y - padding - small_height,
                                  mid + small_height,
                                  rect.max.y - padding];

        let prev_btn = DisabledIcon::new("angle-up-grey", rect_prev);
        children.push(Box::new(prev_btn) as Box<dyn View>);
        if is_next_disabled {
            let next_btn = DisabledIcon::new("angle-down-grey", rect_next);
            children.push(Box::new(next_btn) as Box<dyn View>);

        } else {
            let next_btn = Icon::new("angle-down", rect_next, Event::Page(CycleDir::Next));
            children.push(Box::new(next_btn) as Box<dyn View>);
        }

        let msg_rect = rect![dx + padding,
                             dy + small_height,
                             dx + max_message_width + padding,
                             dy + max_message_height + small_height];

       
        Overlay {
            id,
            rect,
            children,
            msg_area: msg_rect,
            view_id,
            will_close: false,
            current_page: 0,
            pages_count,
            is_next_disabled,
            is_prev_disabled: true,
        }
    }

    fn update_buttons(&mut self, rq: &mut RenderQueue) {
        let is_prev_disabled = self.pages_count < 2 || self.current_page == 0;
        if self.is_prev_disabled != is_prev_disabled {
            let rect_prev = *self.child(1).rect();
            if is_prev_disabled {
                let prev = DisabledIcon::new("angle-up-grey", rect_prev);
                self.children[1] = Box::new(prev) as Box<dyn View>;
            } else {
                let prev = Icon::new("angle-up", rect_prev, Event::Page(CycleDir::Previous));
                self.children[1] = Box::new(prev) as Box<dyn View>;
            }

            self.is_prev_disabled = is_prev_disabled;
            rq.add(RenderData::new(self.id, rect_prev, UpdateMode::Gui));
        }

        let is_next_disabled = self.pages_count < 2 || self.current_page == self.pages_count - 1;
        if self.is_next_disabled != is_next_disabled {
            let rect_next = *self.child(2).rect();
            if is_next_disabled {
                let next = DisabledIcon::new("angle-down-grey", rect_next);
                self.children[2] = Box::new(next) as Box<dyn View>;
            } else {
                let next = Icon::new("angle-down", rect_next, Event::Page(CycleDir::Next));
                self.children[2] = Box::new(next) as Box<dyn View>;
            }

            self.is_next_disabled = is_next_disabled;
            rq.add(RenderData::new(self.id, rect_next, UpdateMode::Gui));
        }
    }

    pub fn set_max(&mut self, max: usize) {
        self.pages_count = max;
        let is_prev_disabled = self.pages_count < 2 || self.current_page == 0;
        if self.is_prev_disabled != is_prev_disabled {
            let rect_prev = *self.child(1).rect();
            if is_prev_disabled {
                let prev = DisabledIcon::new("angle-up-grey", rect_prev);
                self.children[1] = Box::new(prev) as Box<dyn View>;
            } else {
                let prev = Icon::new("angle-up", rect_prev, Event::Page(CycleDir::Previous));
                self.children[1] = Box::new(prev) as Box<dyn View>;
            }

            self.is_prev_disabled = is_prev_disabled;
        }

        let is_next_disabled = self.pages_count < 2 || self.current_page == self.pages_count - 1;
        if self.is_next_disabled != is_next_disabled {
            let rect_next = *self.child(2).rect();
            if is_next_disabled {
                let next = DisabledIcon::new("angle-down-grey", rect_next);
                self.children[2] = Box::new(next) as Box<dyn View>;
            } else {
                let next = Icon::new("angle-down", rect_next, Event::Page(CycleDir::Next));
                self.children[2] = Box::new(next) as Box<dyn View>;
            }

            self.is_next_disabled = is_next_disabled;
        }

    }

    pub fn msg_rect(&self) -> Rectangle {
        self.msg_area
    }

}

impl View for Overlay {
    fn handle_event(&mut self, evt: &Event, hub: &Hub, _bus: &mut Bus, rq: &mut RenderQueue, _context: &mut Context) -> bool {
        match *evt {
            Event::Validate | Event::Cancel => {
                if self.will_close {
                    return true;
                }
                let hub2 = hub.clone();
                let view_id = self.view_id;
                thread::spawn(move || {
                    thread::sleep(CLOSE_IGNITION_DELAY);
                    hub2.send(Event::Close(view_id)).ok();
                });
                self.will_close = true;
                true
            },
            Event::Gesture(GestureEvent::Tap(center)) if !self.rect.includes(center) => {
                hub.send(Event::Close(self.view_id)).ok();
                true
            },
            Event::Page(ref dir) => {
                match dir {
                    CycleDir::Next => if self.current_page < self.pages_count - 1 {self.current_page = self.current_page + 1 },
                    CycleDir::Previous => if self.current_page > 0 {self.current_page = self.current_page - 1}
                }
                self.update_buttons(rq);
                false
            },
            Event::Gesture(..) => true,
            _ => false,
        }
    }

    fn render(&self, fb: &mut dyn Framebuffer, _rect: Rectangle, _fonts: &mut Fonts) {
        let dpi = CURRENT_DEVICE.dpi;

        let border_radius = scale_by_dpi(BORDER_RADIUS_MEDIUM, dpi) as i32;
        let border_thickness = scale_by_dpi(THICKNESS_LARGE, dpi) as u16;

        fb.draw_rounded_rectangle_with_border(&self.rect,
                                              &CornerSpec::Uniform(border_radius),
                                              &BorderSpec { thickness: border_thickness,
                                                            color: BLACK },
                                              &WHITE);
    }

    fn resize(&mut self, _rect: Rectangle, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        let dpi = CURRENT_DEVICE.dpi;
        let (width, _height) = context.display.dims;
        let dialog_width = self.rect.width() as i32;
        let dialog_height = self.rect.height() as i32;
        let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;
        let padding = scale_by_dpi(BORDER_RADIUS_MEDIUM, dpi) as i32;

        let dx = (width as i32 - dialog_width) / 2;
        let dy = small_height + padding;
        let rect = rect![dx, dy,
                         dx + dialog_width, dy + dialog_height];

        let close_rect = rect![rect.max.x - small_height - padding,
                                rect.min.y + padding,
                                rect.max.x - padding,
                                rect.min.y + padding + small_height];
        self.children[0].resize(close_rect, hub, rq, context);

        let mid = width as i32 / 2;
        let rect_prev = rect![mid - small_height,
                                  rect.max.y - padding - small_height,
                                  mid,
                                  rect.max.y - padding];

        let rect_next = rect![mid,
                                  rect.max.y - padding - small_height,
                                  mid + small_height,
                                  rect.max.y - padding];
        self.children[1].resize(rect_prev, hub, rq, context);
        self.children[2].resize(rect_next, hub, rq, context);
        self.rect = rect;
    }

    fn is_background(&self) -> bool {
        true
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

    fn view_id(&self) -> Option<ViewId> {
        Some(self.view_id)
    }

}
