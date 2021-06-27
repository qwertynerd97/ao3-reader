use crate::font::{Fonts, Style, font_from_style};
use crate::view::{View, Event, Hub, Bus, RenderQueue,  ViewId, Id, ID_FEEDER, RenderData};
use crate::view::{ BORDER_RADIUS_MEDIUM, SMALL_PADDING};
use crate::app::Context;
use crate::unit::scale_by_dpi;
use crate::geom::{Rectangle, CycleDir};
use crate::document::{Location, Chapter};
use crate::device::CURRENT_DEVICE;
use crate::framebuffer::{Framebuffer, UpdateMode};
use super::Overlay;
use crate::font::{LABEL_STYLE, BOLD_STYLE, BOLD_TITLE};
use crate::view::tag::Tag;
use crate::ao3_metadata::Ao3Info;
use crate::helpers::ceil;

#[derive(Clone)]
pub struct About {
    overlay: Overlay,
    children: Vec<Box<dyn View>>,
    id: Id,
    view_id: ViewId,
    page: Vec<Tag>,
    index: Vec<u32>,
    current_page: usize,
    max_pages: usize
}

// pub struct AboutPage {
//     labels: Vec<Vec<(RenderPlan, Point)>,
//     children: Vec<Tag>
// }

// pub fn gen_section(start_pt: Point, width: i32, offset: i32, fonts: &mut Fonts, items: mut Vec<AboutItem>, line_height: i32) -> Vec<Tag> {
//     let mut start_x = start_pt.x;
//     let mut start_y = start_pt.y;
//     let mut tag_list = Vec::new();

//     for item in items {
//         if item.loc.is_none() && start_x != 0 {
//             // tags without locations are labels, and start on a new line always.
//             start_y += line_height;
//             start_x = 0;
//         }
//         let tag_rect = rect![start_x, start_y, width, start_y + line_height];
//         let tag = Tag::new(tag_rect, item.title, width, offset, item.loc, fonts, item.style);
//         let end_pt = tag.end_point();
//         let lines = tag.lines();
//         let rem_width = width as i32 - end_pt.x;

//         if rem_width < line_height {
//             start_y += lines as i32 * line_height;
//             start_x = 0;
//         } else {
//             start_x = end_pt.x + padding;
//             start_y += (lines as i32 - 1 ) * line_height;
//         }

//         tag_list.push(tag);
//     }

//     tag_list
// }

// pub fn gen_pages(rect: Rectangle, info: Ao3Info, fonts: &mut Fonts) -> Vec<AboutPage> {
//     let dpi = CURRENT_DEVICE.dpi;
//     let header_font = font_from_style(fonts, &MD_TITLE, dpi);
//     let font = font_from_style(fonts, &LABEL_STYLE, dpi);
//     let font_height = font.line_height();
//     let box_height = font_height * 1.5;
//     let max_lines = rect.height() / box_height;
//     let line_height = rect.height() / max_lines;

//     let mut items = Vec::new();

//     items.push(AboutItem::new(info.title, None, MD_TITLE));
//     items.push(AboutItem("by ".to_string(), None, LABEL_STYLE));
//     for author in info.authors {
//         let temp = author.clone();
//         items.push(AboutItem::new(temp.title, Some(Location::Uri(temp.location)), LABEL_STYLE));
//     }
//     items.push(AboutItem("Fandoms:".to_string(), None, BOLD_STYLE));
//     for fandom in info.fandoms {
//         let temp = fandom.clone();
//         items.push(AboutItem::new(temp.title, Some(Location::Uri(temp.location)), LABEL_STYLE));
//     }

//     items.push(AboutItem("Tags:".to_string(), None, BOLD_STYLE));
//     for tag in info.tags {
//         let temp = tag.clone();
//         items.push(AboutItem::new(temp.title, Some(Location::Uri(temp.location)), LABEL_STYLE));
//     }

//     let mut pages_lines = 0;
//     while pages_lines <= max_lines {

//     }

// }

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

        items.push((info.title, None, BOLD_TITLE));
        items.push(("    by".to_string(), None, LABEL_STYLE));
        for author in info.authors {
            let temp = author.clone();
            items.push((temp.title, Some(temp.location), LABEL_STYLE));
        }
        items.push(("Fandoms:".to_string(), None, BOLD_STYLE));
        for fandom in info.fandoms {
            let temp = fandom.clone();
            items.push((temp.title, Some(temp.location), LABEL_STYLE));
        }
    
        items.push(("Tags:".to_string(), None, BOLD_STYLE));
        for tag in info.tags {
            let temp = tag.clone();
            items.push((temp.title, Some(temp.location), LABEL_STYLE));
        }

        // Actually generate the items
        let padding = scale_by_dpi(SMALL_PADDING, dpi) as i32;
        let width = (rect.width() as i32) - (2 * padding);
        let offset = rect.min.x + padding;
        let mut start_x = rect.min.x + padding;
        let mut start_y = rect.min.y;
        let mut tag_list = Vec::new();

        for item in items {
            // tags without locations are labels, and start on a new line always.
            let label = item.1.is_none();
            if label && start_x != offset {
                start_y += line_height;
                start_x = offset;
            }
            let tag_rect = rect![start_x, start_y, offset + width, start_y + line_height];
            let tag = Tag::new(tag_rect, item.0, width, offset, item.1, &mut context.fonts, item.2);
            let end_pt = tag.end_point();
            let lines = tag.lines();
            let rem_width = width as i32 - end_pt.x;
    
            if rem_width < line_height {
                start_y += lines as i32 * line_height;
                start_x = offset;
            } else {
                start_x = if label {end_pt.x} else {end_pt.x + padding};
                start_y += (lines as i32 - 1 ) * line_height;
            }
    
            tag_list.push(tag);
        }

        // split items across pages
        let mut page = Vec::new();
        let mut index = Vec::new();
        let mut i = 0;
        index.push(i);
        let mut pg_count = 1;

        for mut tag in tag_list{
            let end_pt = tag.end_point().y;

            if end_pt > (rect.max.y * pg_count){
                if tag.lines() > 1 {
                    // we need to split a tag
                    let split = ceil((end_pt - rect.max.y) as usize, line_height as usize);
                    let new_tag = tag.split(split);
                    page.push(tag); 
                    i += 1;
                    index.push(i);
                    
                    // start a new page and reset line counts
                    page.push(new_tag);
                } else {
                    index.push(i);
                    page.push(tag);
                }
                pg_count += 1;
                
            } else {
                page.push(tag);
            }
            i += 1;
        }

        // Shift all pages appropriately up.
        // for (page_i, count) in index.iter().enumerate() {
        //     let offset = count * rect.height();
        //     let next = (count + 1) as usize;
        //     let end = if next < index.len() {index[next] as usize } else { page.len() };
        //     for tag in &page[page_i..end] {
        //         tag.shift_vertical(offset as i32);
        //     }
        // }

        let max_pages = index.len();
        overlay.set_max(max_pages);

        let end = if 1 < max_pages {index[1] as usize} else {page.len()};

        let temp = Vec::from(&page[0..end]);
        let mut children = Vec::new();
        children.push(Box::new(overlay.clone()) as Box<dyn View>);
        for tag in temp {
            children.push(Box::new(tag) as Box<dyn View>);
        }


        About {
            overlay,
            children,
            id,
            view_id: ViewId::AboutOverlay,
            page,
            index,
            current_page: 0,
            max_pages
        }
    }

    pub fn update_page(&mut self) {
        let start = self.index[self.current_page] as usize;
        let next = self.current_page + 1;
        let end = if next < self.max_pages {self.index[next] as usize} else {self.page.len()};

        self.children_mut().drain(1..); // Remove old chapter items
        //let page_items = Vec::from_iter(self.page[start..end].iter().cloned());
        let temp = Vec::from(&self.page[start..end]);

        for tag in temp {
            self.children_mut().push(Box::new(tag) as Box<dyn View>);
        }
    }
}

impl View for About {
    fn handle_event(&mut self, evt: &Event, hub: &Hub, bus: &mut Bus, rq: &mut RenderQueue, context: &mut Context) -> bool {
        match *evt {
            Event::Page(ref dir) => {
                match dir {
                    CycleDir::Next => if self.current_page < self.max_pages - 1 {self.current_page = self.current_page + 1 },
                    CycleDir::Previous => if self.current_page > 0 {self.current_page = self.current_page - 1}
                }
                self.update_page();
                rq.add(RenderData::new(self.id, *self.rect(), UpdateMode::Gui));
                true
            },
            Event::GoToTag(..) => {
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

