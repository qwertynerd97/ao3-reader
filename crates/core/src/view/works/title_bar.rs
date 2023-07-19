use crate::view::{View, Event, Hub, Bus, Id, ID_FEEDER, RenderQueue, RenderData, THICKNESS_MEDIUM, Align};
use crate::context::Context;
use crate::font::{Fonts, font_from_style, NORMAL_STYLE};
use crate::geom::Rectangle;
use crate::view::icon::Icon;
use crate::view::label::Label;
use url::Url;
use crate::device::CURRENT_DEVICE;
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::color::{BLACK, WHITE};
use crate::view::filler::Filler;
use crate::unit::scale_by_dpi;

#[derive(Debug, Clone)]
pub struct TitleBar {
    id: Id,
    pub rect: Rectangle,
    children: Vec<Box<dyn View>>,
    title: String,
    url: Url,
    fave: bool
}

impl TitleBar {
    pub fn new(rect: Rectangle, title: String, url: Url, context: &Context) -> TitleBar {
        let fave = context.settings.ao3.url_in_faves(url.clone());
        let dpi = CURRENT_DEVICE.dpi;
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;

        let mut children = Vec::new();
        let side = rect.height() as i32 - thickness;

        let root_icon = Icon::new("back",
                                  rect![rect.min, rect.min+side],
                                  Event::Back);
        children.push(Box::new(root_icon) as Box<dyn View>);

        let title_label = Label::new(rect![rect.min.x + side, rect.min.y, rect.max.x - side, rect.max.y - thickness],
        title.clone(), Align::Left(0));
        children.push(Box::new(title_label) as Box<dyn View>);

        let icon = if fave { "star" } else { "star-outline" };
        let fave_icon = Icon::new(icon,
                rect![rect.max.x - side, rect.max.y - side,
                    rect.max.x, rect.max.y - thickness],
                Event::ToggleFaveIcon);
        children.push(Box::new(fave_icon) as Box<dyn View>);

        let separator = Filler::new(rect![rect.min.x, rect.max.y - thickness,
            rect.max.x, rect.max.y],
            BLACK);
        children.push(Box::new(separator) as Box<dyn View>);

        TitleBar {
            id: ID_FEEDER.next(),
            rect,
            children,
            title,
            url,
            fave
        }
    }

    fn update_icon(&mut self, rq: &mut RenderQueue) {
        let index = self.len() - 2;
        let icon_rect = *self.children[index].rect();
        let icon = if self.fave { "star" } else { "star-outline" };
        let fave_icon = Icon::new(icon,
                icon_rect,
                Event::ToggleFaveIcon);
        self.children[index] = Box::new(fave_icon) as Box<dyn View>;
        rq.add(RenderData::new(self.id, icon_rect, UpdateMode::Gui));
    }

}


impl View for TitleBar {
    fn handle_event(&mut self, evt: &Event, hub: &Hub, _bus: &mut Bus, rq: &mut RenderQueue, _context: &mut Context) -> bool {
        match *evt {
            Event::ToggleFaveIcon  => {
                self.fave = !self.fave;
                self.update_icon(rq);
                hub.send(Event::ToggleFave(self.title.clone(), self.url.clone())).ok();
                true
            },
            _ => false,
        }
    }

    fn render(&self, fb: &mut dyn Framebuffer, _rect: Rectangle, fonts: &mut Fonts) {
        println!("trying to render title");
        let dpi = CURRENT_DEVICE.dpi;
        let font = font_from_style(fonts, &NORMAL_STYLE, dpi);
        let side = self.rect.height() as i32;
        let padding = font.em() as i32 / 2;
        let max_width = self.rect.width().saturating_sub(2 * padding as u32) as i32 - side;
        println!("max width is {}", max_width);
        let mut plan = font.plan(&self.title, None, None);
        font.crop_right(&mut plan, max_width);
        let dx = padding + (max_width - plan.width) / 2;
        let dy = (self.rect.height() as i32 - font.x_heights.0 as i32) / 2;
        let pt = pt!(self.rect.min.x + dx, self.rect.max.y - dy);
        fb.draw_rectangle(&self.rect, WHITE);
        font.render(fb, BLACK, &plan, pt);
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
