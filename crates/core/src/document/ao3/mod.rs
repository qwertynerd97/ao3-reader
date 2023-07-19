use std::io::Read;
use std::fs::File;
use std::path::{Path, PathBuf};
use fxhash::FxHashMap;
use regex::Regex;
use anyhow::Error;
// use scraper::Node;
use crate::http::{scrape, scrape_csrf};
use crate::ao3_metadata::Ao3Info;
use crate::framebuffer::Pixmap;
use crate::helpers::decode_entities;
use crate::document::{Document, Location, TextLocation, TocEntry, BoundedText, Chapter};
use crate::unit::pt_to_px;
use crate::geom::{Rectangle, Edge, CycleDir, Boundary};
use super::html::dom::{XmlTree, NodeRef};
// use super::html::dom::Node;
use super::html::engine::{Page, Engine};
use super::html::layout::{StyleData, LoopContext};
use super::html::layout::{RootData, DrawState, DrawCommand, TextCommand, ImageCommand};
use super::html::layout::TextAlign;
use super::html::style::StyleSheet;
use super::html::xml::XmlParser;

const VIEWER_STYLESHEET: &str = "css/epub.css";
const USER_STYLESHEET: &str = "css/epub-user.css";

type UriCache = FxHashMap<String, usize>;

pub struct Ao3Document {
    text: String,
    url: Option<String>,
    parsed_doc: scraper::html::Html,
    content: XmlTree,
    engine: Engine,
    pages: Vec<Page>,
    parent: PathBuf,
    size: usize,
    ao3info: Ao3Info,
    viewer_stylesheet: PathBuf,
    user_stylesheet: PathBuf,
    ignore_document_css: bool,
}

#[derive(Debug)]
struct Chunk {
    path: String,
    size: usize,
}

unsafe impl Send for Ao3Document {}
unsafe impl Sync for Ao3Document {}

impl Ao3Document {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Ao3Document, Error> {
        let mut file = File::open(&path)?;
        let size = file.metadata()?.len() as usize;
        let mut text = String::new();
        file.read_to_string(&mut text)?;
        let document = scraper::Html::parse_document(&text);
        let body_selector = scraper::Selector::parse("#workskin").unwrap();
        let body_text = document.select(&body_selector).next().unwrap().inner_html();
        let mut content = XmlParser::new(&body_text).parse();
        content.wrap_lost_inlines();
        let parent = path.as_ref().parent().unwrap_or_else(|| Path::new(""));

        let meta = scrape(&document, ".wrapper .meta");
        let preface = scrape(&document, "#workskin .preface");
        let ao3info = Ao3Info::new_from_work(meta, preface, "0".to_string());

        Ok(Ao3Document {
            text,
            url: None,
            parsed_doc: document,
            content,
            engine: Engine::new(),
            pages: Vec::new(),
            parent: parent.to_path_buf(),
            size,
            viewer_stylesheet: PathBuf::from(VIEWER_STYLESHEET),
            user_stylesheet: PathBuf::from(USER_STYLESHEET),
            ignore_document_css: false,
            ao3info
        })
    }

    pub fn new_from_memory(text: &str, url: Option<&str>) -> Ao3Document {
        let document = scraper::Html::parse_document(&text);
        let body_selector = scraper::Selector::parse("#workskin").unwrap();
        let body_text = document.select(&body_selector).next().unwrap().inner_html();
        let size = body_text.len();
        let mut content = XmlParser::new(&body_text).parse();
        content.wrap_lost_inlines();

        let mut id = "0".to_string(); 
        let mut rewrapped_url = None;
        if let Some(unwrapped_url) = url {
            let id_re = Regex::new(r"works/(\d+)").unwrap();
            if let Some(caps) = id_re.captures(&unwrapped_url) {
                id = caps[1].to_string();
            }
            rewrapped_url = Some(unwrapped_url.to_string());
        }

        let meta = scrape(&document, ".wrapper .meta");
        let preface = scrape(&document, "#workskin .preface");
        let ao3info = Ao3Info::new_from_work(meta, preface, id);

        Ao3Document {
            text: text.to_string(),
            url: rewrapped_url,
            parsed_doc: document,
            content,
            engine: Engine::new(),
            pages: Vec::new(),
            parent: PathBuf::from(""),
            size,
            viewer_stylesheet: PathBuf::from(VIEWER_STYLESHEET),
            user_stylesheet: PathBuf::from(USER_STYLESHEET),
            ignore_document_css: false,
            ao3info
        }
    }

    // pub fn new_from_uri(uri: &str) -> Ao3Document {

    // }
    pub fn update(&mut self, text: &str) {
        self.parsed_doc = scraper::Html::parse_document(&text);
        let body_selector = scraper::Selector::parse("#workskin").unwrap();
        let body_text = self.parsed_doc.select(&body_selector).next().unwrap().inner_html();
        self.size = body_text.len();
        self.content = XmlParser::new(&body_text).parse();
        self.content.wrap_lost_inlines();
        self.text = text.to_string();
        self.pages.clear();
    }

    pub fn set_margin(&mut self, margin: &Edge) {
        self.engine.set_margin(margin);
        self.pages.clear();
    }

    pub fn set_font_size(&mut self, font_size: f32) {
        self.engine.set_font_size(font_size);
        self.pages.clear();
    }

    pub fn set_viewer_stylesheet<P: AsRef<Path>>(&mut self, path: P) {
        self.viewer_stylesheet = path.as_ref().to_path_buf();
        self.pages.clear();
    }

    pub fn set_user_stylesheet<P: AsRef<Path>>(&mut self, path: P) {
        self.user_stylesheet = path.as_ref().to_path_buf();
        self.pages.clear();
    }

    #[inline]
    fn rect(&self) -> Rectangle {
        let (width, height) = self.engine.dims;
        rect![0, 0, width as i32, height as i32]
    }

    #[inline]
    fn page_index(&mut self, offset: usize) -> Option<usize> {
        if self.pages.is_empty() {
            self.pages = self.build_pages();
        }
        if self.pages.len() < 2 || self.pages[1].first().map(|dc| offset < dc.offset()) == Some(true) {
            return Some(0);
        } else if self.pages[self.pages.len() - 1].first().map(|dc| offset >= dc.offset()) == Some(true) {
            return Some(self.pages.len() - 1);
        } else {
            for i in 1..self.pages.len()-1 {
                if self.pages[i].first().map(|dc| offset >= dc.offset()) == Some(true) &&
                   self.pages[i+1].first().map(|dc| offset < dc.offset()) == Some(true) {
                    return Some(i);
                }
            }
        }
        None
    }

    fn resolve_link(&mut self, uri: &str, cache: &mut UriCache) -> Option<usize> {
        let frag_index = uri.find('#')?;
        let name = &uri[..frag_index];
        let content = self.content.clone();
        self.cache_uris(content.root(), name, cache);
        cache.get(uri).cloned()
    }
    // fn resolve_remote(&mut self, uri: &str) -> Option<usize> {
    //     return reqwest::blocking::get(uri).ok()?.text().ok()?;
    // }

    fn cache_uris(&mut self, node: NodeRef, name: &str, cache: &mut UriCache) {
        if let Some(id) = node.attribute("id") {
            cache.insert(format!("{}#{}", name, id), node.offset());
        }
        for child in node.children() {
            self.cache_uris(child, name, cache);
        }
    }

    fn images(&mut self, _loc: Location) -> Option<(Vec<Rectangle>, usize)> {
        // if self.spine.is_empty() {
        //     return None;
        // }

        // let offset = self.resolve_location(loc)?;
        // let (index, start_offset) = self.vertebra_coordinates(offset)?;
        // let page_index = self.page_index(offset, index, start_offset)?;

        // self.cache.get(&index).map(|display_list| {
        //     (display_list[page_index].iter().filter_map(|dc| {
        //         match dc {
        //             DrawCommand::Image(ImageCommand { rect, .. }) => Some(*rect),
        //             _ => None,
        //         }
        //     }).collect(), offset)
        // })

        return None;
    }

    fn build_pages(&mut self) -> Vec<Page> {
        let mut stylesheet = StyleSheet::new();
        let spine_dir = PathBuf::from("");

        // if let Ok(text) = fs::read_to_string(&self.viewer_stylesheet) {
        //     let (mut css, _) = CssParser::new(&text).parse(RuleKind::Viewer);
        //     stylesheet.append(&mut css);
        // }

        // if let Ok(text) = fs::read_to_string(&self.user_stylesheet) {
        //     let (mut css, _) = CssParser::new(&text).parse(RuleKind::User);
        //     stylesheet.append(&mut css);
        // }

        // if !self.ignore_document_css {
        //     if let Some(head) = self.content.find("head") {
        //         if let Some(children) = head.children() {
        //             for child in children {
        //                 if child.tag_name() == Some("link") && child.attr("rel") == Some("stylesheet") {
        //                     if let Some(href) = child.attr("href") {
        //                         if let Some(name) = spine_dir.join(href).normalize().to_str() {
        //                             if let Ok(buf) = self.parent.fetch(name) {
        //                                 if let Ok(text) = String::from_utf8(buf) {
        //                                     let (mut css, _) = CssParser::new(&text).parse(RuleKind::Document);
        //                                     stylesheet.append(&mut css);
        //                                 }
        //                             }
        //                         }
        //                     }
        //                 } else if child.tag_name() == Some("style") && child.attr("type") == Some("text/css") {
        //                     if let Some(text) = child.text() {
        //                         let (mut css, _) = CssParser::new(text).parse(RuleKind::Document);
        //                         stylesheet.append(&mut css);
        //                     }
        //                 }
        //             }
        //         }
        //     }
        // }

        let mut pages = Vec::new();

        let mut rect = self.engine.rect();
        rect.shrink(&self.engine.margin);

        let language = self.content.root().find("html")
                           .and_then(|html| html.attribute("xml:lang"))
                           .map(String::from);

        let style = StyleData {
            language,
            font_size: self.engine.font_size,
            line_height: pt_to_px(self.engine.line_height * self.engine.font_size, self.engine.dpi).round() as i32,
            text_align: self.engine.text_align,
            start_x: rect.min.x,
            end_x: rect.max.x,
            width: rect.max.x - rect.min.x,
            .. Default::default()
        };

        let loop_context = LoopContext::default();
        let mut draw_state = DrawState {
            position: rect.min,
            .. Default::default()
        };

        let root_data = RootData {
            start_offset: 0,
            spine_dir,
            rect,
        };

        pages.push(Vec::new());

        self.engine.build_display_list(self.content.root(), &style, &loop_context, &stylesheet, &root_data, &mut self.parent, &mut draw_state, &mut pages);

        pages.retain(|page| !page.is_empty());

        if pages.is_empty() {
            pages.push(vec![DrawCommand::Marker(self.content.root().offset())]);
        }

        pages
    }

    // pub fn categories(&self) -> Option<String> {
    //     None
    // }

    // fn chapter_aux<'a>(&mut self, toc: &'a [TocEntry], offset: usize, next_offset: usize, path: &str, chap_before: &mut Option<&'a TocEntry>, offset_before: &mut usize, chap_after: &mut Option<&'a TocEntry>, offset_after: &mut usize) {
    //     for entry in toc {
    //         if let Location::Uri(ref uri) = entry.location {
    //             if uri.starts_with(path) {
    //                 if let Some(entry_offset) = self.resolve_location(entry.location.clone()) {
    //                     if entry_offset < offset && (chap_before.is_none() || entry_offset > *offset_before) {
    //                         *chap_before = Some(entry);
    //                         *offset_before = entry_offset;
    //                     }
    //                     if entry_offset >= offset && entry_offset < next_offset && (chap_after.is_none() || entry_offset < *offset_after) {
    //                         *chap_after = Some(entry);
    //                         *offset_after = entry_offset;
    //                     }
    //                 }
    //             }
    //         }
    //         self.chapter_aux(&entry.children, offset, next_offset, path,
    //                          chap_before, offset_before, chap_after, offset_after);
    //     }
    // }

    fn previous_chapter<'a>(&mut self, chap: Option<&TocEntry>, start_offset: usize, end_offset: usize, toc: &'a [TocEntry]) -> Option<&'a TocEntry> {
        for entry in toc.iter().rev() {
            let result = self.previous_chapter(chap, start_offset, end_offset, &entry.children);
            if result.is_some() {
                return result;
            }

            if let Some(chap) = chap {
                if entry.index < chap.index {
                    let entry_offset = self.resolve_location(entry.location.clone())?;
                    if entry_offset < start_offset || entry_offset >= end_offset {
                        return Some(entry)
                    }
                }
            } else {
                let entry_offset = self.resolve_location(entry.location.clone())?;
                if entry_offset < start_offset {
                    return Some(entry);
                }
            }
        }
        None
    }

    fn next_chapter<'a>(&mut self, chap: Option<&TocEntry>, start_offset: usize, end_offset: usize, toc: &'a [TocEntry]) -> Option<&'a TocEntry> {
        for entry in toc {
            if let Some(chap) = chap {
                if entry.index > chap.index {
                    let entry_offset = self.resolve_location(entry.location.clone())?;
                    if entry_offset < start_offset || entry_offset >= end_offset {
                        return Some(entry)
                    }
                }
            } else {
                let entry_offset = self.resolve_location(entry.location.clone())?;
                if entry_offset >= end_offset {
                    return Some(entry);
                }
            }

            let result = self.next_chapter(chap, start_offset, end_offset, &entry.children);
            if result.is_some() {
                return result;
            }
        }
        None
    }

    // pub fn series(&self) -> Option<(String, String)> {
    //     self.info.find("metadata")
    //         .and_then(Node::children)
    //         .and_then(|children| {
    //             let mut title = None;
    //             let mut index = None;

    //             for child in children {
    //                 if child.tag_name() == Some("meta") {
    //                     if child.attr("name") == Some("calibre:series") {
    //                         title = child.attr("content").map(|s| decode_entities(s).into_owned());
    //                     } else if child.attr("name") == Some("calibre:series_index") {
    //                         index = child.attr("content").map(|s| decode_entities(s).into_owned());
    //                     } else if child.attr("property") == Some("belongs-to-collection") {
    //                         title = child.text().map(|text| decode_entities(text).into_owned());
    //                     } else if child.attr("property") == Some("group-position") {
    //                         index = child.text().map(|text| decode_entities(text).into_owned());
    //                     }
    //                 }

    //                 if title.is_some() && index.is_some() {
    //                     break;
    //                 }
    //             }

    //             title.into_iter().zip(index).next()
    //         })
    // }

    // pub fn language(&self) -> Option<String> {
    //     let lang_selector = scraper::Selector::parse("dd.language").unwrap();
    //     Some(self.parsed_doc.select(&lang_selector).next().unwrap().html())
    // }

    // pub fn year(&self) -> Option<String> {
    //     let year_selector = scraper::Selector::parse("dd.published").unwrap();
    //     Some(self.parsed_doc.select(&year_selector).next().unwrap().html())
    // }

}

impl Document for Ao3Document {
    #[inline]
    fn dims(&self, _index: usize) -> Option<(f32, f32)> {
        Some((self.engine.dims.0 as f32, self.engine.dims.1 as f32))
    }

    fn pages_count(&self) -> usize {
        self.size
    }

    fn about(&self) -> String {
        let blurb_select = scraper::Selector::parse(".wrapper .meta").unwrap();
        self.parsed_doc.select(&blurb_select).next().unwrap().inner_html()
    }

    fn kudos_token(&self) -> String {
        scrape_csrf(&self.parsed_doc)
    }

    fn set_hyphen_penalty(&mut self, hyphen_penalty: i32) {
        self.engine.set_hyphen_penalty(hyphen_penalty);
        self.pages.clear();
    }

    fn set_stretch_tolerance(&mut self, stretch_tolerance: f32) {
        self.engine.set_stretch_tolerance(stretch_tolerance);
        self.pages.clear();
    }

    fn work_id(&self) -> String {
        let mut work_id = "".to_string();
        if let Some(unwrapped_url) = &self.url {
            let work_url = Regex::new(r"works/(\d+)").unwrap();
            let caps = work_url.captures(&unwrapped_url).unwrap();
            work_id = caps[1].to_string();
        }
        work_id
    }

    fn images(&mut self, loc: Location) -> Option<(Vec<Boundary>, usize)> {
        let offset = self.resolve_location(loc)?;
        let page_index = self.page_index(offset)?;

        Some((self.pages[page_index].iter().filter_map(|dc| {
            match dc {
                DrawCommand::Image(ImageCommand { rect, .. }) => Some((*rect).into()),
                _ => None,
            }
        }).collect(), offset))
    }

    fn set_ignore_document_css(&mut self, ignore: bool) {
        self.ignore_document_css = ignore;
        self.pages.clear();
    }

    fn toc(&mut self) -> Option<Vec<TocEntry>> {
        let mut entries = Vec::new();
        let mut index = 0;

        let chapter_selector = scraper::Selector::parse(".chapter h3.title").unwrap();

        for (i, element) in self.parsed_doc.select(&chapter_selector).enumerate() {
            let text = element.text().collect::<Vec<_>>();
            let title = text.join("");
            // TODO: need to add the base work path to this
            let location = Location::Uri(format!("#chapter-{}", i + 1));
            let current_index = index;
            index += 1;
            let sub_entries = Vec::new(); // AO3 doesn't have nested chapters

            entries.push(TocEntry {
                title,
                location,
                index: current_index,
                children: sub_entries,
            });
        }

        Some(entries)
    }

    fn chapterlist(&self) -> Vec<Chapter> {
        let mut entries = Vec::new();

        let chapter_selector = scraper::Selector::parse(".chapter h3.title").unwrap();

        for (i, element) in self.parsed_doc.select(&chapter_selector).enumerate() {
            let text = element.text().collect::<Vec<_>>();
            let title = text.join("").trim().to_string();
            let location = format!("#chapter-{}", i + 1);

            entries.push(Chapter {
                title,
                location,
            });
        }

        entries
    }

    fn chapter<'a>(&mut self, _offset: usize, _toc: &'a [TocEntry]) -> Option<(&'a TocEntry, f32)> {
        // let next_offset = self.resolve_location(Location::Next(offset))
        //                       .unwrap_or(usize::MAX);
        // let (index, _) = self.vertebra_coordinates(offset)?;
        // let path = self.spine[index].path.clone();
        // let mut chap_before = None;
        // let mut chap_after = None;
        // let mut offset_before = 0;
        // let mut offset_after = usize::MAX;
        // self.chapter_aux(toc, offset, next_offset, &path,
        //                  &mut chap_before, &mut offset_before,
        //                  &mut chap_after, &mut offset_after);
        // if chap_after.is_none() && chap_before.is_none() {
        //     for i in (0..index).rev() {
        //         let chap = chapter_from_uri(&self.spine[i].path, toc);
        //         if chap.is_some() {
        //             return chap;
        //         }
        //     }
        //     None
        // } else {
        //     chap_after.or(chap_before)
        // }
        None
    }

    fn chapter_relative<'a>(&mut self, offset: usize, dir: CycleDir, toc: &'a [TocEntry]) -> Option<&'a TocEntry> {
        let next_offset = self.resolve_location(Location::Next(offset))
                              .unwrap_or(usize::MAX);
        let chap = self.chapter(offset, toc).map(|(c, _)| c);

        match dir {
            CycleDir::Previous => self.previous_chapter(chap, offset, next_offset, toc),
            CycleDir::Next => self.next_chapter(chap, offset, next_offset, toc),
        }
    }

    fn resolve_location(&mut self, loc: Location) -> Option<usize> {
        self.engine.load_fonts();

        match loc {
            Location::Exact(offset) => {
                let page_index = self.page_index(offset)?;
                self.pages[page_index].first()
                    .map(DrawCommand::offset)
            },
            Location::Previous(offset) => {
                let page_index = self.page_index(offset)?;
                if page_index > 0 {
                    self.pages[page_index-1].first().map(DrawCommand::offset)
                } else {
                    None
                }
            },
            Location::Next(offset) => {
                let page_index = self.page_index(offset)?;
                if page_index < self.pages.len() - 1 {
                    self.pages[page_index+1].first().map(DrawCommand::offset)
                } else {
                    None
                }
            },
            Location::LocalUri(_, ref uri) | Location::Uri(ref  uri) => {
                    let mut cache = FxHashMap::default();
                    self.resolve_link(uri, &mut cache)
                // }
            },
        }
    }

    fn words(&mut self, loc: Location) -> Option<(Vec<BoundedText>, usize)> {
        let offset = self.resolve_location(loc)?;
        let page_index = self.page_index(offset)?;

        Some((self.pages[page_index].iter().filter_map(|dc| {
            match dc {
                DrawCommand::Text(TextCommand { text, rect, offset, .. }) => {
                    Some(BoundedText {
                        text: text.clone(),
                        rect: (*rect).into(),
                        location: TextLocation::Dynamic(*offset),
                    })
                },
                _ => None,
            }
        }).collect(), offset))
    }

    fn lines(&mut self, _loc: Location) -> Option<(Vec<BoundedText>, usize)> {
        None
    }

    fn links(&mut self, loc: Location) -> Option<(Vec<BoundedText>, usize)> {
        let offset = self.resolve_location(loc)?;
        let page_index = self.page_index(offset)?;

        Some((self.pages[page_index].iter().filter_map(|dc| {
            match dc {
                DrawCommand::Text(TextCommand { uri, rect, offset, .. }) |
                DrawCommand::Image(ImageCommand { uri, rect, offset, .. }) if uri.is_some() => {
                    Some(BoundedText {
                        text: uri.clone().unwrap(),
                        rect: (*rect).into(),
                        location: TextLocation::Dynamic(*offset),
                    })
                },
                _ => None,
            }
        }).collect(), offset))
    }

    fn pixmap(&mut self, loc: Location, scale: f32) -> Option<(Pixmap, usize)> {
        let offset = self.resolve_location(loc)?;
        let page_index = self.page_index(offset)?;
        let page = self.pages[page_index].clone();
        let pixmap = self.engine.render_page(&page, scale, &mut self.parent)?;

        Some((pixmap, offset))
    }

    fn layout(&mut self, width: u32, height: u32, font_size: f32, dpi: u16) {
        self.engine.layout(width, height, font_size, dpi);
        self.pages.clear();
    }

    fn set_text_align(&mut self, text_align: TextAlign) {
        self.engine.set_text_align(text_align);
        self.pages.clear();
    }

    fn set_font_family(&mut self, family_name: &str, search_path: &str) {
        self.engine.set_font_family(family_name, search_path);
        self.pages.clear();
    }

    fn set_margin_width(&mut self, width: i32) {
        self.engine.set_margin_width(width);
        self.pages.clear();
    }

    fn set_line_height(&mut self, line_height: f32) {
        self.engine.set_line_height(line_height);
        self.pages.clear();
    }

    fn title(&self) -> Option<String> {
        let title = scrape(&self.parsed_doc,"h2.title");
        Some(title)
    }

    fn author(&self) -> Option<String> {
        let author_selector = scraper::Selector::parse("h3.byline").unwrap();
        Some(self.parsed_doc.select(&author_selector).next().unwrap().inner_html())
    }

    fn metadata(&self, key: &str) -> Option<String> {
        self.content.root().find("metadata")
            .and_then(|md| md.children().find(|child| child.tag_qualified_name() == Some(key)))
            .map(|child| decode_entities(&child.text()).into_owned())
    }

    fn is_reflowable(&self) -> bool {
        true
    }

    fn has_synthetic_page_numbers(&self) -> bool {
        true
    }

    fn ao3_meta(&self) -> Ao3Info {
        self.ao3info.clone()
    }
}
