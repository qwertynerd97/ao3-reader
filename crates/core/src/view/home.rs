use crate::font::{Fonts, BOLD_STYLE};
use crate::view::{View, Event, Hub, Bus, RenderQueue, Align, ViewId, Id, ID_FEEDER, RenderData};
use crate::view::{MINI_BAR_HEIGHT, THICKNESS_MEDIUM, SMALL_PADDING, SMALL_BAR_HEIGHT};
use crate::context::Context;
use crate::unit::scale_by_dpi;
use crate::geom::{Rectangle, halves};
use crate::color::{BLACK, WHITE};
use crate::device::CURRENT_DEVICE;
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::view::textlabel::TextLabel;
use crate::view::filler::Filler;
use crate::font::LABEL_STYLE;
use crate::view::common::{locate, toggle_main_menu, toggle_battery_menu, toggle_clock_menu};
use super::top_bar::TopBar;

#[derive(Clone)]
pub struct Home {
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    id: Id,
    view_id: ViewId,
}

pub fn row_calc(rect: Rectangle) -> usize {
    let dpi = CURRENT_DEVICE.dpi;
    let small_height = scale_by_dpi(MINI_BAR_HEIGHT, dpi) as i32;
    let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
    ((rect.height() as i32 + thickness) / small_height) as usize
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

        let padding = scale_by_dpi(SMALL_PADDING, dpi) as i32;
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let (_small_thickness, big_thickness) = halves(thickness);
        let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;

        // TODO - let top bar determine it's own thickness
        home.create_top_bar(context, small_height + big_thickness);

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
            home.children.push(Box::new(sep) as Box<dyn View>);
            let label_rect = rect![x_min, start_y + thickness,
            x_max, start_y + row_height];
            let loc = faves[n].1.clone();

            let chapter = TextLabel::new(label_rect,
                                (*faves[n].0).to_string(),
                                Align::Left(padding), LABEL_STYLE, Event::LoadIndex(loc.to_string()));
                                home.children.push(Box::new(chapter) as Box<dyn View>);
            start_y += row_height;
        }

        let sep_rect = rect![x_min, start_y,
        x_max, start_y + thickness];
        let sep = Filler::new(sep_rect, BLACK);
        home.children.push(Box::new(sep) as Box<dyn View>);

        // Link to 'Marked for Later' view
        if context.client.logged_in {
            let label_rect = rect![x_min, start_y + thickness,
            x_max, start_y + row_height];
            let history = TextLabel::new(label_rect,
                "Marked For Later".to_string(),
                Align::Left(padding), BOLD_STYLE, Event::LoadHistory(super::works::HistoryView::MarkedForLater));
                home.children.push(Box::new(history) as Box<dyn View>);

        }

        rq.add(RenderData::new(home.id, rect, UpdateMode::Full));

        home
    }

    fn create_background(&mut self) {
        let bg = Filler::new(self.rect, WHITE);
        self.children.push(Box::new(bg) as Box<dyn View>);
    }

    fn create_top_bar(&mut self, context: &mut Context, bar_height: i32) {
        let top_bar = TopBar::new(rect![self.rect.min.x, self.rect.min.y,
                                        self.rect.max.x, self.rect.min.y + bar_height],
                                  Event::Toggle(ViewId::SearchBar),
                                  "Favorite Tags".to_string(),
                                  context);
        self.children.push(Box::new(top_bar) as Box<dyn View>);
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

    // TODO - make it so the top bar is not dependant on the frame buffer in context
    // #[test]
    // #[allow(non_snake_case)]
    // fn WHEN_createTopBarIsCalled_THEN_aTopBarIsAddedToChildren() {
    //     // WHEN create_top_bar is called
    //     let mut home = Home::new_empty(rect![0, 0, 600, 800]);
    //     home.create_top_bar();
    //     // THEN a top bar is added to children
    //     assert_eq!(home.children.len(), 1);
    //     assert_eq!(home.children[0].rect(), &rect![0, 0, 16, 16]);
    // }
}
