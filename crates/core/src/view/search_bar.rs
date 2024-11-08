use crate::framebuffer::Framebuffer;
use crate::device::CURRENT_DEVICE;
use super::{View, Event, Hub, Bus, Id, ID_FEEDER, RenderQueue, ViewId, THICKNESS_MEDIUM, SMALL_BAR_HEIGHT};
use super::icon::Icon;
use super::input_field::InputField;
use super::filler::Filler;
use crate::gesture::GestureEvent;
use crate::input::DeviceEvent;
use crate::color::{TEXT_BUMP_SMALL, SEPARATOR_NORMAL, BLACK};
use crate::geom::Rectangle;
use crate::context::Context;
use crate::unit::scale_by_dpi;
use crate::font::Fonts;

#[derive(Debug, Clone)]
pub struct SearchBar {
    id: Id,
    pub rect: Rectangle,
    children: Vec<Box<dyn View>>,
}

impl SearchBar {
    pub fn new_empty(availible_space: Rectangle) -> SearchBar {
        let dpi = CURRENT_DEVICE.dpi;
        let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let bar_height = small_height + thickness;

        SearchBar {
            id: ID_FEEDER.next(),
            children: Vec::new(),
            // NOTE - SearchBar is a bottom-aligned component
            rect: rect![
                availible_space.min.x, availible_space.max.y - bar_height,
                availible_space.max.x, availible_space.max.y]
        }
    }
    pub fn new(availible_space: Rectangle, input_id: ViewId, placeholder: &str) -> SearchBar {
        let mut search_bar = SearchBar::new_empty(availible_space);

        let dpi = CURRENT_DEVICE.dpi;
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let side = search_bar.rect.height() as i32;

        // Search icon
        // TODO - There is probably a more logical way to handle this
        // event. Currently, clicking the search icon triggers ToggleNear
        // which he parent component has to handle.  Each parnt can implement
        // unique and contradictory logic for this behavior, which usually also
        // includes sending oof a search event
        let search_rect = rect![search_bar.rect.min, search_bar.rect.min + side];
        let search_icon = Icon::new("search",
                                    search_rect,
                                    Event::ToggleNear(ViewId::SearchMenu, search_rect))
                               .background(TEXT_BUMP_SMALL[0]);

        search_bar.children.push(Box::new(search_icon) as Box<dyn View>);
        
        // Vertical bar between icon and text input
        let separator = Filler::new(rect![search_bar.rect.min.x + side, search_bar.rect.min.y,
                                          search_bar.rect.min.x + side + thickness, search_bar.rect.max.y],
                                    SEPARATOR_NORMAL);

        search_bar.children.push(Box::new(separator) as Box<dyn View>);

        // Text input
        let input_field = InputField::new(rect![search_bar.rect.min.x + side + thickness, search_bar.rect.min.y,
                                                search_bar.rect.max.x - side - thickness, search_bar.rect.max.y],
                                          input_id)
                                     .border(false)
                                     .placeholder(placeholder);

        search_bar.children.push(Box::new(input_field) as Box<dyn View>);

        // Vertical bar between text input and close button
        let separator = Filler::new(rect![search_bar.rect.max.x - side - thickness, search_bar.rect.min.y,
                                          search_bar.rect.max.x - side, search_bar.rect.max.y],
                                    SEPARATOR_NORMAL);

        search_bar.children.push(Box::new(separator) as Box<dyn View>);

        // Close button
        let close_icon = Icon::new("close",
                                   rect![search_bar.rect.max.x - side, search_bar.rect.min.y,
                                         search_bar.rect.max.x, search_bar.rect.max.y],
                                   Event::Close(ViewId::SearchBar))
                              .background(TEXT_BUMP_SMALL[0]);

        search_bar.children.push(Box::new(close_icon) as Box<dyn View>);

        // Top seperator
        let separator = Filler::new(rect![
            search_bar.rect.min.x, search_rect.min.y,
            search_bar.rect.max.x, search_rect.min.y + thickness], BLACK);
        search_bar.children.push(Box::new(separator) as Box<dyn View>);

        search_bar
    }

    pub fn set_text(&mut self, text: &str, rq: &mut RenderQueue, context: &mut Context) {
        if let Some(input_field) = self.children[2].downcast_mut::<InputField>() {
            input_field.set_text(text, true, rq, context);
        }
    }
}

impl View for SearchBar {
    fn handle_event(&mut self, evt: &Event, _hub: &Hub, _bus: &mut Bus, _rq: &mut RenderQueue, _context: &mut Context) -> bool {
        match *evt {
            Event::Gesture(GestureEvent::Tap(center)) |
            Event::Gesture(GestureEvent::HoldFingerShort(center, ..)) if self.rect.includes(center) => true,
            Event::Gesture(GestureEvent::Swipe { start, .. }) if self.rect.includes(start) => true,
            Event::Device(DeviceEvent::Finger { position, .. }) if self.rect.includes(position) => true,
            _ => false,
        }
    }

    fn render(&self, _fb: &mut dyn Framebuffer, _rect: Rectangle, _fonts: &mut Fonts) {
    }

    fn resize(&mut self, rect: Rectangle, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        let dpi = CURRENT_DEVICE.dpi;
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let side = rect.height() as i32;
        self.children[0].resize(rect![rect.min, rect.min + side], hub, rq, context);
        self.children[1].resize(rect![pt!(rect.min.x + side, rect.min.y),
                                      pt!(rect.min.x + side + thickness, rect.max.y)], hub, rq, context);
        self.children[2].resize(rect![pt!(rect.min.x + side + thickness, rect.min.y),
                                      pt!(rect.max.x - side - thickness, rect.max.y)], hub, rq, context);
        self.children[3].resize(rect![pt!(rect.max.x - side - thickness, rect.min.y),
                                      pt!(rect.max.x - side, rect.max.y)], hub, rq, context);
        self.children[4].resize(rect![pt!(rect.max.x - side, rect.min.y),
                                      pt!(rect.max.x, rect.max.y)], hub, rq, context);
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
