use super::Overlay;
use crate::ao3_metadata::Ao3Info;
use crate::context::Context;
use crate::device::CURRENT_DEVICE;
use crate::font::{font_from_style, Fonts};
use crate::font::{ABOUT_STYLE, BOLD_STYLE, BOLD_TITLE, LABEL_STYLE};
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::geom::{CycleDir, Rectangle};
use crate::unit::scale_by_dpi;
use crate::view::tag::{Tag, TagInfo};
use crate::view::SMALL_PADDING;
use crate::view::{
    Bus, Event, Hub, Id, RenderData, RenderQueue, View, ViewId, ID_FEEDER,
};

#[derive(Clone)]
pub struct About {
    overlay: Overlay,
    children: Vec<Box<dyn View>>,
    id: Id,
    view_id: ViewId,
    pages: Vec<Vec<Tag>>,
    current_page: usize,
    max_pages: usize,
}

#[derive(Clone, Debug)]
pub struct PageInfo {
    pub start: usize,
    pub end: usize,
    pub elements: Vec<Tag>,
}

impl About {
    pub fn new(info: Ao3Info, context: &mut Context) -> About {
        let id = ID_FEEDER.next();
        let mut overlay = Overlay::new(ViewId::AboutOverlay, context);

        // Figure out our line heights
        let dpi = CURRENT_DEVICE.dpi;
        let rect = overlay.msg_rect();
        let font = font_from_style(&mut context.fonts, &LABEL_STYLE, dpi);
        let font_height = font.line_height();
        let box_height = font_height * 3 / 2;
        let max_lines = (rect.height() as i32 / box_height) as usize;
        let line_height = rect.height() as i32 / max_lines as i32;

        // Generate our list of elements to iterate over
        let mut items = Vec::new();

        items.push(TagInfo::new(info.title, None, BOLD_TITLE));
        items.push(TagInfo::new("    by".to_string(), None, LABEL_STYLE));
        for author in info.authors {
            let temp = author.clone();
            items.push(TagInfo::new(temp.title, Some(temp.location), LABEL_STYLE));
        }
        items.push(TagInfo::new("Fandoms:".to_string(), None, BOLD_STYLE));
        for fandom in info.fandoms {
            let temp = fandom.clone();
            items.push(TagInfo::new(temp.title, Some(temp.location), ABOUT_STYLE));
        }

        items.push(TagInfo::new("Tags:".to_string(), None, BOLD_STYLE));
        for tag in info.tags {
            let temp = tag.clone();
            items.push(TagInfo::new(temp.title, Some(temp.location), ABOUT_STYLE));
        }

        items.push(TagInfo::new("Summary:".to_string(), None, BOLD_STYLE));
        items.push(TagInfo::new(info.summary.clone(), None, LABEL_STYLE));

        // Actually generate the items
        let padding = scale_by_dpi(SMALL_PADDING, dpi) as i32;
        let width = (rect.width() as i32) - (2 * padding);
        let height = rect.height() as i32;
        let offset = rect.min.x + padding;
        let mut start_x = rect.min.x + padding;
        let mut start_y = rect.min.y;
        let mut elements = Vec::new();
        let mut pages = Vec::new();

        for item in items {
            // tags without locations are labels, and start on a new line always.
            let label = item.location.is_none();
            if label && start_x != offset {
                start_y += line_height;
                start_x = offset;
            }

            let tag_rect = rect![start_x, start_y, width, start_y + line_height];
            let mut tag = Tag::new(tag_rect, item, width, offset, &mut context.fonts);
            let end_pt = tag.end_point();
            let lines = tag.lines();
            let rem_width = width as i32 - end_pt.x;

            if rem_width < line_height {
                start_y += lines as i32 * line_height;
                start_x = offset;
            } else {
                start_x = if label { end_pt.x } else { end_pt.x + padding };
                start_y += (lines as i32 - 1) * line_height;
            }

            // Check if the tag fits on this page. If yes, continue on
            // If not, we're done here
            let mut tag_end = tag.end_point().y.clone();
            while tag_end > rect.max.y {
                if tag.lines() > 1 {
                    // we need to split a tag
                    let new_tag = tag.split(rect.max.y, height);

                    elements.push(tag);
                    pages.push(elements);

                    // start a new page and reset line counts
                    elements = Vec::new();
                    tag = new_tag;
                    tag_end = tag.end_point().y.clone();
                    start_x = tag.end_point().x + padding;
                    start_y = rect.min.y;
                } else {
                    pages.push(elements);
                    elements = Vec::new();
                    tag.vertical_shift(0, height);
                    start_x = tag.end_point().x + padding;
                    start_y = rect.min.y;
                    break;
                }
            }
                elements.push(tag);

        }
        if elements.len() > 0 {
            pages.push(elements);
        }
        let max_pages = pages.len();
        overlay.set_max(max_pages);

        let mut children = Vec::new();
        children.push(Box::new(overlay.clone()) as Box<dyn View>);

        About {
            overlay,
            children,
            id,
            view_id: ViewId::AboutOverlay,
            pages,
            current_page: 0,
            max_pages,
        }
    }

    pub fn update_page(&mut self) {
        self.children_mut().drain(1..); // Remove old items, but not the overlay reference

        let page = self.pages[self.current_page].clone();
        for item in page {
            self.children_mut().push(Box::new(item) as Box<dyn View>);
        }
    }
}

impl View for About {
    fn handle_event(
        &mut self,
        evt: &Event,
        hub: &Hub,
        bus: &mut Bus,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match *evt {
            Event::Page(ref dir) => {
                match dir {
                    CycleDir::Next => {
                        if self.current_page < self.max_pages - 1 {
                            self.current_page = self.current_page + 1
                        }
                    }
                    CycleDir::Previous => {
                        if self.current_page > 0 {
                            self.current_page = self.current_page - 1
                        }
                    }
                }
                self.update_page();
                rq.add(RenderData::new(self.id, *self.rect(), UpdateMode::Gui));
                true
            }
            Event::LoadIndex(..) => {
                hub.send(Event::Close(self.view_id)).ok();
                false
            }
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
