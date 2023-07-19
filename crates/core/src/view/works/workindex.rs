use crate::device::CURRENT_DEVICE;
use crate::gesture::GestureEvent;
use crate::font::Fonts;
use rand_core::RngCore;
use crate::color::{WHITE, SEPARATOR_NORMAL};
use crate::geom::{Rectangle, CycleDir, Dir, halves, divide};
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::view::{View, Event, Hub, Bus, Id, ID_FEEDER, RenderQueue, RenderData, ViewId};
use crate::view::{THICKNESS_MEDIUM, BIG_BAR_HEIGHT, SMALL_BAR_HEIGHT};
use crate::input::{DeviceEvent, ButtonCode, ButtonStatus};
use crate::unit::scale_by_dpi;
use crate::context::Context;
use super::work::{Work, WorkView};
use crate::view::filler::Filler;
use crate::http::{scrape_many, scrape, scrape_many_outer};
use crate::ao3_metadata::str_to_usize;
use regex::Regex;
use crate::helpers::{ceil, get_url, update_url};
use fxhash::FxHashMap;
use url::Url;
use super::bottom_bar::BottomBar;
use super::title_bar::TitleBar;

#[derive(Clone)]
pub enum PageStatus {
    Clean,
    Dirty
}

#[derive(Clone)]
pub struct WorkIndex {
    id: Id,
    pub rect: Rectangle,
    works_rect: Rectangle,
    children: Vec<Box<dyn View>>,
    pages: FxHashMap<usize, IndexPage>,
    pub max_lines: usize,
    pub work_display: WorkView,
    thumbnail_previews: bool,
    pub current_page: usize,
    pub max_page: usize,
    internal_page: usize,
    internal_max: usize,
    pub max_works: usize,
    pub url: Url,
    pub title: String,
}

#[derive(Clone)]
pub struct IndexPage {
    pub works: Vec<String>,
    pub status: PageStatus
}

pub fn fetch_index(url: &Url, context: &Context) -> (IndexPage, usize, usize, String) {
    let data = context.client.get_parse(url.as_str());
    let works = scrape_many_outer(&data, "li.work");
    let max_works_data = scrape(&data, "h2.heading");
    let title = scrape(&data, "h2.heading a.tag");
    let max_page_data = scrape_many(&data, ".pagination li a");
    let max_page_text = &max_page_data[max_page_data.len() - 2];
    let max_page = str_to_usize(max_page_text.to_string());

    let max_works_re = Regex::new(r"\d+ - \d+ of (\d+) Works").unwrap();
    let mut max_works = 0;
    if let Some(caps) = max_works_re.captures(&max_works_data) {
        max_works = str_to_usize(caps[1].to_string());
    }

    (IndexPage { works, status: PageStatus::Clean }, max_page, max_works, title)

}

impl WorkIndex {
    pub fn new(rect: Rectangle, thumbnail_previews: bool, source_url: String, hub: &Hub, context: &Context) -> WorkIndex {
        let dpi = CURRENT_DEVICE.dpi;
        let big_height = scale_by_dpi(BIG_BAR_HEIGHT, dpi) as i32;
        let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let (small_thickness, _big) = halves(thickness);

        let work_display = context.settings.ao3.work_display.clone();
        let height = rect.height() as i32 - (2 * small_height + 2 * small_thickness);
        let work_height = match work_display {
            WorkView::Short => big_height,
            WorkView::Long => 2 * big_height
        };

        let max_lines = (height / work_height) as usize;
        let url = get_url(&source_url);
        let works_rect = rect![rect.min.x, rect.min.y + small_height + small_thickness, rect.max.x, rect.max.y - small_height - small_thickness];

        
        let (index_data, internal_max, max_works, title) = fetch_index(&url, context);
        let max_page = if max_works > 0 { ceil(max_works, max_lines) } else {1};
        let mut pages = FxHashMap::default();
        pages.insert(1, index_data);

        let mut children = Vec::new();
        // Title bar
        let title_rect = rect![rect.min.x, rect.min.y,
                                rect.max.x, rect.min.y + small_height + small_thickness];
        let title_bar = TitleBar::new(title_rect, title.clone(), url.clone(), context);
        children.push(Box::new(title_bar) as Box<dyn View>);

        // Bottom bar
        let bottom_bar = BottomBar::new(rect![rect.min.x, rect.max.y - small_height - small_thickness,
            rect.max.x, rect.max.y], 0, max_page, max_works, max_lines);
        children.push(Box::new(bottom_bar) as Box<dyn View>);

        hub.send(Event::Update(UpdateMode::Partial)).ok();
        
        WorkIndex {
            id: ID_FEEDER.next(),
            rect,
            works_rect,
            children,
            work_display,
            max_lines,
            thumbnail_previews,
            current_page: 0,
            max_page,
            internal_page: 1,
            internal_max,
            url,
            max_works,
            pages,
            title
        }
    }

    pub fn set_thumbnail_previews(&mut self, thumbnail_previews: bool) {
        self.thumbnail_previews = thumbnail_previews;
    }

    pub fn get_works(&mut self, context: &Context, rq: &mut RenderQueue) {
        let start = self.max_lines * self.current_page;
        let end = start + self.max_lines;
    
        let start_page = (start / 20) + 1; // remote pages aren't 0-indexed
        let offset = start % 20;
        let offset_end = offset + self.max_lines;
        let mut end_page = if offset + self.max_lines > 20 { (end / 20) + 1 } else { start_page };  
        if end_page > self.internal_max { end_page = self.internal_max };

        let works = if start_page == end_page {
            let page = self.get_page(start_page, context);
            page[offset..offset_end].to_vec()
        } else {
            let mut temp = Vec::new();

            for i in start_page..(end_page + 1) {
                let page = self.get_page(i, context);
                if i == start_page {
                    temp.append(&mut page[offset..].to_vec());
                } else {
                    temp.append(&mut page[..].to_vec());
                }
            }

          temp[..self.max_lines].to_vec()
        };
        self.update(&works, rq);
    }

    pub fn get_page(&mut self, page: usize, context: &Context) -> Vec<String> {
        let index_page = self.pages.get(&page);
        
        let this_page = match index_page {
            Some(page_data) => {
                match page_data.status {
                    PageStatus::Clean => {page_data.clone()},
                    PageStatus::Dirty => {
                        update_url(&mut self.url, vec![("page", &page.to_string())]);
                        let (index, max_pages, max_works, _title) = fetch_index(&self.url, context);
                        if self.max_works != max_works { 
                            self.mark_dirty(page);
                            self.internal_max = max_pages;
                            self.max_works = max_works;
                        };
                        self.pages.insert(page, index.clone());
                        index
                    }
                }
            },
            None => {
                update_url(&mut self.url, vec![("page", &page.to_string())]);
                let (index, max_pages, max_works, _title) = fetch_index(&self.url, context);
                if self.max_works != max_works { 
                    self.mark_dirty(page);
                    self.internal_max = max_pages;
                    self.max_works = max_works;
                };
                self.pages.insert(page, index.clone());
                index
            }

        };

        this_page.works
    }

    pub fn mark_dirty(&mut self, ignore: usize) {
        for (index, page) in self.pages.iter_mut() {
            page.status = if *index == ignore { PageStatus::Clean } else { PageStatus::Dirty };
        }
    }

    pub fn update(&mut self, metadata: &Vec<String>, rq: &mut RenderQueue) {
        self.children.drain(2..);
        let dpi = CURRENT_DEVICE.dpi;
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let (small_thickness, big_thickness) = halves(thickness);
        let book_heights = divide(self.works_rect.height() as i32, self.max_lines as i32);
        let mut y_pos = self.works_rect.min.y;

        for (index, info) in metadata.iter().enumerate() {
            let y_min = y_pos + if index > 0 { big_thickness } else { 0 };
            let y_max = y_pos + book_heights[index] - if index < self.max_lines - 1 { small_thickness } else { 0 };

            let work = Work::new(rect![self.works_rect.min.x, y_min,
                                       self.works_rect.max.x, y_max],
                                 info.clone(),
                                 index,
                                self.thumbnail_previews,
                                self.work_display.clone());
            self.children.push(Box::new(work) as Box<dyn View>);

            if index < self.max_lines - 1 {
                let separator = Filler::new(rect![self.works_rect.min.x, y_max,
                                                  self.works_rect.max.x, y_max + thickness],
                                            SEPARATOR_NORMAL);
                self.children.push(Box::new(separator) as Box<dyn View>);
            }

            y_pos += book_heights[index];
        }

        if metadata.len() < self.max_lines {
            let y_start = y_pos + if metadata.is_empty() { 0 } else { thickness };
            let filler = Filler::new(rect![self.works_rect.min.x, y_start,
                                           self.works_rect.max.x, self.works_rect.max.y],
                                     WHITE);
            self.children.push(Box::new(filler) as Box<dyn View>);
        }

        self.update_bottom_bar(rq);
        rq.add(RenderData::new(self.id, self.works_rect, UpdateMode::Full));
    }

    pub fn set_page(&mut self, page: usize) {
        self.current_page = page;
    }

    pub fn go_to_page(&mut self, index: usize, _hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        println!("trying to go to page  {}", index);
        if index >= self.max_page {
            return;
        }
        self.current_page = index;
        self.get_works(context, rq);
        self.update_bottom_bar(rq);
    }

    pub fn update_bottom_bar(&mut self, rq: &mut RenderQueue) {
        let bottom_bar = self.children[1].as_mut().downcast_mut::<BottomBar>().unwrap();
        bottom_bar.update_works_label(self.current_page, self.max_works, self.max_lines, rq);
        bottom_bar.update_page_label(self.current_page, self.max_page, rq);
        bottom_bar.update_icons(self.current_page, self.max_page, rq);
    }

    fn go_to_neighbor(&mut self, dir: CycleDir, _hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        match dir {
            CycleDir::Next if self.current_page < self.max_page.saturating_sub(1) => {
                self.current_page += 1;
            },
            CycleDir::Previous if self.current_page > 0 => {
                self.current_page -= 1;
            },
            _ => return,
        }

        self.get_works(context, rq);
        self.update_bottom_bar(rq);
    }
}

impl View for WorkIndex {
    fn handle_event(&mut self, evt: &Event, hub: &Hub, bus: &mut Bus, rq: &mut RenderQueue, context: &mut Context) -> bool {
        match *evt {
            Event::Submit(ViewId::GoToPageInput, ref text) => {
                println!("Go to page {}", text);
                if text == "(" {
                    self.go_to_page(0, hub, rq, context);
                } else if text == ")" {
                    self.go_to_page(self.max_page.saturating_sub(1), hub, rq, context);
                } else if text == "_" {
                    let index = (context.rng.next_u64() % self.max_page as u64) as usize;
                    self.go_to_page(index, hub, rq, context);
                } else if let Ok(index) = text.parse::<usize>() {
                    self.go_to_page(index.saturating_sub(1), hub, rq, context);
                }
                true
            },
            Event::Gesture(GestureEvent::Swipe { dir, start, .. }) if self.rect.includes(start) => {
                match dir {
                    Dir::West => {
                        bus.push_back(Event::Page(CycleDir::Next));
                        true
                    },
                    Dir::East => {
                        bus.push_back(Event::Page(CycleDir::Previous));
                        true
                    },
                    _ => false,
                }
            },
            Event::Page(dir) => {
                self.go_to_neighbor(dir, hub, rq, context);
                true
            },
            Event::GoTo(location) => {
                self.go_to_page(location as usize, hub, rq, context);
                true
            },
            Event::Device(DeviceEvent::Button { code: ButtonCode::Backward, status: ButtonStatus::Pressed, .. }) => {
                self.go_to_neighbor(CycleDir::Previous, hub, rq, context);
                true
            },
            Event::Device(DeviceEvent::Button { code: ButtonCode::Forward, status: ButtonStatus::Pressed, .. }) => {
                self.go_to_neighbor(CycleDir::Next, hub, rq, context);
                true
            },
            _ => false,
        }
    }

    fn render(&self, _fb: &mut dyn Framebuffer, _rect: Rectangle, _fonts: &mut Fonts) {
        println!("calling render on workindex");
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
