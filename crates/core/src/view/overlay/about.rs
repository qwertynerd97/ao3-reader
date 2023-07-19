use crate::font::{Fonts, font_from_style};
use crate::view::{View, Event, Hub, Bus, RenderQueue,  ViewId, Id, ID_FEEDER, RenderData};
use crate::view::SMALL_PADDING;
use crate::context::Context;
use crate::unit::scale_by_dpi;
use crate::geom::{Rectangle, CycleDir};
use crate::device::CURRENT_DEVICE;
use crate::framebuffer::{Framebuffer, UpdateMode};
use super::Overlay;
use crate::font::{LABEL_STYLE, BOLD_STYLE, BOLD_TITLE, ABOUT_STYLE};
use crate::view::tag::Tag;
use crate::ao3_metadata::Ao3Info;
use std::{thread, time};

#[derive(Clone)]
pub struct About {
    overlay: Overlay,
    children: Vec<Box<dyn View>>,
    id: Id,
    view_id: ViewId,
    page: Vec<Tag>,
    index: Vec<usize>,
    current_page: usize,
    max_pages: usize
}

impl About {
    pub fn new(info: Ao3Info, context: &mut Context) -> About {
        let id = ID_FEEDER.next();
        let mut overlay = Overlay::new(ViewId::AboutOverlay, context);
        
        // Figure out our line heights
        let dpi = CURRENT_DEVICE.dpi;
        let rect = overlay.msg_rect();
        println!("rect is {:?}", rect);
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
            items.push((temp.title, None, LABEL_STYLE));
        }
        items.push(("Fandoms:".to_string(), None, BOLD_STYLE));
        for fandom in info.fandoms {
            let temp = fandom.clone();
            items.push((temp.title, None, ABOUT_STYLE));
        }
    
        items.push(("Tags:".to_string(), None, BOLD_STYLE));
        for tag in info.tags {
            let temp = tag.clone();
            items.push((temp.title, None, ABOUT_STYLE));
        }

        items.push(("Summary:".to_string(), None, BOLD_STYLE));
        items.push((info.summary.clone(), None, LABEL_STYLE));

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
            // println!("tag rects are {:?}", tag.rects);
            // println!("Tag link is {:?}", tag.loc);
    
            tag_list.push(tag);
        }
        println!("===================");
        // split items across pages
        let mut page = Vec::new();
        let mut index = Vec::new();
        let mut i = 0;
        index.push(i);
        let mut pg_count = 1;

        for mut tag in tag_list{
            let end_pt = tag.end_point().y;

            if end_pt > ((rect.max.y - rect.min.y) * pg_count + rect.min.y){
                if tag.lines() > 1 {
                    // we need to split a tag
                    let split = (rect.max.y - rect.min.y) * pg_count + rect.min.y;
                    // println!("Tag lines is {}, split is {}", tag.lines(), split);
                    let new_tag = tag.split(split);
                    // println!("tag rects are {:?}", tag.rects);
                    // println!("new tag rects are {:?}", new_tag.rects);

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

        println!("indexes are {:?}", index);
        // Shift all pages appropriately up.
        for (count, page_i) in index.iter().enumerate() {
            let offset = count as u32 * rect.height();
            let next = (count + 1) as usize;
            let end = if next < index.len() {index[next] as usize } else { page.len() };
            for tag in &mut page[*page_i..end] {
                tag.shift_vertical(offset as i32);
            }
        }

        let max_pages = index.len();
        overlay.set_max(max_pages);

        // let end = if 1 < max_pages {index[1] as usize} else {page.len()};
        // println!("end is {:?}", end);

        // let temp = Vec::from(&page[0..end]);
        let mut children = Vec::new();
        children.push(Box::new(overlay.clone()) as Box<dyn View>);
        // for tag in temp {
        //      //println!("tag rects are {:?}", tag.rects);
        //      println!("Tag link is {:?}", tag.text);
        //     children.push(Box::new(tag) as Box<dyn View>);
        // }
        //  println!("child len is {}", children.len());
        //  println!("page len is {}", page.len());
        //  println!("-------");

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
        println!("updating page....");
        let start = self.index[self.current_page] as usize;
        let next = self.current_page + 1;
        let end = if next < self.max_pages {self.index[next] as usize} else {self.page.len()};
        println!("start is {:?}", start);
        println!("end is {:?}", end);
        println!("next is {:?}", next);

        self.children_mut().drain(1..); // Remove old chapter items
        println!("child len is {}", self.children.len());
        println!("page len is {}", self.page.len());
        //let page_items = Vec::from_iter(self.page[start..end].iter().cloned());
        let temp = Vec::from(&self.page[start..end]);

        // println!("temp len is {}", temp.len());
        // println!("temp elements are {:?}", temp);
        
        for tag in temp {
             //println!("tag rects are {:?}", tag.rects);
             println!("Tag link is {:?}", tag.text);
            self.children_mut().push(Box::new(tag.clone()) as Box<dyn View>);
        }
         println!("child len is {}", self.children.len());
         println!("child elements are {:?}", self.children);
         println!("-------");
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
                thread::sleep(time::Duration::from_secs(1));
                rq.add(RenderData::new(self.id, *self.rect(), UpdateMode::Partial));
                true
            },
            Event::LoadIndex(..) => {
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

