use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::gesture::GestureEvent;
use crate::input::DeviceEvent;
use crate::view::{View, Event, Hub, Bus, Id, ID_FEEDER, RenderQueue, RenderData, ViewId, Align, THICKNESS_MEDIUM};
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

#[derive(Debug, Clone)]
pub struct TopBar {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    content_height: i32,
    border_thickness: i32
}

impl TopBar {
    pub fn new_empty(rect: Rectangle) -> TopBar {
        let id = ID_FEEDER.next();
        let children = Vec::new();
        let dpi = CURRENT_DEVICE.dpi;
        let border_thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let content_height = rect.height() as i32 - border_thickness;

        TopBar {
            id,
            rect,
            children,
            content_height,
            border_thickness
        }
    }
    pub fn new(rect: Rectangle, root_event: Event, title: String, context: &mut Context) -> TopBar {
        let mut top_bar = TopBar::new_empty(rect);

        // Left Align items
        top_bar.create_icon(root_event);

        // Right align items
        // TODO - remove dependency on context in order to follow principle of least access
        top_bar.create_clock(context.settings.time_format.clone(), &mut context.fonts);
        top_bar.create_battery(&mut context.battery);
        top_bar.create_frontlight(context.settings.frontlight);
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
        self.children.push(Box::new(root_icon) as Box<dyn View>);
    }

    fn create_clock(&mut self, format: String, fonts: &mut Fonts) {
        let mut clock_rect = rect![
            self.rect.max.x - (self.content_height * 4), self.rect.min.y,
            self.rect.max.x - (self.content_height * 3), self.rect.min.y + self.content_height];
        let clock_label = Clock::new(&mut clock_rect, format, fonts);
        self.children.push(Box::new(clock_label) as Box<dyn View>);
    }

    fn create_battery(&mut self, battery: &mut Box<dyn Battery>) {
        let capacity = battery.capacity().map_or(0.0, |v| v[0]);
        let status = battery.status().map_or(Status::Discharging, |v| v[0]);
        let battery_rect = rect![
            self.rect.max.x - (self.content_height * 3), self.rect.min.y,
            self.rect.max.x - (self.content_height * 2), self.rect.min.y + self.content_height];
        let battery_widget = BatteryWidget::new(battery_rect, capacity, status);
        self.children.push(Box::new(battery_widget) as Box<dyn View>);
    }

    fn create_frontlight(&mut self, has_frontlight: bool) {
        let name = if has_frontlight { "frontlight" } else { "frontlight-disabled" };
        let frontlight_rect = rect![
            self.rect.max.x - (self.content_height * 2), self.rect.min.y,
            self.rect.max.x - (self.content_height), self.rect.min.y + self.content_height];
        let frontlight_icon = Icon::new(name, frontlight_rect,
                                        Event::Show(ViewId::Frontlight));
        self.children.push(Box::new(frontlight_icon) as Box<dyn View>);
    }

    fn create_menu(&mut self) {
        let menu_rect = rect![
            self.rect.max.x-self.content_height, self.rect.min.y,
            self.rect.max.x, self.rect.min.y + self.content_height];
        let menu_icon = Icon::new("menu",
                                  menu_rect,
                                  Event::ToggleNear(ViewId::MainMenu, menu_rect));
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
        self.children.push(Box::new(title_label) as Box<dyn View>);
    }

    fn create_border(&mut self) {
        let border_rect = rect![
            self.rect.min.x, self.rect.max.y - self.border_thickness,
            self.rect.max.x, self.rect.max.y];
        let separator = Filler::new(border_rect, BLACK);
        self.children.push(Box::new(separator) as Box<dyn View>);
    }


    pub fn update_root_icon(&mut self, name: &str, rq: &mut RenderQueue) {
        let icon = self.child_mut(0).downcast_mut::<Icon>().unwrap();
        if icon.name != name {
            icon.name = name.to_string();
            rq.add(RenderData::new(icon.id(), *icon.rect(), UpdateMode::Gui));
        }
    }

    pub fn update_title_label(&mut self, title: &str, rq: &mut RenderQueue) {
        let title_label = self.children[1].as_mut().downcast_mut::<Label>().unwrap();
        title_label.update(title, rq);
    }

    pub fn update_frontlight_icon(&mut self, rq: &mut RenderQueue, context: &mut Context) {
        let name = if context.settings.frontlight { "frontlight" } else { "frontlight-disabled" };
        let icon = self.child_mut(4).downcast_mut::<Icon>().unwrap();
        icon.name = name.to_string();
        rq.add(RenderData::new(icon.id(), *icon.rect(), UpdateMode::Gui));
    }

    pub fn update_clock_label(&mut self, rq: &mut RenderQueue) {
        if let Some(clock_label) = self.children[2].downcast_mut::<Clock>() {
            clock_label.update(rq);
        }
    }

    pub fn update_battery_widget(&mut self, rq: &mut RenderQueue, context: &mut Context) {
        if let Some(battery_widget) = self.children[3].downcast_mut::<BatteryWidget>() {
            battery_widget.update(rq, context);
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
        let width = 600;
        let height = 67;
        let border_thickness = 1;
        let content_height = height - border_thickness;
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
        let width = 600;
        let height = 67;
        let border_thickness = 1;
        let content_height = height - border_thickness;
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
        let width = 600;
        let height = 67;
        let border_thickness = 1;
        let content_height = height - border_thickness;
        let mut top_bar = TopBar::new_empty(rect![0, 0, width, height]);
        top_bar.create_clock("%H:%M".to_string(), &mut Fonts::load_with_prefix("../../").unwrap());
        // THEN a right-aligned Clock widget is created
        assert_eq!(top_bar.children.len(), 1);
        assert_eq!(top_bar.children[0].rect(), &rect![338, 0, width-(content_height*3), content_height]);
        let _widget = top_bar.child_mut(0).downcast_mut::<Clock>().unwrap();
    }

    #[test]
    #[allow(non_snake_case)]
    fn WHEN_createBatteryIsCalled_THEN_aRightAlignedBatteryWidgetIsCreated() {
        // WHEN create_battery is called
        let width = 600;
        let height = 67;
        let border_thickness = 1;
        let content_height = height - border_thickness;
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
        let width = 600;
        let height = 67;
        let border_thickness = 1;
        let content_height = height - border_thickness;
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
        let width = 600;
        let height = 67;
        let border_thickness = 1;
        let content_height = height - border_thickness;
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
        let width = 600;
        let height = 67;
        let border_thickness = 1;
        let content_height = height - border_thickness;
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
        let width = 600;
        let height = 67;
        let border_thickness = 1;
        let content_height = height - border_thickness;
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
        let width = 600;
        let height = 67;
        let border_thickness = 1;
        let content_height = height - border_thickness;
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
        let width = 600;
        let height = 67;
        let border_thickness = 1;
        let mut top_bar = TopBar::new_empty(rect![0, 0, width, height]);
        top_bar.create_border();
        // THEN a line is created
        assert_eq!(top_bar.children.len(), 1);
        assert_eq!(top_bar.children[0].rect(), &rect![0, height-border_thickness, width, height]);
        let _title = top_bar.child_mut(0).downcast_mut::<Filler>().unwrap();
    }
}
