use std::{cell::RefCell, rc::Rc};

use ggez::{graphics::Color, Context};

use crate::{renderer::draw_rectangle, vec2d, vector::Vector, MainState};

pub enum UIElement<'a, T> {
    Button(Button<'a, T>),
    Menu(Menu<'a, T>),
}

impl<T> UIElement<'_, T> {
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

    pub fn input_at(&self, position: Vector, state: &mut T) {
        match self {
            UIElement::Button(x) => x.input_at(position, state),
            UIElement::Menu(x) => x.input_at(position, state),
        }
    }
}

pub struct Button<'a, T> {
    parent: Rc<RefCell<Menu<'a, T>>>,
    position: Vector,
    size: Vector,
    callback: fn(&mut T),
}

impl<'a, T> Button<'a, T> {
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

    pub fn new(
        position: Vector,
        size: Vector,
        parent: Rc<RefCell<Menu<'a, T>>>,
        callback: fn(&mut T),
    ) -> Self
    where
        T: Sized,
    {
        Self {
            position,
            size,
            parent,
            callback,
        }
    }

    pub fn is_hovered(&self, mouse: Vector) -> bool {
        self.x() <= mouse.x
            && self.y() <= mouse.y
            && self.x() + self.width() >= mouse.x
            && self.x() + self.height() >= mouse.y
    }

    pub fn click(&self, state: &mut T) {
        let mut callback = self.callback;
        callback(state);
    }

    pub fn draw(&self, ctx: &mut Context) {
        draw_rectangle(
            ctx,
            vec2d!(self.x(), self.y()),
            vec2d!(self.width(), self.height()),
            Color::BLUE,
        );
    }

    pub fn input_at(&self, position: Vector, state: &mut T) {
        if self.x() <= position.x
            && self.y() <= position.y
            && self.x() + self.width() >= position.x
            && self.y() + self.height() >= position.y
        {
            self.click(state);
        }
    }
}

impl<'a, T> Into<UIElement<'a, T>> for Button<'a, T> {
    fn into(self) -> UIElement<'a, T> {
        UIElement::Button(self)
    }
}

pub struct Menu<'a, T> {
    position: Vector,
    scale: f32,
    elements: Vec<UIElement<'a, T>>,
    parent: Option<&'a Menu<'a, T>>,
}

impl<'a, T> Menu<'a, T> {
    pub fn new(position: Vector, scale: f32, parent: Option<&'a Menu<'a, T>>) -> Self {
        Self {
            position,
            scale,
            elements: Vec::new(),
            parent,
        }
    }

    /// There *must* be a better way
    /// TODO: find a better way
    pub fn add_elements(&mut self, elements: Vec<UIElement<'a, T>>) {
        self.elements.reserve(elements.len());
        for element in elements {
            self.elements.push(element);
        }
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

    pub fn input_at(&self, position: Vector, state: &mut T) {
        let bounds = self.bounds();
        if bounds.0.x <= position.x
            && bounds.0.y <= position.y
            && bounds.1.x >= position.x
            && bounds.1.y >= position.y
        {
            for element in &self.elements {
                element.input_at(position, state);
            }
        }
    }

    pub fn set_position(&mut self, position: Vector) {
        self.position = position;
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }
}

impl<'a, T> Into<UIElement<'a, T>> for Menu<'a, T> {
    fn into(self) -> UIElement<'a, T> {
        UIElement::Menu(self)
    }
}

impl<T> Default for Menu<'_, T> {
    fn default() -> Self {
        Self {
            position: vec2d!(0.0, 0.0),
            scale: 1.0,
            elements: vec![],
            parent: None,
        }
    }
}
