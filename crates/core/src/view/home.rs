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
    pub fn new(rect: Rectangle, rq: &mut RenderQueue, context: &mut Context) -> Home {
        let id = ID_FEEDER.next();
        let mut children = Vec::new();
        let dpi = CURRENT_DEVICE.dpi;

        let bg = Filler::new(rect, WHITE);
        children.push(Box::new(bg) as Box<dyn View>);

        let padding = scale_by_dpi(SMALL_PADDING, dpi) as i32;
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let (_small_thickness, big_thickness) = halves(thickness);
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
    fn handle_event(&mut self, evt: &Event, hub: &Hub, bus: &mut Bus, rq: &mut RenderQueue, context: &mut Context) -> bool {
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