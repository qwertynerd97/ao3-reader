use std::collections::HashMap;

use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::gesture::GestureEvent;
use crate::input::DeviceEvent;
use crate::view::{View, Event, Hub, Bus, Id, ID_FEEDER, RenderQueue, RenderData, ViewId, Align, SMALL_BAR_HEIGHT, THICKNESS_MEDIUM};
use crate::view::icon::Icon;
use crate::view::clock::Clock;
use crate::view::battery::Battery as BatteryWidget;
use crate::view::label::Label;
use crate::geom::{Rectangle};
use crate::font::Fonts;
use crate::view::filler::Filler;
use crate::unit::scale_by_dpi;
use crate::color::BLACK;
use crate::device::CURRENT_DEVICE;
use crate::context::Context;
use crate::battery::{Battery, Status};

// Children names for lookup
pub const MENU_ACTION: &str = "menu_action";
pub const CLOCK: &str = "clock";
pub const BATTERY: &str = "battery";
pub const FRONTLIGHT: &str = "frontlight";
pub const MENU: &str = "menu";
pub const TITLE: &str = "title";
pub const BORDER: &str = "border";

#[derive(Debug, Clone)]
pub struct TopBar {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    content_height: i32,
    border_thickness: i32,
    children_lookup: HashMap<String, usize>
}

impl TopBar {
    pub fn new_empty(parent_rect: Rectangle) -> TopBar {
        let dpi = CURRENT_DEVICE.dpi;
        let content_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;
        let border_thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;

        TopBar {
            id: ID_FEEDER.next(),
            rect: rect![
                parent_rect.min.x, parent_rect.min.y,
                parent_rect.max.x, parent_rect.min.y + content_height + border_thickness],
            children: Vec::new(),
            content_height,
            border_thickness,
            children_lookup: HashMap::new()
        }
    }
    pub fn new(parent_rect: Rectangle, root_event: Event, title: String,
               format: String, fonts: &mut Fonts, battery: &mut Box<dyn Battery>, frontlight: bool) -> TopBar {
        let mut top_bar = TopBar::new_empty(parent_rect);

        // Left aligned items
        top_bar.create_icon(root_event);

        // Right aligned items
        top_bar.create_clock(format, fonts);
        top_bar.create_battery(battery);
        top_bar.create_frontlight(frontlight);
        top_bar.create_menu();

        // Note: title needs to be declared last, because it takes up the remaining space
        top_bar.create_title(title);

        top_bar.create_border();
        top_bar
    }

    fn create_icon(&mut self, root_event: Event) {
        let icon_name = match root_event {
            Event::Back => "back",
            _ => "search",
        };

        let root_icon = Icon::new(icon_name,
                                  rect![self.rect.min, self.rect.min + self.content_height],
                                  root_event);
        self.children_lookup.insert(MENU_ACTION.to_string(), self.children.len());
        self.children.push(Box::new(root_icon) as Box<dyn View>);
    }

    fn create_clock(&mut self, format: String, fonts: &mut Fonts) {
        let mut clock_rect = rect![
            self.rect.max.x - (self.content_height * 4), self.rect.min.y,
            self.rect.max.x - (self.content_height * 3), self.rect.min.y + self.content_height];
        let clock_label = Clock::new(&mut clock_rect, format, fonts);

        self.children_lookup.insert(CLOCK.to_string(), self.children.len());
        self.children.push(Box::new(clock_label) as Box<dyn View>);
    }

    fn create_battery(&mut self, battery: &mut Box<dyn Battery>) {
        let capacity = battery.capacity().map_or(0.0, |v| v[0]);
        let status = battery.status().map_or(Status::Discharging, |v| v[0]);
        let battery_rect = rect![
            self.rect.max.x - (self.content_height * 3), self.rect.min.y,
            self.rect.max.x - (self.content_height * 2), self.rect.min.y + self.content_height];
        let battery_widget = BatteryWidget::new(battery_rect, capacity, status);

        self.children_lookup.insert(BATTERY.to_string(), self.children.len());
        self.children.push(Box::new(battery_widget) as Box<dyn View>);
    }

    fn create_frontlight(&mut self, has_frontlight: bool) {
        let name = if has_frontlight { "frontlight" } else { "frontlight-disabled" };
        let frontlight_rect = rect![
            self.rect.max.x - (self.content_height * 2), self.rect.min.y,
            self.rect.max.x - (self.content_height), self.rect.min.y + self.content_height];
        let frontlight_icon = Icon::new(name, frontlight_rect,
                                        Event::Show(ViewId::Frontlight));

        self.children_lookup.insert(FRONTLIGHT.to_string(), self.children.len());
        self.children.push(Box::new(frontlight_icon) as Box<dyn View>);
    }

    fn create_menu(&mut self) {
        let menu_rect = rect![
            self.rect.max.x-self.content_height, self.rect.min.y,
            self.rect.max.x, self.rect.min.y + self.content_height];
        let menu_icon = Icon::new("menu",
                                  menu_rect,
                                  Event::ToggleNear(ViewId::MainMenu, menu_rect));

        self.children_lookup.insert(MENU.to_string(), self.children.len());
        self.children.push(Box::new(menu_icon) as Box<dyn View>);
    }

    fn create_title(&mut self, title: String) {
        // We want the title to take up all the remaining space in the toolbar
        // so we need to calculate the size of all the existing children
        let used_width = self.children.iter().fold(0, |width, child| width + child.rect().width() as i32);
        let title_rect = rect![
            self.rect.min.x + self.content_height, self.rect.min.y,
            self.rect.max.x - used_width, self.rect.min.y + self.content_height];
        let title_label = Label::new(title_rect, title, Align::Center)
                                .event(Some(Event::ToggleNear(ViewId::TitleMenu, title_rect)));

        self.children_lookup.insert(TITLE.to_string(), self.children.len());
        self.children.push(Box::new(title_label) as Box<dyn View>);
    }

    fn create_border(&mut self) {
        let border_rect = rect![
            self.rect.min.x, self.rect.max.y - self.border_thickness,
            self.rect.max.x, self.rect.max.y];
        let separator = Filler::new(border_rect, BLACK);

        self.children_lookup.insert(BORDER.to_string(), self.children.len());
        self.children.push(Box::new(separator) as Box<dyn View>);
    }


    pub fn update_root_icon(&mut self, name: &str, rq: &mut RenderQueue) {
        match self.children_lookup.get(MENU_ACTION) {
            Some(index) => {
                let icon = self.child_mut(*index).downcast_mut::<Icon>().unwrap();
                if icon.name != name {
                    icon.name = name.to_string();
                    rq.add(RenderData::new(icon.id(), *icon.rect(), UpdateMode::Gui));
                }
            },
            None => ()
        }
    }

    pub fn update_title_label(&mut self, title: &str, rq: &mut RenderQueue) {
        match self.children_lookup.get(TITLE) {
            Some(index) => {
                let title_label = self.children[*index].as_mut().downcast_mut::<Label>().unwrap();
                title_label.update(title, rq);
            },
            None => ()
        }
    }

    pub fn update_frontlight_icon(&mut self, rq: &mut RenderQueue, context: &mut Context) {
        self.context_free_update_frontlight_icon(rq, context.settings.frontlight);
    }

    pub fn context_free_update_frontlight_icon(&mut self, rq: &mut RenderQueue, frontlight: bool) {
        match self.children_lookup.get(FRONTLIGHT) {
            Some(index) => {
                let name = if frontlight { "frontlight" } else { "frontlight-disabled" };
                let icon = self.child_mut(*index).downcast_mut::<Icon>().unwrap();
                icon.name = name.to_string();
                rq.add(RenderData::new(icon.id(), *icon.rect(), UpdateMode::Gui));
            },
            None => ()
        }
    }

    pub fn update_clock_label(&mut self, rq: &mut RenderQueue) {
        match self.children_lookup.get(CLOCK) {
            Some(index) => {
                if let Some(clock_label) = self.children[*index].downcast_mut::<Clock>() {
                    clock_label.update(rq);
                }
            },
            None => ()
        }
    }

    pub fn update_battery_widget(&mut self, rq: &mut RenderQueue, context: &mut Context) {
        match self.children_lookup.get(BATTERY) {
            Some(index) => {
                if let Some(battery_widget) = self.children[*index].downcast_mut::<BatteryWidget>() {
                    battery_widget.update(rq, context);
                }
            },
            None => ()
        }
    }

    pub fn reseed(&mut self, rq: &mut RenderQueue, context: &mut Context) {
        self.update_frontlight_icon(rq, context);
        self.update_clock_label(rq);
        self.update_battery_widget(rq, context);
    }
}

impl View for TopBar {
    fn handle_event(&mut self, evt: &Event, _hub: &Hub, _bus: &mut Bus, _rq: &mut RenderQueue, _context: &mut Context) -> bool {
        match *evt {
            Event::Gesture(GestureEvent::Tap(center)) |
            Event::Gesture(GestureEvent::HoldFingerShort(center, ..)) if self.rect.includes(center) => true,
            Event::Gesture(GestureEvent::Swipe { start, end, .. }) if self.rect.includes(start) && self.rect.includes(end) => true,
            Event::Device(DeviceEvent::Finger { position, .. }) if self.rect.includes(position) => true,
            _ => false,
        }
    }

    fn render(&self, _fb: &mut dyn Framebuffer, _rect: Rectangle, _fonts: &mut Fonts) {
    }

    fn resize(&mut self, rect: Rectangle, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        let dpi = CURRENT_DEVICE.dpi;
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let side = rect.height() as i32 - thickness;
        self.children[0].resize(rect![rect.min, rect.min+side], hub, rq, context);
        let clock_width = self.children[2].rect().width() as i32;
        let clock_rect = rect![rect.max - pt!(3*side + clock_width, side),
                               rect.max - pt!(3*side, 0)];
        self.children[1].resize(rect![rect.min.x + side,
                                      rect.min.y,
                                      clock_rect.min.x,
                                      rect.max.y],
                                hub, rq, context);
        self.children[2].resize(clock_rect, hub, rq, context);
        self.children[3].resize(rect![rect.max - pt!(3*side, side),
                                      rect.max - pt!(2*side, 0)],
                                hub, rq, context);
        self.children[4].resize(rect![rect.max - pt!(2*side, side),
                                      rect.max - pt!(side, 0)],
                                hub, rq, context);
        self.children[5].resize(rect![rect.max-side, rect.max],
                                hub, rq, context);
        self.children[6].resize(rect![rect.min.x, rect.max.y - thickness,
            rect.max.x, rect.max.y], hub, rq, context);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battery::FakeBattery;

    #[test]
    #[allow(non_snake_case)]
    fn WHEN_createIconIsCalledWithSearchEvent_THEN_aLeftAlignedSearchIconIsCreated() {
        // WHEN create_icon is called with Search Event
        let width = 300;
        let height = 600;
        let content_height = 67;
        let mut top_bar = TopBar::new_empty(rect![0, 0, width, height]);
        top_bar.create_icon(Event::Toggle(ViewId::SearchBar));
        // THEN a left-aligned Search Icon is created
        assert_eq!(top_bar.children.len(), 1);
        assert_eq!(top_bar.children[0].rect(), &rect![0, 0, content_height, content_height]);
        let icon = top_bar.child_mut(0).downcast_mut::<Icon>().unwrap();
        assert_eq!(icon.name, "search");
    }

    #[test]
    #[allow(non_snake_case)]
    fn WHEN_createIconIsCalledWithBackEvent_THEN_aLeftAlignedBackIconIsCreated() {
        // WHEN create_icon is called with Back Event
        let width = 300;
        let height = 600;
        let content_height = 67;
        let mut top_bar = TopBar::new_empty(rect![0, 0, width, height]);
        top_bar.create_icon(Event::Back);
        // THEN a left-aligned Back Icon is created
        assert_eq!(top_bar.children.len(), 1);
        assert_eq!(top_bar.children[0].rect(), &rect![0, 0, content_height, content_height]);
        let icon = top_bar.child_mut(0).downcast_mut::<Icon>().unwrap();
        assert_eq!(icon.name, "back");
    }

    #[test]
    #[allow(non_snake_case)]
    fn WHEN_createClockIsCalled_THEN_aRightAlignedClockWidgetIsCreated() {
        // WHEN create_clock is called
        let width = 300;
        let height = 600;
        let content_height = 67;
        let mut top_bar = TopBar::new_empty(rect![0, 0, width, height]);
        top_bar.create_clock("%H:%M".to_string(), &mut Fonts::load_with_prefix("../../").unwrap());
        // THEN a right-aligned Clock widget is created
        assert_eq!(top_bar.children.len(), 1);
        assert_eq!(top_bar.children[0].rect(), &rect![35, 0, width-(content_height*3), content_height]);
        let _widget = top_bar.child_mut(0).downcast_mut::<Clock>().unwrap();
    }

    #[test]
    #[allow(non_snake_case)]
    fn WHEN_createBatteryIsCalled_THEN_aRightAlignedBatteryWidgetIsCreated() {
        // WHEN create_battery is called
        let width = 300;
        let height = 600;
        let content_height = 67;
        let mut top_bar = TopBar::new_empty(rect![0, 0, width, height]);
        let mut battery = Box::new(FakeBattery::new()) as Box<dyn Battery>;
        top_bar.create_battery(&mut battery);
        // THEN a right-aligned Battery widget is created
        assert_eq!(top_bar.children.len(), 1);
        assert_eq!(top_bar.children[0].rect(), &rect![width-((content_height)*3), 0, width-(content_height*2), content_height]);
        let _widget = top_bar.child_mut(0).downcast_mut::<BatteryWidget>().unwrap();
    }

    #[test]
    #[allow(non_snake_case)]
    fn WHEN_createFrontLightIsCalledWithLight_THEN_aRightAlignedFrontlightIconIsCreated() {
        // WHEN create_frontlight is called
        let width = 300;
        let height = 600;
        let content_height = 67;
        let mut top_bar = TopBar::new_empty(rect![0, 0, width, height]);
        top_bar.create_frontlight(true);
        // THEN a right-aligned Frontlight Icon is created
        assert_eq!(top_bar.children.len(), 1);
        assert_eq!(top_bar.children[0].rect(), &rect![width-((content_height)*2), 0, width-content_height, content_height]);
        let icon = top_bar.child_mut(0).downcast_mut::<Icon>().unwrap();
        assert_eq!(icon.name, "frontlight");
    }

    #[test]
    #[allow(non_snake_case)]
    fn WHEN_createFrontLightIsCalledWithoutLight_THEN_aRightAlignedDisabledFrontlightIconIsCreated() {
        // WHEN create_frontlight is called
        let width = 300;
        let height = 600;
        let content_height = 67;
        let mut top_bar = TopBar::new_empty(rect![0, 0, width, height]);
        top_bar.create_frontlight(false);
        // THEN a right-aligned Disabled Frontlight Icon is created
        assert_eq!(top_bar.children.len(), 1);
        assert_eq!(top_bar.children[0].rect(), &rect![width-((content_height)*2), 0, width-content_height, content_height]);
        let icon = top_bar.child_mut(0).downcast_mut::<Icon>().unwrap();
        assert_eq!(icon.name, "frontlight-disabled");
    }

    #[test]
    #[allow(non_snake_case)]
    fn WHEN_createMenuIsCalled_THEN_aRightAlignedMenuIconIsCreated() {
        // WHEN create_menu is called
        let width = 300;
        let height = 600;
        let content_height = 67;
        let mut top_bar = TopBar::new_empty(rect![0, 0, width, height]);
        top_bar.create_menu();
        // THEN a right-aligned Menu Icon is created
        assert_eq!(top_bar.children.len(), 1);
        assert_eq!(top_bar.children[0].rect(), &rect![width-(content_height), 0, width, content_height]);
        let icon = top_bar.child_mut(0).downcast_mut::<Icon>().unwrap();
        assert_eq!(icon.name, "menu");
    }

    #[test]
    #[allow(non_snake_case)]
    fn WHEN_createTitleIsCalled_THEN_aCenteredTitleIsCreated() {
        // WHEN create_title is called
        let width = 300;
        let height = 600;
        let content_height = 67;
        let mut top_bar = TopBar::new_empty(rect![0, 0, width, height]);
        top_bar.create_title("Test Title".to_string());
        // THEN a centered Title is created
        assert_eq!(top_bar.children.len(), 1);
        assert_eq!(top_bar.children[0].rect(), &rect![content_height, 0, width, content_height]);
        let _title = top_bar.child_mut(0).downcast_mut::<Label>().unwrap();
    }

    #[test]
    #[allow(non_snake_case)]
    fn GIVEN_otherChildrenHaveBeenCreated_WHEN_createTitleIsCalled_THEN_aSmallerCenteredTitleIsCreated() {
        // GIVEN other children have been created
        let width = 300;
        let height = 600;
        let content_height = 67;
        let mut top_bar = TopBar::new_empty(rect![0, 0, width, height]);
        top_bar.create_menu();
        // WHEN create_title is called
        top_bar.create_title("Test Title".to_string());
        // THEN a smaller centered Title is created
        assert_eq!(top_bar.children.len(), 2);
        assert_eq!(top_bar.children[1].rect(), &rect![content_height, 0, width-content_height, content_height]);
        let _title = top_bar.child_mut(1).downcast_mut::<Label>().unwrap();
    }

    #[test]
    #[allow(non_snake_case)]
    fn WHEN_createBorderIsCalled_THEN_aLineIsCreated() {
        // WHEN create_border is called
        let width = 300;
        let height = 600;
        let content_height = 67;
        let border_thickness = 1;
        let mut top_bar = TopBar::new_empty(rect![0, 0, width, height]);
        top_bar.create_border();
        // THEN a line is created
        assert_eq!(top_bar.children.len(), 1);
        assert_eq!(top_bar.children[0].rect(), &rect![0, content_height, width, content_height+border_thickness]);
        let _title = top_bar.child_mut(0).downcast_mut::<Filler>().unwrap();
    }

    #[test]
    #[allow(non_snake_case)]
    fn WHEN_topbarNewIsCalled_THEN_aTopBarWithTheStandardChildrenIsCreated() {
        // WHEN TopBar::new() is called
        let mut battery = Box::new(FakeBattery::new()) as Box<dyn Battery>;
        let top_bar = TopBar::new(rect![0, 0, 600, 800], Event::Back, "Test Title".to_string(),
                                 "%H:%M".to_string(), &mut Fonts::load_with_prefix("../../").unwrap(),
                                  &mut battery, true);
        // THEN a top bar with the standard children is called
        assert_eq!(top_bar.children_lookup, HashMap::from([
            (MENU_ACTION.to_string(), 0usize),
            (CLOCK.to_string(), 1usize),
            (BATTERY.to_string(), 2usize),
            (FRONTLIGHT.to_string(), 3usize),
            (MENU.to_string(), 4usize),
            (TITLE.to_string(), 5usize),
            (BORDER.to_string(), 6usize)
        ]));
    }

    #[test]
    #[allow(non_snake_case)]
    fn GIVEN_aStandardTopBar_WHEN_updateFrontlightIconIsCalled_THEN_noneOfTheChildrenAreChanged() {
        // GIVEN a standard TopBar
        let mut battery = Box::new(FakeBattery::new()) as Box<dyn Battery>;
        let mut top_bar = TopBar::new(rect![0, 0, 600, 800], Event::Back, "Test Title".to_string(),
                                 "%H:%M".to_string(), &mut Fonts::load_with_prefix("../../").unwrap(),
                                  &mut battery, true);
        let mut rq = RenderQueue::new();
        // WHEN update_frontlight_icon is called
        top_bar.context_free_update_frontlight_icon(&mut rq, true);
        // THEN none of the children are changed
        assert_eq!(top_bar.children_lookup, HashMap::from([
            (MENU_ACTION.to_string(), 0usize),
            (CLOCK.to_string(), 1usize),
            (BATTERY.to_string(), 2usize),
            (FRONTLIGHT.to_string(), 3usize),
            (MENU.to_string(), 4usize),
            (TITLE.to_string(), 5usize),
            (BORDER.to_string(), 6usize)
        ]));
        let menu_action = top_bar.child_mut(0).downcast_mut::<Icon>().unwrap();
        assert_eq!(menu_action.name, "back");
        let _clock = top_bar.child_mut(1).downcast_mut::<Clock>().unwrap();
        let _battery = top_bar.child_mut(2).downcast_mut::<BatteryWidget>().unwrap();
        let frontlight = top_bar.child_mut(3).downcast_mut::<Icon>().unwrap();
        assert_eq!(frontlight.name, "frontlight");
        let menu = top_bar.child_mut(4).downcast_mut::<Icon>().unwrap();
        assert_eq!(menu.name, "menu");
    }
}
