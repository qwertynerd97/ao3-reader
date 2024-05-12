use crate::color::WHITE;
use crate::framebuffer::Framebuffer;
use crate::metadata::ReaderInfo;
use crate::settings::ReaderSettings;
use crate::view::filler::Filler;
//use crate::metadata::{DEFAULT_CONTRAST_EXPONENT, DEFAULT_CONTRAST_GRAY};
use crate::view::{Align, Bus, Event, Hub, Id, RenderQueue, View, ViewId, ID_FEEDER};
// use crate::view::filler::Filler;
// use crate::view::slider::Slider;
use crate::view::icon::{DisabledIcon, Icon};
// use crate::view::labeled_icon::LabeledIcon;
use crate::context::Context;
use crate::font::Fonts;
use crate::geom::Rectangle;
use crate::gesture::GestureEvent;
use crate::input::DeviceEvent;
use crate::view::label::Label;

#[derive(Debug, Clone)]
pub struct ToolBar {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    reflowable: bool,
    has_chapters: bool,
}

impl ToolBar {
    pub fn new(
        rect: Rectangle,
        reflowable: bool,
        _reader_info: Option<&ReaderInfo>,
        _reader_settings: &ReaderSettings,
        has_chapters: bool,
        has_kudos: bool,
    ) -> ToolBar {
        let id = ID_FEEDER.next();
        let mut children = Vec::new();
        let side = rect.height() as i32;

        if has_chapters {
            let toc_icon = Icon::new(
                "toc",
                rect![rect.min.x, rect.max.y - side, side, rect.max.y],
                Event::Show(ViewId::ChapterList),
            );
            children.push(Box::new(toc_icon) as Box<dyn View>);
        } else {
            let toc_icon = DisabledIcon::new(
                "toc-grey",
                rect![rect.min.x, rect.max.y - side, side, rect.max.y],
            );
            children.push(Box::new(toc_icon) as Box<dyn View>);
        }

        let remaining_width = rect.width() as i32 - 4 * side;

        // About Work
        let about_work_rect = rect![
            rect.min.x + side,
            rect.min.y,
            remaining_width + side,
            rect.min.y + side
        ];
        let about_work_label =
            Label::new(about_work_rect, "About Work".to_string(), Align::Left(0))
                .event(Some(Event::ToggleAbout));
        children.push(Box::new(about_work_label) as Box<dyn View>);

        // Kudos
        let kudos_rect = rect![
            remaining_width + rect.min.x + side,
            rect.min.y,
            remaining_width + 2 * side + rect.min.x,
            rect.min.y + side
        ];
        if has_kudos {
            let kudos_icon = Icon::new("heart", kudos_rect, Event::Kudos);
            children.push(Box::new(kudos_icon) as Box<dyn View>);
        } else {
            let kudos_filler = Filler::new(kudos_rect, WHITE);
            children.push(Box::new(kudos_filler) as Box<dyn View>);
        }



        // Bookmark
        let bookmark_icon = Icon::new(
            "bookmark",
            rect![
                remaining_width + 2 * side + rect.min.x,
                rect.min.y,
                remaining_width + 3 * side + rect.min.x,
                rect.max.y
            ],
            Event::Show(ViewId::LineHeightMenu),
        );
        children.push(Box::new(bookmark_icon) as Box<dyn View>);

        let search_icon = Icon::new(
            "search",
            rect![
                rect.max.x - 1 * side,
                rect.max.y - side,
                rect.max.x,
                rect.max.y
            ],
            Event::Show(ViewId::SearchBar),
        );
        children.push(Box::new(search_icon) as Box<dyn View>);

        ToolBar {
            id,
            rect,
            children,
            reflowable,
            has_chapters,
        }
    }

    // pub fn update_margin_width(&mut self, margin_width: i32, rq: &mut RenderQueue) {
    //     let index = if self.reflowable { 0 } else { 8 };
    //     if let Some(labeled_icon) = self.children[index].downcast_mut::<LabeledIcon>() {
    //         labeled_icon.update(&format!("{} mm", margin_width), rq);
    //     }
    // }

    // pub fn update_font_family(&mut self, font_family: String, rq: &mut RenderQueue) {
    //     if let Some(labeled_icon) = self.children[1].downcast_mut::<LabeledIcon>() {
    //         labeled_icon.update(&font_family, rq);
    //     }
    // }

    // pub fn update_line_height(&mut self, line_height: f32, rq: &mut RenderQueue) {
    //     if let Some(labeled_icon) = self.children[2].downcast_mut::<LabeledIcon>() {
    //         labeled_icon.update(&format!("{:.1} em", line_height), rq);
    //     }
    // }

    // pub fn update_text_align_icon(&mut self, text_align: TextAlign, rq: &mut RenderQueue) {
    //     let icon = self.child_mut(4).downcast_mut::<Icon>().unwrap();
    //     let name = text_align.icon_name();
    //     if icon.name != name {
    //         icon.name = name.to_string();
    //         rq.add(RenderData::new(icon.id(), *icon.rect(), UpdateMode::Gui));
    //     }
    // }

    // pub fn update_font_size_slider(&mut self, font_size: f32, rq: &mut RenderQueue) {
    //     let slider = self.children[6].as_mut().downcast_mut::<Slider>().unwrap();
    //     slider.update(font_size, rq);
    // }

    // pub fn update_contrast_exponent_slider(&mut self, exponent: f32, rq: &mut RenderQueue) {
    //     let slider = self.children[1].as_mut().downcast_mut::<Slider>().unwrap();
    //     slider.update(exponent, rq);
    // }

    // pub fn update_contrast_gray_slider(&mut self, gray: f32, rq: &mut RenderQueue) {
    //     let slider = self.children[3].as_mut().downcast_mut::<Slider>().unwrap();
    //     slider.update(gray, rq);
    // }
}

impl View for ToolBar {
    fn handle_event(
        &mut self,
        evt: &Event,
        _hub: &Hub,
        _bus: &mut Bus,
        _rq: &mut RenderQueue,
        _context: &mut Context,
    ) -> bool {
        match *evt {
            Event::Gesture(GestureEvent::Tap(center))
            | Event::Gesture(GestureEvent::HoldFingerShort(center, ..))
                if self.rect.includes(center) =>
            {
                true
            }
            Event::Gesture(GestureEvent::Swipe { start, .. }) if self.rect.includes(start) => true,
            Event::Device(DeviceEvent::Finger { position, .. }) if self.rect.includes(position) => {
                true
            }
            _ => false,
        }
    }

    fn render(&self, _fb: &mut dyn Framebuffer, _rect: Rectangle, _fonts: &mut Fonts) {}

    fn resize(&mut self, rect: Rectangle, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        let side = rect.height() as i32;

        let mut index = 0;

        // Chapters icon
        self.children[index].resize(
            rect![rect.min.x, rect.max.y - side, side, rect.max.y],
            hub,
            rq,
            context,
        );
        index += 1;

        let remaining_width = rect.width() as i32 - 4 * side;

        // About Work label
        self.children[index].resize(
            rect![
                rect.min.x + side,
                rect.min.y,
                remaining_width + side,
                rect.min.y + side
            ],
            hub,
            rq,
            context,
        );
        index += 1;

        // Kudos icon
        self.children[index].resize(
            rect![
                remaining_width + rect.min.x + side,
                rect.min.y,
                remaining_width + 2 * side + rect.min.x,
                rect.min.y + side
            ],
            hub,
            rq,
            context,
        );
        index += 1;

        // Bookmark icon
        self.children[index].resize(
            rect![
                remaining_width + 2 * side + rect.min.x,
                rect.min.y,
                remaining_width + 3 * side + rect.min.x,
                rect.max.y
            ],
            hub,
            rq,
            context,
        );

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
