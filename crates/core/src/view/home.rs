use url::Url;
use crate::font::{Fonts, BOLD_STYLE};
use crate::view::{View, Event, Hub, Bus, RenderQueue, Align, ViewId, Id, ID_FEEDER, RenderData};
use crate::view::{MINI_BAR_HEIGHT, THICKNESS_MEDIUM, SMALL_PADDING, SMALL_BAR_HEIGHT, BIG_BAR_HEIGHT};
use crate::context::Context;
use crate::unit::scale_by_dpi;
use crate::geom::Rectangle;
use crate::color::{BLACK, WHITE};
use crate::device::CURRENT_DEVICE;
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::view::textlabel::TextLabel;
use crate::view::filler::Filler;
use crate::font::LABEL_STYLE;
use crate::view::common::{locate, toggle_main_menu, toggle_battery_menu, toggle_clock_menu, rlocate};
use super::top_bar::TopBar;
use super::bottom_bar::BottomBar;
use crate::view::keyboard::Keyboard;
use crate::view::search_bar::SearchBar;
use crate::view::notification::Notification;
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
        let mut children = Vec::new();
        let dpi = CURRENT_DEVICE.dpi;

        let bg = Filler::new(rect, WHITE);
        children.push(Box::new(bg) as Box<dyn View>);

        let padding = scale_by_dpi(SMALL_PADDING, dpi) as i32;
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let (small_thickness, big_thickness) = halves(thickness);
        let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;

        let top_bar = TopBar::new(rect![rect.min.x, rect.min.y,
                                        rect.max.x, rect.min.y + small_height + big_thickness],
                                  Event::Toggle(ViewId::SearchBar),
                                  "Favorite Tags".to_string(),
                                  context);
        children.push(Box::new(top_bar) as Box<dyn View>);


        let entries_rect =  rect![rect.min.x, rect.min.y + small_height + big_thickness,
        rect.max.x, rect.max.y];
        let rows = row_calc(entries_rect);
        let row_height = rect.height() as i32 / rows as i32;
        let faves = &context.settings.ao3.faves;
        let end = if faves.len() > rows {rows} else {faves.len()};

        let x_min = rect.min.x; // + padding;
        let x_max = rect.max.x; // - padding;
        let mut start_y = rect.min.y + small_height + big_thickness;

        for n in 0..end {
            let sep_rect = rect![x_min, start_y,
            x_max, start_y + thickness];
            let sep = Filler::new(sep_rect, BLACK);
            children.push(Box::new(sep) as Box<dyn View>);
            let label_rect = rect![x_min, start_y + thickness,
            x_max, start_y + row_height];
            let loc = faves[n].1.clone();

            let chapter = TextLabel::new(label_rect,
                                (*faves[n].0).to_string(),
                                Align::Left(padding), LABEL_STYLE, Event::LoadIndex(loc.to_string()));
                                children.push(Box::new(chapter) as Box<dyn View>);
            start_y += row_height;
        }

        let sep_rect = rect![x_min, start_y,
        x_max, start_y + thickness];
        let sep = Filler::new(sep_rect, BLACK);
        children.push(Box::new(sep) as Box<dyn View>);
        
        // Link to 'Marked for Later' view
        if context.client.logged_in {
            let label_rect = rect![x_min, start_y + thickness,
            x_max, start_y + row_height];
            let history = TextLabel::new(label_rect,
                "Marked For Later".to_string(),
                Align::Left(padding), BOLD_STYLE, Event::LoadHistory(super::works::HistoryView::MarkedForLater));
                children.push(Box::new(history) as Box<dyn View>);

        }


        rq.add(RenderData::new(id, rect, UpdateMode::Full));
    
        Home {
            rect,
            children,
            id,
            view_id: ViewId::Home,
            query: None,
            focus: None,
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

        let shelf_index = home.children.len() - 1;

        let separator = Filler::new(rect![rect.min.x, rect.max.y - small_height - small_thickness,
            rect.max.x, rect.max.y - small_height + big_thickness],
      BLACK);
        home.children.push(Box::new(separator) as Box<dyn View>);

        let bottom_bar = BottomBar::new(rect![rect.min.x, rect.max.y - small_height + big_thickness,
            rect.max.x, rect.max.y],
            1,
            1,
            "test",
            2,
            false);
            home.children.push(Box::new(bottom_bar) as Box<dyn View>);

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

    fn toggle_search_bar(&mut self, enable: Option<bool>, update: bool, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        let dpi = CURRENT_DEVICE.dpi;
        let (small_height, big_height) = (scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32,
                                          scale_by_dpi(BIG_BAR_HEIGHT, dpi) as i32);
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let delta_y = small_height;
        let search_visible: bool;
        let mut has_keyboard = false;

        if let Some(index) = rlocate::<SearchBar>(self) {
            if let Some(true) = enable {
                return;
            }

            if let Some(ViewId::HomeSearchInput) = self.focus {
                self.toggle_keyboard(false, false, Some(ViewId::HomeSearchInput), hub, rq, context);
            }

            // Remove the search bar and its separator.
            self.children.drain(index - 1 ..= index);

            // Move the shelf's bottom edge.
            self.children[self.shelf_index].rect_mut().max.y += delta_y;

            self.query = None;
            search_visible = false;
        } else {
            if let Some(false) = enable {
                return;
            }

            let sp_rect = *self.child(self.shelf_index+1).rect() - pt!(0, delta_y);
            let search_bar = SearchBar::new(rect![self.rect.min.x, sp_rect.max.y,
                                                  self.rect.max.x,
                                                  sp_rect.max.y + delta_y - thickness],
                                            ViewId::HomeSearchInput,
                                            "Search Ao3",
                                            "", context);
            self.children.insert(self.shelf_index+1, Box::new(search_bar) as Box<dyn View>);

            let separator = Filler::new(sp_rect, BLACK);
            self.children.insert(self.shelf_index+1, Box::new(separator) as Box<dyn View>);

            // Move the shelf's bottom edge.
            self.children[self.shelf_index].rect_mut().max.y -= delta_y;

            if self.query.is_none() {
                if rlocate::<Keyboard>(self).is_none() {
                    self.toggle_keyboard(true, false, Some(ViewId::HomeSearchInput), hub, rq, context);
                    has_keyboard = true;
                }

                hub.send(Event::Focus(Some(ViewId::HomeSearchInput))).ok();
            }

            search_visible = true;
        }

        if update {
            if !search_visible {
                println!("running search?");
                rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
            }


            if search_visible {
                rq.add(RenderData::new(self.child(self.shelf_index-1).id(), *self.child(self.shelf_index-1).rect(), UpdateMode::Partial));
                let mut rect = *self.child(self.shelf_index).rect();
                rect.max.y = self.child(self.shelf_index+1).rect().min.y;
                // Render the part of the shelf that isn't covered.
                rq.add(RenderData::new(self.child(self.shelf_index).id(), rect, UpdateMode::Partial));
                // Render the views on top of the shelf.
                rect.min.y = rect.max.y;
                let end_index = self.shelf_index + if has_keyboard { 4 } else { 2 };
                rect.max.y = self.child(end_index).rect().max.y;
                rq.add(RenderData::expose(rect, UpdateMode::Partial));
            } else {
                for i in self.shelf_index - 1 ..= self.shelf_index + 1 {
                    if i == self.shelf_index {
                        // self.update_shelf(true, hub, rq, context);
                        continue;
                    }
                    rq.add(RenderData::new(self.child(i).id(), *self.child(i).rect(), UpdateMode::Partial));
                }
            }

            // self.update_bottom_bar(rq);
        }
    }

    fn toggle_keyboard(&mut self, enable: bool, update: bool, id: Option<ViewId>, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        let dpi = CURRENT_DEVICE.dpi;
        let (small_height, big_height) = (scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32,
                                          scale_by_dpi(BIG_BAR_HEIGHT, dpi) as i32);
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let (small_thickness, big_thickness) = halves(thickness);
        let has_search_bar = self.children[self.shelf_index+2].is::<SearchBar>();

        if let Some(index) = rlocate::<Keyboard>(self) {
            if enable {
                return;
            }

            let y_min = self.child(self.shelf_index+1).rect().min.y;
            let mut rect = *self.child(index).rect();
            rect.absorb(self.child(index-1).rect());

            self.children.drain(index - 1 ..= index);

            let delta_y = rect.height() as i32;

            if has_search_bar {
                for i in self.shelf_index+1..=self.shelf_index+2 {
                    let shifted_rect = *self.child(i).rect() + pt!(0, delta_y);
                    self.child_mut(i).resize(shifted_rect, hub, rq, context);
                }
            }

            context.kb_rect = Rectangle::default();
            hub.send(Event::Focus(None)).ok();
            if update {
                let rect = rect![self.rect.min.x, y_min,
                                 self.rect.max.x, y_min + delta_y];
                rq.add(RenderData::expose(rect, UpdateMode::Gui));
            }
        } else {
            if !enable {
                return;
            }

            let index = rlocate::<BottomBar>(self).unwrap() - 1;
            let mut kb_rect = rect![self.rect.min.x,
                                    self.rect.max.y - (small_height + 3 * big_height) as i32 + big_thickness,
                                    self.rect.max.x,
                                    self.rect.max.y - small_height - small_thickness];

            let number = matches!(id, Some(ViewId::GoToPageInput));
            let keyboard = Keyboard::new(&mut kb_rect, number, context);
            self.children.insert(index, Box::new(keyboard) as Box<dyn View>);

            let separator = Filler::new(rect![self.rect.min.x, kb_rect.min.y - thickness,
                                              self.rect.max.x, kb_rect.min.y],
                                        BLACK);
            self.children.insert(index, Box::new(separator) as Box<dyn View>);

            let delta_y = kb_rect.height() as i32 + thickness;

            if has_search_bar {
                for i in self.shelf_index+1..=self.shelf_index+2 {
                    let shifted_rect = *self.child(i).rect() + pt!(0, -delta_y);
                    self.child_mut(i).resize(shifted_rect, hub, rq, context);
                }
            }
        }

        if update {
            if enable {
                if has_search_bar {
                    for i in self.shelf_index+1..=self.shelf_index+4 {
                        let update_mode = if (i - self.shelf_index) == 1 { UpdateMode::Partial } else { UpdateMode::Gui };
                        rq.add(RenderData::new(self.child(i).id(), *self.child(i).rect(), update_mode));
                    }
                } else {
                    for i in self.shelf_index+1..=self.shelf_index+2 {
                        rq.add(RenderData::new(self.child(i).id(), *self.child(i).rect(), UpdateMode::Gui));
                    }
                }
            } else if has_search_bar {
                for i in self.shelf_index+1..=self.shelf_index+2 {
                    rq.add(RenderData::new(self.child(i).id(), *self.child(i).rect(), UpdateMode::Gui));
                }
            }
        }
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
            Event::Toggle(ViewId::SearchBar) => {
                self.toggle_search_bar(None, true, hub, rq, context);
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
            Event::Close(ViewId::SearchBar) => {
                self.toggle_search_bar(Some(false), true, hub, rq, context);
                true
            },
            Event::Submit(ViewId::HomeSearchInput, ref text) => {
                self.query = Some(text.to_string());
                if self.query.is_some() {
                    self.toggle_keyboard(false, false, None, hub, rq, context);
                    self.toggle_search_bar(Some(false), false, hub, rq, context);
                    rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                    hub.send(Event::LoadSearch(text.to_string())).ok();
                } else {
                    let notif = Notification::new("Invalid search query.".to_string(),
                                                  hub, rq, context);
                    self.children.push(Box::new(notif) as Box<dyn View>);
                }
                true
            },
            Event::Focus(v) => {
                if self.focus != v {
                    self.focus = v;
                    if v.is_some() {
                        self.toggle_keyboard(true, true, v, hub, rq, context);
                    }
                }
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
