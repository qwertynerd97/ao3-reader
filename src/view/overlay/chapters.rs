use crate::font::Fonts;
use crate::view::{View, Event, Hub, Bus, RenderQueue, Align, ViewId, Id, ID_FEEDER, RenderData};
use crate::view::{MINI_BAR_HEIGHT, THICKNESS_MEDIUM, SMALL_PADDING};
use crate::app::Context;
use crate::unit::scale_by_dpi;
use crate::geom::{Rectangle, CycleDir};
use crate::document::{Location, Chapter};
use crate::color::{BLACK};
use crate::device::CURRENT_DEVICE;
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::view::textlabel::TextLabel;
use crate::view::filler::Filler;
use super::Overlay;
use crate::font::{LABEL_STYLE};
use crate::helpers::ceil;

#[derive(Clone)]
pub struct Chapters {
    overlay: Overlay,
    children: Vec<Box<dyn View>>,
    id: Id,
    view_id: ViewId,
    entries: Vec<Chapter>,
    rows: usize,
    current_page: usize,
    max_pages: usize
}

pub fn row_calc(rect: Rectangle) -> usize {
    let dpi = CURRENT_DEVICE.dpi;
    let small_height = scale_by_dpi(MINI_BAR_HEIGHT, dpi) as i32;
    let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
    ((rect.height() as i32 + thickness) / small_height) as usize
}

impl Chapters {
    pub fn new(entries: Vec<Chapter>, context: &mut Context) -> Chapters {
        let id = ID_FEEDER.next();
        let mut overlay = Overlay::new(ViewId::ChapterList, context);
        let rows = row_calc(overlay.msg_rect());
        let max_pages = if entries.len() > 0 { ceil(entries.len(), rows) } else { 1 };
        overlay.set_max(max_pages);
        let children = vec![Box::new(overlay.clone()) as Box<dyn View>];
    
        Chapters {
            overlay,
            children,
            id,
            entries,
            rows,
            view_id: ViewId::ChapterList,
            current_page: 0,
            max_pages
        }
    }

    pub fn update_chapters(&mut self) {
        let rect = self.overlay.msg_rect();
        let dpi = CURRENT_DEVICE.dpi;
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let padding = scale_by_dpi(SMALL_PADDING, dpi) as i32;
        let row_height = rect.height() as i32 / self.rows as i32;
        let x_min = rect.min.x; // + padding;
        let x_max = rect.max.x; // - padding;
        let mut start_y = rect.min.y as i32;

        self.children_mut().drain(1..); // Remove old chapter items

        if self.entries.len() > 0 {
            let start = self.current_page * self.rows;
            let mut end = start + self.rows;
            end = if end < self.entries.len() { end } else { self.entries.len() };

            for n in start..end {
                let sep_rect = rect![x_min, start_y,
                x_max, start_y + thickness];
                let sep = Filler::new(sep_rect, BLACK);
                self.children_mut().push(Box::new(sep) as Box<dyn View>);
                let label_rect = rect![x_min, start_y + thickness,
                x_max, start_y + row_height];
                let loc = Location::Uri((*self.entries[n].location).to_string());

                let chapter = TextLabel::new(label_rect,
                                    (*self.entries[n].title).to_string(),
                                    Align::Left(padding), LABEL_STYLE, Event::GoToLocation(loc));
                self.children_mut().push(Box::new(chapter) as Box<dyn View>);
                start_y += row_height;
            }

            let sep_rect = rect![x_min, start_y,
            x_max, start_y + thickness];
            let sep = Filler::new(sep_rect, BLACK);
            self.children_mut().push(Box::new(sep) as Box<dyn View>);
        }
    }

}

impl View for Chapters {
    fn handle_event(&mut self, evt: &Event, hub: &Hub, bus: &mut Bus, rq: &mut RenderQueue, context: &mut Context) -> bool {
        match *evt {
            Event::Page(ref dir) => {
                match dir {
                    CycleDir::Next => if self.current_page < self.max_pages - 1 {self.current_page = self.current_page + 1 },
                    CycleDir::Previous => if self.current_page > 0 {self.current_page = self.current_page - 1}
                }
                self.update_chapters();
                rq.add(RenderData::new(self.id, *self.rect(), UpdateMode::Gui));
                true
            },
            Event::GoToLocation(..) => {
                hub.send(Event::Close(self.view_id)).ok();
                false
            },
            Event::Gesture(..) => true,
            _ => self.overlay.handle_event(evt, hub, bus, rq, context),
        }
    }

    fn render(&self, fb: &mut dyn Framebuffer, rect: Rectangle, fonts: &mut Fonts) {
        self.overlay.render(fb, rect, fonts);
    }

    fn rect(&self) -> &Rectangle {
        &self.overlay.rect()
    }

    fn rect_mut(&mut self) -> &mut Rectangle {
        self.overlay.rect_mut()
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