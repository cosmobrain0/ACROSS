use std::cell::RefCell;

use ggez::{graphics::Color, Context};

use crate::{renderer::draw_rectangle, vec2d, vector::Vector};

#[derive(Debug, Clone)]
pub enum UIElement<'a> {
    Button(Button<'a>),
    Menu(Menu<'a>),
}

impl UIElement<'_> {
    pub fn position(&self) -> Vector {
        match self {
            UIElement::Button(x) => vec2d!(x.x(), x.y()),
            UIElement::Menu(x) => x.position(),
        }
    }

    pub fn size(&self) -> Vector {
        match self {
            UIElement::Button(x) => vec2d!(x.width(), x.height()),
            UIElement::Menu(x) => x.size(),
        }
    }

    pub fn draw(&self, ctx: &mut Context) {
        match self {
            UIElement::Button(x) => x.draw(ctx),
            UIElement::Menu(x) => x.draw(ctx),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Button<'a> {
    parent: RefCell<Menu<'a>>,
    position: Vector,
    size: Vector,
}

impl<'a> Button<'a> {
    #[inline]
    pub fn x(&self) -> f32 {
        self.position.x * self.parent.borrow().scale() + self.parent.borrow().position().x
    }
    #[inline]
    pub fn y(&self) -> f32 {
        self.position.y * self.parent.borrow().scale() + self.parent.borrow().position().y
    }
    #[inline]
    pub fn width(&self) -> f32 {
        self.size.x * self.parent.borrow().scale()
    }
    #[inline]
    pub fn height(&self) -> f32 {
        self.size.y * self.parent.borrow().scale()
    }

    pub fn new(position: Vector, size: Vector, parent: RefCell<Menu<'a>>) -> Self {
        Self {
            position,
            size,
            parent,
        }
    }

    pub fn is_hovered(&self, mouse: Vector) -> bool {
        self.x() <= mouse.x
            && self.y() <= mouse.y
            && self.x() + self.width() >= mouse.x
            && self.x() + self.height() >= mouse.y
    }

    pub fn draw(&self, ctx: &mut Context) {
        draw_rectangle(ctx, self.position, self.size, Color::BLUE);
    }
}

impl<'a> Into<UIElement<'a>> for Button<'a> {
    fn into(self) -> UIElement<'a> {
        UIElement::Button(self)
    }
}

#[derive(Debug, Clone)]
pub struct Menu<'a> {
    position: Vector,
    scale: f32,
    elements: Vec<UIElement<'a>>,
    parent: Option<&'a Menu<'a>>,
}

impl<'a> Menu<'a> {
    pub fn new(position: Vector, scale: f32, parent: Option<&'a Menu<'a>>) -> Self {
        Self {
            position,
            scale,
            elements: Vec::new(),
            parent,
        }
    }

    /// There *must* be a better way
    /// TODO: find a better way
    pub fn add_elements(&mut self, elements: Vec<UIElement<'a>>) {
        self.elements = [self.elements.clone(), elements].concat();
        println!("{}", self.elements.len());
    }

    pub fn position(&self) -> Vector {
        match self.parent {
            Some(parent) => self.position * parent.scale + parent.position(),
            None => self.position,
        }
    }

    pub fn scale(&self) -> f32 {
        match self.parent {
            Some(parent) => self.scale * parent.scale(),
            None => self.scale,
        }
    }

    /// Returns (top_left, bottom_right)
    pub fn bounds(&self) -> (Vector, Vector) {
        let initial: Vector = self.elements[0].position();
        let initial: (Vector, Vector) = (initial, initial);
        self.elements.iter().fold(initial, |bounds, element| {
            let position = element.position();
            let size = element.size();
            (bounds.0.min(position), bounds.1.max(position + size))
        })
    }

    pub fn size(&self) -> Vector {
        let bounds = self.bounds();
        bounds.1 - bounds.0
    }

    pub fn draw(&self, ctx: &mut Context) {
        let bounds = self.bounds();
        draw_rectangle(
            ctx,
            bounds.0,
            bounds.1 - bounds.0,
            Color::new(1.0, 1.0, 1.0, 0.3),
        );
        self.elements.iter().for_each(|x| x.draw(ctx));
    }
}

impl<'a> Into<UIElement<'a>> for Menu<'a> {
    fn into(self) -> UIElement<'a> {
        UIElement::Menu(self)
    }
}

impl Default for Menu<'_> {
    fn default() -> Self {
        Self {
            position: vec2d!(0.0, 0.0),
            scale: 1.0,
            elements: vec![],
            parent: None,
        }
    }
}
