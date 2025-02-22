#![warn(missing_docs)]

//! The Border widget provides a stylized, static border around its child widget. See [`Border`] docs for more info and
//! usage examples.

use crate::{
    core::{
        algebra::Vector2, math::Rect, pool::Handle, reflect::prelude::*, scope_profile,
        visitor::prelude::*,
    },
    define_constructor,
    draw::{CommandTexture, Draw, DrawingContext},
    message::UiMessage,
    widget::{Widget, WidgetBuilder},
    BuildContext, Control, MessageDirection, Thickness, UiNode, UserInterface, BRUSH_PRIMARY,
};
use fyrox_core::uuid_provider;
use std::{
    any::{Any, TypeId},
    ops::{Deref, DerefMut},
};

/// The Border widget provides a stylized, static border around its child widget. Below is an example of creating a 1 pixel
/// thick border around a button widget:
///
/// ```rust
/// use fyrox_ui::{
///     UserInterface,
///     widget::WidgetBuilder,
///     border::BorderBuilder,
///     Thickness,
///     text::TextBuilder,
/// };
///
/// fn create_border_with_button(ui: &mut UserInterface) {
///     BorderBuilder::new(
///         WidgetBuilder::new()
///             .with_child(
///                 TextBuilder::new(WidgetBuilder::new())
///                     .with_text("I'm boxed in!")
///                     .build(&mut ui.build_ctx())
///             )
///     )
///     //You can also use Thickness::uniform(1.0)
///     .with_stroke_thickness(Thickness {left: 1.0, right: 1.0, top: 1.0, bottom: 1.0})
///     .build(&mut ui.build_ctx());
/// }
/// ```
///
/// As with other UI elements, we create the border using the BorderBuilder helper struct. The widget that should have a
/// border around it is added as a child of the base WidgetBuilder, and the border thickness can be set by providing a
/// Thickness struct to the BorderBuilder's *with_stroke_thickness* function. This means you can set different thicknesses
/// for each edge of the border.
///
/// You can style the border by creating a Brush and setting the border's base WidgetBuilder's foreground or background.
/// The foreground will set the style of the boarder itself, while setting the background will color the whole area within
/// the border. Below is an example of a blue border and a red background with white text inside.
///
/// ```rust
/// # use fyrox_ui::{
/// #     brush::Brush,
/// #     core::color::Color,
/// #     widget::WidgetBuilder,
/// #     text::TextBuilder,
/// #     border::BorderBuilder,
/// #     UserInterface,
/// #     Thickness,
/// # };
///
/// # let mut ui = UserInterface::new(Default::default());
///
/// BorderBuilder::new(
///     WidgetBuilder::new()
///         .with_foreground(Brush::Solid(Color::opaque(0, 0, 200)))
///         .with_background(Brush::Solid(Color::opaque(200, 0, 0)))
///         .with_child(
///             TextBuilder::new(WidgetBuilder::new())
///                 .with_text("I'm boxed in Blue and backed in Red!")
///                 .build(&mut ui.build_ctx())
///         )
/// )
/// .with_stroke_thickness(Thickness {left: 2.0, right: 2.0, top: 2.0, bottom: 2.0})
/// .build(&mut ui.build_ctx());
/// ```
#[derive(Default, Clone, Visit, Reflect, Debug)]
pub struct Border {
    /// Base widget of the border. See [`Widget`] docs for more info.
    pub widget: Widget,
    /// Stroke thickness for each side of the border.
    pub stroke_thickness: Thickness,
}

crate::define_widget_deref!(Border);

/// Supported border-specific messages.
#[derive(Debug, Clone, PartialEq)]
pub enum BorderMessage {
    /// Allows you to set stroke thickness at runtime. See [`Self::stroke_thickness`] docs for more.
    StrokeThickness(Thickness),
}

impl BorderMessage {
    define_constructor!(
        /// Creates a new [Self::StrokeThickness] message.
        BorderMessage:StrokeThickness => fn stroke_thickness(Thickness), layout: false
    );
}

uuid_provider!(Border = "6aba3dc5-831d-481a-bc83-ec10b2b2bf12");

impl Control for Border {
    fn query_component(&self, type_id: TypeId) -> Option<&dyn Any> {
        if type_id == TypeId::of::<Self>() {
            Some(self)
        } else {
            None
        }
    }

    fn measure_override(&self, ui: &UserInterface, available_size: Vector2<f32>) -> Vector2<f32> {
        scope_profile!();

        let margin_x = self.stroke_thickness.left + self.stroke_thickness.right;
        let margin_y = self.stroke_thickness.top + self.stroke_thickness.bottom;

        let size_for_child = Vector2::new(available_size.x - margin_x, available_size.y - margin_y);
        let mut desired_size = Vector2::default();

        for child_handle in self.widget.children() {
            ui.measure_node(*child_handle, size_for_child);
            let child = ui.nodes.borrow(*child_handle);
            let child_desired_size = child.desired_size();
            if child_desired_size.x > desired_size.x {
                desired_size.x = child_desired_size.x;
            }
            if child_desired_size.y > desired_size.y {
                desired_size.y = child_desired_size.y;
            }
        }

        desired_size.x += margin_x;
        desired_size.y += margin_y;

        desired_size
    }

    fn arrange_override(&self, ui: &UserInterface, final_size: Vector2<f32>) -> Vector2<f32> {
        scope_profile!();

        let rect_for_child = Rect::new(
            self.stroke_thickness.left,
            self.stroke_thickness.top,
            final_size.x - (self.stroke_thickness.right + self.stroke_thickness.left),
            final_size.y - (self.stroke_thickness.bottom + self.stroke_thickness.top),
        );

        for child_handle in self.widget.children() {
            ui.arrange_node(*child_handle, &rect_for_child);
        }

        final_size
    }

    fn draw(&self, drawing_context: &mut DrawingContext) {
        let bounds = self.widget.bounding_rect();
        DrawingContext::push_rect_filled(drawing_context, &bounds, None);
        drawing_context.commit(
            self.clip_bounds(),
            self.widget.background(),
            CommandTexture::None,
            None,
        );

        drawing_context.push_rect_vary(&bounds, self.stroke_thickness);
        drawing_context.commit(
            self.clip_bounds(),
            self.widget.foreground(),
            CommandTexture::None,
            None,
        );
    }

    fn handle_routed_message(&mut self, ui: &mut UserInterface, message: &mut UiMessage) {
        self.widget.handle_routed_message(ui, message);

        if message.destination() == self.handle()
            && message.direction() == MessageDirection::ToWidget
        {
            if let Some(BorderMessage::StrokeThickness(thickness)) = message.data() {
                if *thickness != self.stroke_thickness {
                    self.stroke_thickness = *thickness;
                    ui.send_message(message.reverse());
                    self.invalidate_layout();
                }
            }
        }
    }
}

/// Border builder.
pub struct BorderBuilder {
    /// Widget builder that will be used to build the base of the widget.
    pub widget_builder: WidgetBuilder,
    /// Stroke thickness for each side of the border. Default is 1px wide border for each side.
    pub stroke_thickness: Thickness,
}

impl BorderBuilder {
    /// Creates a new border builder with a widget builder specified.
    pub fn new(widget_builder: WidgetBuilder) -> Self {
        Self {
            widget_builder,
            stroke_thickness: Thickness::uniform(1.0),
        }
    }

    /// Sets the desired stroke thickness for each side of the border.
    pub fn with_stroke_thickness(mut self, stroke_thickness: Thickness) -> Self {
        self.stroke_thickness = stroke_thickness;
        self
    }

    /// Creates a [`Border`] widget, but does not add it to the user interface. Also see [`Self::build`] docs.
    pub fn build_border(mut self) -> Border {
        if self.widget_builder.foreground.is_none() {
            self.widget_builder.foreground = Some(BRUSH_PRIMARY);
        }
        Border {
            widget: self.widget_builder.build(),
            stroke_thickness: self.stroke_thickness,
        }
    }

    /// Finishes border building and adds it to the user interface. See examples in [`Border`] docs.
    pub fn build(self, ctx: &mut BuildContext<'_>) -> Handle<UiNode> {
        ctx.add_node(UiNode::new(self.build_border()))
    }
}
