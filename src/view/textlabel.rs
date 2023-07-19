use crate::device::CURRENT_DEVICE;
use crate::font::{Fonts, font_from_style, NORMAL_STYLE, Style, RenderPlan};
use super::{View, Event, Hub, Bus, Id, ID_FEEDER, RenderQueue, RenderData, Align};
use crate::gesture::GestureEvent;
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::geom::Rectangle;
use crate::color::{TEXT_NORMAL, TEXT_INVERTED_HARD};
use crate::app::Context;
use crate::input::{DeviceEvent, FingerStatus};

#[derive(Clone)]
pub struct TextLabel {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    background: u8,
    text: String,
    align: Align,
    font: Style,
    event: Event,
    hold_event: Option<Event>,
    pub active: bool,
    plan: Option<RenderPlan>
}

impl TextLabel {
    pub fn new(rect: Rectangle, text: String, align: Align, font: Style, event: Event) -> TextLabel {
        TextLabel {
            id: ID_FEEDER.next(),
            rect,
            children: Vec::new(),
            background: TEXT_NORMAL[0],
            text,
            align,
            font,
            event,
            hold_event: None,
            active: false,
            plan: None
        }
    }


    pub fn hold_event(mut self, event: Option<Event>) -> TextLabel {
        self.hold_event = event;
        self
    }

   pub fn background(mut self, background: u8) -> TextLabel {
        self.background = background;
        self
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

impl View for TextLabel {
    fn handle_event(&mut self, evt: &Event, _hub: &Hub, bus: &mut Bus, rq: &mut RenderQueue, _context: &mut Context) -> bool {
        match *evt {
                Event::Device(DeviceEvent::Finger { status, position, .. }) => {
                match status {
                    FingerStatus::Down if self.rect.includes(position) => {
                        self.active = true;
                        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Fast));
                        true
                    },
                    FingerStatus::Up if self.active => {
                        self.active = false;
                        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                        true
                    },
                    _ => false,
                }
            },
            Event::Gesture(GestureEvent::Tap(center)) if self.rect.includes(center) => {
                bus.push_back(self.event.clone());
                true
            },
            Event::Gesture(GestureEvent::HoldFingerShort(center, _)) if self.rect.includes(center) => {
                if let Some(event) = self.hold_event.clone() {
                    bus.push_back(event);
                }
                true
            },
            _ => false,
        }
    }

    fn render(&self, fb: &mut dyn Framebuffer, _rect: Rectangle, fonts: &mut Fonts) {
        let scheme = if self.active {
            TEXT_INVERTED_HARD
        } else {
            TEXT_NORMAL
        };
        let dpi = CURRENT_DEVICE.dpi;

        fb.draw_rectangle(&self.rect, scheme[0]);

        let plan = &self.get_plan(fonts);

        let font = font_from_style(fonts, &NORMAL_STYLE, dpi);
        let x_height = font.x_heights.0 as i32;

        let dx = self.align.offset(plan.width, self.rect.width() as i32);
        let dy = (self.rect.height() as i32 - x_height) / 2;
        let pt = pt!(self.rect.min.x + dx, self.rect.max.y - dy);
        println!("rendering label {:?}", self.text);
        font.render(fb, scheme[1], &plan, pt);
    }

    fn resize(&mut self, rect: Rectangle, _hub: &Hub, _rq: &mut RenderQueue, _context: &mut Context) {
        if let Event::ToggleNear(_, ref mut event_rect) = self.event {
            *event_rect = rect;
        }
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