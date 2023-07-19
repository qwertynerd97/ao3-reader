
use std::sync::{Arc, Mutex};
use std::collections::{VecDeque, BTreeMap};
use fxhash::{FxHashMap, FxHashSet};
use crate::view::{View, Scrollable, Event, AppCmd, Hub, Bus, RenderQueue, RenderData};
use crate::view::{ViewId, Id, ID_FEEDER, EntryKind, EntryId, SliderId};
use crate::geom::{Point, Vec2, Rectangle, Boundary, CornerSpec, BorderSpec};
use crate::document::{Document, open, Location, TextLocation, BoundedText, Neighbors, BYTES_PER_PAGE};
use crate::document::html::HtmlDocument;
use crate::framebuffer::{Framebuffer, UpdateMode, Pixmap};
use crate::gesture::GestureEvent;
use crate::unit::{scale_by_dpi, mm_to_px};
use crate::view::reader::{Resource, RenderChunk, RECT_DIST_JITTER};
use crate::app::Context;
use crate::device::CURRENT_DEVICE;

pub struct HtmlView {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    doc: HtmlDocument,
    cache: BTreeMap<usize, Resource>,                // Cached page pixmaps.
    chunks: Vec<RenderChunk>,                        // Chunks of pages being rendered.
    text: FxHashMap<usize, Vec<BoundedText>>,        // Text of the current chunks.
    current_page: usize,
    pages_count: usize,
    font_size: f32
}

impl HtmlView {
    fn new(rect: Rectangle, hub: &Hub, context: &mut Context, content: &str) -> HtmlView {
        let id = ID_FEEDER.next();
        let settings = &context.settings;
        let doc = HtmlDocument::new_from_memory(content);
        let pages_count = doc.pages_count();

        HtmlView {
            id,
            rect,
            children: Vec::new(),
            doc: doc,
            cache: BTreeMap::new(),
            text: FxHashMap::default(),
            chunks: Vec::new(),
            pages_count,
            current_page: 0,
            font_size: settings.reader.font_size
        }
    }
}

impl Scrollable for HtmlView {
    fn next(&self, rq: &mut RenderQueue) {
        self.current_page = self.current_page + 1;
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    fn prev(&self, rq: &mut RenderQueue) {
        self.current_page = self.current_page - 1;
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    fn set_size(&self, rq: &mut RenderQueue, rect: Rectangle) {
        self.rect = rect;
        self.doc.layout(rect.width(), rect.height(), self.font_size, CURRENT_DEVICE.dpi);
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

}

impl View for HtmlView {
    fn render(&self, fb: &mut dyn Framebuffer, rect: Rectangle, _fonts: &mut Fonts) {
        fb.draw_rectangle(&rect, WHITE);

        for chunk in &self.chunks {
            let Resource { ref pixmap, scale, .. } = self.cache[&chunk.location];
            let chunk_rect = chunk.frame - chunk.frame.min + chunk.position;

            if let Some(region_rect) = rect.intersection(&chunk_rect) {
                let chunk_frame = region_rect - chunk.position + chunk.frame.min;
                let chunk_position = region_rect.min;
                fb.draw_framed_pixmap_contrast(pixmap, &chunk_frame, chunk_position, self.contrast.exponent, self.contrast.gray);

                if let Some(groups) = self.search.as_ref().and_then(|s| s.highlights.get(&chunk.location)) {
                    for rects in groups {
                        let mut last_rect: Option<Rectangle> = None;
                        for r in rects {
                            let rect = (*r * scale).to_rect() - chunk.frame.min + chunk.position;
                            if let Some(ref search_rect) = rect.intersection(&region_rect) {
                                fb.invert_region(search_rect);
                            }
                            if let Some(last) = last_rect {
                                if rect.min.y < last.max.y && last.min.y < rect.max.y && (last.max.x < rect.min.x || rect.max.x < last.min.x) {
                                    let space = if last.max.x < rect.min.x {
                                        rect![last.max.x, (last.min.y + rect.min.y) / 2,
                                              rect.min.x, (last.max.y + rect.max.y) / 2]
                                    } else {
                                        rect![rect.max.x, (last.min.y + rect.min.y) / 2,
                                              last.min.x, (last.max.y + rect.max.y) / 2]
                                    };
                                    if let Some(ref res_rect) = space.intersection(&region_rect) {
                                        fb.invert_region(res_rect);
                                    }
                                }
                            }
                            last_rect = Some(rect);
                        }
                    }
                }

                if let Some(annotations) = self.annotations.get(&chunk.location) {
                    for annot in annotations {
                        let drift = if annot.note.is_empty() { HIGHLIGHT_DRIFT } else { ANNOTATION_DRIFT };
                        let [start, end] = annot.selection;
                        if let Some(text) = self.text.get(&chunk.location) {
                            let mut last_rect: Option<Rectangle> = None;
                            for word in text.iter().filter(|w| w.location >= start && w.location <= end) {
                                let rect = (word.rect * scale).to_rect() - chunk.frame.min + chunk.position;
                                if let Some(ref sel_rect) = rect.intersection(&region_rect) {
                                    fb.shift_region(sel_rect, drift);
                                }
                                if let Some(last) = last_rect {
                                    if rect.min.y < last.max.y && last.min.y < rect.max.y && (last.max.x < rect.min.x || rect.max.x < last.min.x) {
                                        let space = if last.max.x < rect.min.x {
                                            rect![last.max.x, (last.min.y + rect.min.y) / 2,
                                                  rect.min.x, (last.max.y + rect.max.y) / 2]
                                        } else {
                                            rect![rect.max.x, (last.min.y + rect.min.y) / 2,
                                                  last.min.x, (last.max.y + rect.max.y) / 2]
                                        };
                                        if let Some(ref sel_rect) = space.intersection(&region_rect) {
                                            fb.shift_region(sel_rect, drift);
                                        }
                                    }
                                }
                                last_rect = Some(rect);
                            }
                        }
                    }
                }

                if let Some(sel) = self.selection.as_ref() {
                    if let Some(text) = self.text.get(&chunk.location) {
                        let mut last_rect: Option<Rectangle> = None;
                        for word in text.iter().filter(|w| w.location >= sel.start && w.location <= sel.end) {
                            let rect = (word.rect * scale).to_rect() - chunk.frame.min + chunk.position;
                            if let Some(ref sel_rect) = rect.intersection(&region_rect) {
                                fb.invert_region(sel_rect);
                            }
                            if let Some(last) = last_rect {
                                if rect.min.y < last.max.y && last.min.y < rect.max.y && (last.max.x < rect.min.x || rect.max.x < last.min.x) {
                                    let space = if last.max.x < rect.min.x {
                                        rect![last.max.x, (last.min.y + rect.min.y) / 2,
                                              rect.min.x, (last.max.y + rect.max.y) / 2]
                                    } else {
                                        rect![rect.max.x, (last.min.y + rect.min.y) / 2,
                                              last.min.x, (last.max.y + rect.max.y) / 2]
                                    };
                                    if let Some(ref sel_rect) = space.intersection(&region_rect) {
                                        fb.invert_region(sel_rect);
                                    }
                                }
                            }
                            last_rect = Some(rect);
                        }
                    }
                }
            }
        }

        if self.info.reader.as_ref().map_or(false, |r| r.bookmarks.contains(&self.current_page)) {
            let dpi = CURRENT_DEVICE.dpi;
            let thickness = scale_by_dpi(3.0, dpi) as u16;
            let radius = mm_to_px(0.4, dpi) as i32 + thickness as i32;
            let center = pt!(self.rect.max.x - 5 * radius,
                             self.rect.min.y + 5 * radius);
            fb.draw_rounded_rectangle_with_border(&Rectangle::from_disk(center, radius),
                                                  &CornerSpec::Uniform(radius),
                                                  &BorderSpec { thickness, color: WHITE },
                                                  &BLACK);
        }
    }
    
    fn handle_event(&mut self, evt: &Event, hub: &Hub, _bus: &mut Bus, rq: &mut RenderQueue, context: &mut Context) -> bool {
        match *evt {
            Event::Gesture(GestureEvent::Tap(center)) if self.rect.includes(center) => {

                let mut nearest_link = None;
                let mut dmin = u32::MAX;
                let dmax = (scale_by_dpi(RECT_DIST_JITTER, CURRENT_DEVICE.dpi) as i32).pow(2) as u32;

                for chunk in &self.chunks {
                    let (links, _) = self.doc.lock().ok()
                                         .and_then(|mut doc| doc.links(Location::Exact(chunk.location)))
                                         .unwrap_or((Vec::new(), 0));
                    for link in links {
                        let rect = (link.rect * chunk.scale).to_rect() - chunk.frame.min + chunk.position;
                        let d = center.rdist2(&rect);
                        if d < dmax && d < dmin {
                            dmin = d;
                            nearest_link = Some(link.clone());
                        }
                    }
                }

                if let Some(link) = nearest_link.take() {
                    if link.text.starts_with("http://") | link.text.starts_with("https://") {
                        let uri = String::from(&link.text);
                        if !context.settings.wifi {
                            hub.send(Event::SetWifi(true)).ok();
                        }
                        let html = context.client.get(&uri).send();
                        match html {
                        Ok(r) => {
                            let text = r.text();
                            match text {
                                Ok(t) => hub.send(Event::OpenHtml(t, Some(uri))).ok(),
                                Err(e) => hub.send(Event::Notify(format!("There was an error in the response body of {}:\n{}", uri, e))).ok(),
                            };
                        },
                        Err(e) => {
                            hub.send(Event::Notify(format!("{}", e))).ok();
                        }
                    }

                    } else {
                        let mut doc = self.doc;
                        let loc = Location::LocalUri(self.current_page, link.text.clone());
                        if let Some(location) = doc.resolve_location(loc) {
                            hub.send(Event::GoTo(location)).ok();
                        } else {
                            eprintln!("Can't resolve URI: {}.", link.text);
                        }
                    }
                    return true;
                }

                true
            },
            _ => false,
        }
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

}