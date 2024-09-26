use url::Url;
use crate::font::{Fonts, BOLD_STYLE};
use crate::view::{View, Event, Hub, Bus, RenderQueue, Align, ViewId, Id, ID_FEEDER, RenderData};
use crate::view::{MINI_BAR_HEIGHT, THICKNESS_MEDIUM, SMALL_PADDING};
use crate::context::Context;
use crate::unit::scale_by_dpi;
use crate::geom::Rectangle;
use crate::color::{BLACK, WHITE};
use crate::device::CURRENT_DEVICE;
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::view::textlabel::TextLabel;
use crate::view::filler::Filler;
use crate::font::LABEL_STYLE;
use crate::view::common::{locate, toggle_main_menu, toggle_battery_menu, toggle_clock_menu};
use super::top_bar::TopBar;
use crate::battery::Battery;

#[derive(Clone)]
pub struct Home {
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    id: Id,
    view_id: ViewId,
}

impl Home {
    pub fn new_empty(rect: Rectangle) -> Home {
        let id = ID_FEEDER.next();
        let children = Vec::new();

        Home {
            rect,
            children,
            id,
            view_id: ViewId::Home,
        }
    }

    pub fn new(rect: Rectangle, rq: &mut RenderQueue, context: &mut Context) -> Home {
        let mut home = Home::new_empty(rect);
        let dpi = CURRENT_DEVICE.dpi;

        home.create_background();

        let top_bar_index = home.children.len();
        home.create_top_bar(
            context.settings.time_format.clone(), &mut context.fonts, &mut context.battery, context.settings.frontlight);
        let top_bar = &home.children[top_bar_index];

        // TODO add login/logged in section

        let mut top_pos = top_bar.rect().height() as i32;
        let label_content_height = scale_by_dpi(MINI_BAR_HEIGHT, dpi) as i32;
        let label_seperator_thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let label_padding = scale_by_dpi(SMALL_PADDING, dpi) as i32;
        let row_height = label_content_height + label_seperator_thickness;

        // Link to 'Marked for Later' view
        if context.client.logged_in {
            home.create_marked_for_later(top_pos, label_content_height, label_padding, label_seperator_thickness);
            top_pos = top_pos + row_height;
        }

        let faves = &context.settings.ao3.faves;
        let mut fav_index = 0;
        while top_pos + row_height <= home.rect.max.y && fav_index < faves.len() {
            home.create_fav_search(faves[fav_index].clone(), top_pos, label_content_height, label_padding, label_seperator_thickness);
            top_pos = top_pos + row_height;
            fav_index = fav_index + 1;
        }

        rq.add(RenderData::new(home.id, rect, UpdateMode::Full));
        home
    }

    fn create_background(&mut self) {
        let bg = Filler::new(self.rect, WHITE);
        self.children.push(Box::new(bg) as Box<dyn View>);
    }

    fn create_top_bar(&mut self, format: String, fonts: &mut Fonts, battery: &mut Box<dyn Battery>, frontlight: bool) {
        let top_bar = TopBar::new(self.rect,
                                  Event::Toggle(ViewId::SearchBar),
                                  "Favorite Tags".to_string(),
                                  format, fonts, battery, frontlight);
        self.children.push(Box::new(top_bar) as Box<dyn View>);
    }

    fn create_marked_for_later(&mut self, top_pos: i32, label_content_height: i32, label_padding: i32, label_seperator_thickness: i32) {
        let label_rect = rect![
                self.rect.min.x, top_pos,
                self.rect.max.x, top_pos + label_content_height];
        let history = TextLabel::new(label_rect,
            "Marked For Later".to_string(),
            Align::Left(label_padding), BOLD_STYLE, Event::LoadHistory(super::works::HistoryView::MarkedForLater));
        self.children.push(Box::new(history) as Box<dyn View>);

        let seperator_rect = rect![
            self.rect.min.x, top_pos + label_content_height,
            self.rect.max.x, top_pos + label_content_height + label_seperator_thickness];
        let seperator = Filler::new(seperator_rect, BLACK);
        self.children.push(Box::new(seperator) as Box<dyn View>);
    }

    fn create_fav_search(&mut self, fave: (String, Url),
                         top_pos: i32, label_content_height: i32, label_padding: i32, label_seperator_thickness: i32) {
        // TODO - extract out to favs component
        let label_rect = rect![
            self.rect.min.x, top_pos,
            self.rect.max.x, top_pos + label_content_height];
        let chapter = TextLabel::new(label_rect,
            (*fave.0).to_string(),
            Align::Left(label_padding), LABEL_STYLE,
            Event::LoadIndex((fave.1).to_string()));
        self.children.push(Box::new(chapter) as Box<dyn View>);

        let seperator_rect = rect![
            self.rect.min.x, top_pos + label_content_height,
            self.rect.max.x, top_pos + label_content_height + label_seperator_thickness];
        let seperator = Filler::new(seperator_rect, BLACK);
        self.children.push(Box::new(seperator) as Box<dyn View>);
    }

    fn reseed(&mut self, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
            if let Some(top_bar) = self.child_mut(1).downcast_mut::<TopBar>() {
                top_bar.update_frontlight_icon(&mut RenderQueue::new(), context);
                hub.send(Event::ClockTick).ok();
                hub.send(Event::BatteryTick).ok();
            }
    
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
        }

}

impl View for Home {
    fn handle_event(&mut self, evt: &Event, hub: &Hub, _bus: &mut Bus, rq: &mut RenderQueue, context: &mut Context) -> bool {
        match *evt {
            Event::Reseed => {
                self.reseed(hub, rq, context);
                true
            },
            Event::ToggleFrontlight => {
                if let Some(index) = locate::<TopBar>(self) {
                    self.child_mut(index).downcast_mut::<TopBar>().unwrap()
                        .update_frontlight_icon(rq, context);
                }
                true
            },
            Event::ToggleNear(ViewId::MainMenu, rect) => {
                toggle_main_menu(self, rect, None, rq, context);
                true
            },
            Event::ToggleNear(ViewId::BatteryMenu, rect) => {
                toggle_battery_menu(self, rect, None, rq, context);
                true
            },
            Event::ToggleNear(ViewId::ClockMenu, rect) => {
                toggle_clock_menu(self, rect, None, rq, context);
                true
            },
            Event::Close(ViewId::MainMenu) => {
                toggle_main_menu(self, Rectangle::default(), Some(false), rq, context);
                true
            },
            _ => false
        }
    }

    fn render(&self, _fb: &mut dyn Framebuffer, _rect: Rectangle, _fonts: &mut Fonts) {
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

    fn view_id(&self) -> Option<ViewId> {
        Some(self.view_id)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::battery::FakeBattery;

    #[test]
    #[allow(non_snake_case)]
    fn WHEN_createBackgroundIsCalled_THEN_aFullSizeWhiteRectangleIsAddedToChildren() {
        // WHEN create_background is called
        let mut home = Home::new_empty(rect![0, 0, 600, 800]);
        home.create_background();
        // THEN a full size white rectangle is added to children
        assert_eq!(home.children.len(), 1);
        assert_eq!(home.children[0].rect(), &rect![0, 0, 600, 800]);
        let _test_type = home.child_mut(0).downcast_mut::<Filler>().unwrap();
    }

    #[test]
    #[allow(non_snake_case)]
    fn WHEN_createTopBarIsCalled_THEN_aTopBarIsAddedToChildren() {
        // WHEN create_top_bar is called
        let mut home = Home::new_empty(rect![0, 0, 600, 800]);
        let mut battery = Box::new(FakeBattery::new()) as Box<dyn Battery>;
        home.create_top_bar("%H:%M".to_string(), &mut Fonts::load_with_prefix("../../").unwrap(), &mut battery, true);
        // THEN a top bar is added to children
        assert_eq!(home.children.len(), 1);
        assert_eq!(home.children[0].rect(), &rect![0, 0, 600, 68]);
        let _test_type = home.child_mut(0).downcast_mut::<TopBar>().unwrap();
    }

    #[test]
    #[allow(non_snake_case)]
    fn WHEN_createMarkedForLaterIsCalled_THEN_aMarkedForLaterLabelIsAddedToChildren() {
        // WHEN create_marked_for_later is called
        let mut home = Home::new_empty(rect![0, 0, 600, 800]);
        home.create_marked_for_later(5, 67, 5, 1);
        // THEN a marked for later label is added to children
        assert_eq!(home.children.len(), 2);
        assert_eq!(home.children[0].rect(), &rect![0, 5, 600, 72]);
        let label = home.child_mut(0).downcast_mut::<TextLabel>().unwrap();
        assert_eq!(label.text, "Marked For Later");
        assert_eq!(home.children[1].rect(), &rect![0, 72, 600, 73]);
        let _test_type2 = home.child_mut(1).downcast_mut::<Filler>().unwrap();
    }

    #[test]
    #[allow(non_snake_case)]
    fn WHEN_createFaveSearchIsCalled_THEN_aFaveLabelIsAddedToChildren() {
        // WHEN create_marked_for_later is called
        let mut home = Home::new_empty(rect![0, 0, 600, 800]);
        home.create_fav_search(("Test Fave".to_string(), Url::parse("https://fakeo3.org/tags/super-fake").expect("Test URL")),
                               5, 67, 5, 1);
        // THEN a marked for later label is added to children
        assert_eq!(home.children.len(), 2);
        assert_eq!(home.children[0].rect(), &rect![0, 5, 600, 72]);
        let label = home.child_mut(0).downcast_mut::<TextLabel>().unwrap();
        assert_eq!(label.text, "Test Fave");
        assert_eq!(home.children[1].rect(), &rect![0, 72, 600, 73]);
        let _test_type2 = home.child_mut(1).downcast_mut::<Filler>().unwrap();
    }
}
