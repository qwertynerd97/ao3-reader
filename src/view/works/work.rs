use crate::device::CURRENT_DEVICE;
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::view::{View, Event, Hub, Bus, Id, ID_FEEDER, RenderQueue, RenderData};
use crate::font::{MD_SIZE, MD_YEAR, MD_KIND, MD_AUTHOR, WORK_LARGE, WORK_SMALL, NORMAL_STYLE};
use crate::color::{TEXT_NORMAL, TEXT_INVERTED_HARD};
use crate::gesture::GestureEvent;
use crate::ao3_metadata::Ao3Info;
use crate::font::{Fonts, font_from_style};
use crate::geom::{Rectangle, halves};
use crate::app::Context;
use crate::http::list_to_str;

const SIZE_BASE: f32 = 1000.0;

fn word_count(words: usize) -> String {
    let value = words as f32;
    let level = (value.max(1.0).log(SIZE_BASE).floor() as usize).min(3);
    let factor = value / (SIZE_BASE).powi(level as i32);
    let precision = level.saturating_sub(1 + factor.log(10.0).floor() as usize);
    format!("{0:.1$} {2}", factor, precision, [' ', 'K', 'M', 'B'][level])
}

#[derive(Clone)]
pub struct Work {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    pub info: Ao3Info,
    index: usize,
    active: bool,
    preview: bool,
    length: WorkView
}

#[derive(Clone)]
pub enum WorkView {
    Long,
    Short
}

impl Work {
    pub fn new(rect: Rectangle, data: String, index: usize, preview: bool, length: WorkView) -> Work {
        let info = Ao3Info::new(data);
        Work {
            id: ID_FEEDER.next(),
            rect,
            children: vec![],
            info,
            index,
            active: false,
            preview,
            length
        }
    }
}

impl View for Work {
    fn handle_event(&mut self, evt: &Event, hub: &Hub, bus: &mut Bus, rq: &mut RenderQueue, _context: &mut Context) -> bool {
        match *evt {
            Event::Gesture(GestureEvent::Tap(center)) if self.rect.includes(center) => {
                self.active = true;
                let id = &self.info.id;
                rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                hub.send(Event::OpenWork(id.to_string())).ok();
                true
            },
            Event::Gesture(GestureEvent::HoldFingerShort(center, ..)) if self.rect.includes(center) => {
                let pt = pt!(center.x, self.rect.center().y);
                bus.push_back(Event::ToggleAboutWork(self.info.clone()));
                true
            },
            _ => false,
        }
    }

    fn render(&self, fb: &mut dyn Framebuffer, _rect: Rectangle, fonts: &mut Fonts) {
        let dpi = CURRENT_DEVICE.dpi;

        let scheme = if self.active {
            TEXT_INVERTED_HARD
        } else {
            TEXT_NORMAL
        };

        fb.draw_rectangle(&self.rect, scheme[0]);


        let (x_height, padding, baseline) = {
            let font = font_from_style(fonts, &MD_AUTHOR, dpi);
            let x_height = font.x_heights.0 as i32;
            (x_height, font.em() as i32, font.line_height() as i32)
        };

        let small_baseline = {
            let font = font_from_style(fonts, &MD_YEAR, dpi);
            ((font.line_height() as i32) / 4 )* 3
        };

        let height = self.rect.height() as i32;
        let (small_half_padding, _big_half_padding) = halves(padding);
        let third_width = 8 * x_height;
        let mut width = self.rect.width() as i32 - third_width - padding - small_half_padding;

        let mut start_y = self.rect.min.y;
        let mut start_x = self.rect.min.x + padding;

        if self.preview {
            width = width - height; // Preview icon is a square
            start_x = height;
            let preview_rect = rect![self.rect.min.x + padding,
                                    self.rect.min.y + padding,
                                    self.rect.min.x + (height - padding),
                                    self.rect.max.y - padding,
                                    ];
            let icons = self.info.req_tags.as_icons(preview_rect);
            for icon in icons {
                icon.render(fb, preview_rect, fonts);
            }
        }


        // Title
        {
            let author_list = list_to_str(&self.info.authors, ", ");
            let title = format!("{} by {}", self.info.title, author_list);
            let font = font_from_style(fonts, &WORK_LARGE, dpi);
            let mut plan = font.plan(&title, None, None);
            let mut title_lines = 1;

            if plan.width > width {
                let available = width;
                if available > 3 * padding {
                    let (index, usable_width) = font.cut_point(&plan, width);
                    let leftover = plan.width - usable_width;
                    if leftover > 2 * padding {
                        let mut plan2 = plan.split_off(index, usable_width);
                        let max_width = available - 0;
                        font.trim_left(&mut plan2);
                        font.crop_right(&mut plan2, max_width);
                        let pt = pt!(start_x + padding,
                                        start_y + (baseline / 4) * 7);
                        font.render(fb, scheme[1], &plan2, pt);
                        title_lines += 1;
                    } else {
                        font.crop_right(&mut plan, width);
                    }
                } else {
                    font.crop_right(&mut plan, width);
                }
            }

            let pt = pt!(start_x, start_y + baseline);
            font.render(fb, scheme[1], &plan, pt);

            if title_lines == 1 {
                start_y = start_y + baseline + 2 * small_half_padding;
            } else {
                start_y = start_y +  2 * baseline + small_half_padding;
            };
        }


        // Fandoms
        {
            let fandoms = list_to_str(&self.info.fandoms, ", ");
            let font = font_from_style(fonts, &NORMAL_STYLE, dpi);
            let plan = font.plan(&fandoms, Some(width), None);

            let pt = pt!(start_x, start_y);
            font.render(fb, scheme[1], &plan, pt);
            start_y = start_y + small_baseline;
        }

        // Tags
        match self.length {
            WorkView::Long => {
            let tags = list_to_str(&self.info.tags, ", ");
            let font = font_from_style(fonts, &WORK_SMALL, dpi);
            let mut plan = font.plan(&tags, None, None);
            let mut tag_lines = 1;
            let max_lines = (self.rect.max.y - start_y) / small_baseline;

            while tag_lines < max_lines {
                if plan.width > width {
                    let (index, usable_width) = font.cut_point(&plan, width);
                    let temp_plan = plan.split_off(index, usable_width);
                    let pt = pt!(start_x,
                        start_y);
                    font.render(fb, scheme[1], &plan, pt);
                    plan = temp_plan;
                    tag_lines += 1;
                    start_y += (small_baseline / 4) * 3;
                }
            }
            
            // Handle the last line
            if plan.width > width {
                font.crop_right(&mut plan, width);
            }

            let pt = pt!(start_x,
                start_y);
            font.render(fb, scheme[1], &plan, pt);

        },
        WorkView::Short => {}
    }


        // Pub Date
        {
            let date = format!("{}", self.info.updated.format("%d %b %Y"));
            let font = font_from_style(fonts, &MD_KIND, dpi);
            let plan = font.plan(&date, None, None);
            let pt = pt!(self.rect.max.x - padding - plan.width,
                            self.rect.max.y -  2 * baseline);
            font.render(fb, scheme[1], &plan, pt);
        }

        // Word Count
        {
            let size = word_count(self.info.words);
            let font = font_from_style(fonts, &MD_SIZE, dpi);
            let plan = font.plan(&size, None, None);
            let pt = pt!(self.rect.max.x - padding - plan.width,
                         self.rect.max.y - baseline);
            font.render(fb, scheme[1], &plan, pt);
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

    fn id(&self) -> Id {
        self.id
    }
}
