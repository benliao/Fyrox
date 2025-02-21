use crate::{
    menu::{create_menu_item, create_menu_item_shortcut, ui::UiMenu},
    message::MessageSender,
    scene::{controller::SceneController, Selection},
    ui_scene::{commands::graph::PasteWidgetCommand, UiScene},
    world::WorldViewerItemContextMenu,
    Engine, Message, MessageDirection,
};
use fyrox::{
    core::pool::Handle,
    gui::{
        menu::MenuItemMessage,
        message::UiMessage,
        popup::{Placement, PopupBuilder, PopupMessage},
        stack_panel::StackPanelBuilder,
        widget::{WidgetBuilder, WidgetMessage},
        BuildContext, RcUiNodeHandle, UiNode,
    },
};

pub struct WidgetContextMenu {
    menu: RcUiNodeHandle,
    delete_selection: Handle<UiNode>,
    copy_selection: Handle<UiNode>,
    widgets_menu: UiMenu,
    placement_target: Handle<UiNode>,
    paste: Handle<UiNode>,
}

impl WorldViewerItemContextMenu for WidgetContextMenu {
    fn menu(&self) -> RcUiNodeHandle {
        self.menu.clone()
    }
}

impl WidgetContextMenu {
    pub fn new(ctx: &mut BuildContext) -> Self {
        let delete_selection;
        let copy_selection;
        let paste;

        let widgets_menu = UiMenu::new(UiMenu::default_entries(), "Create Child Widget", ctx);

        let menu = PopupBuilder::new(WidgetBuilder::new().with_visibility(false))
            .with_content(
                StackPanelBuilder::new(
                    WidgetBuilder::new()
                        .with_child({
                            delete_selection =
                                create_menu_item_shortcut("Delete Selection", "Del", vec![], ctx);
                            delete_selection
                        })
                        .with_child({
                            copy_selection =
                                create_menu_item_shortcut("Copy Selection", "Ctrl+C", vec![], ctx);
                            copy_selection
                        })
                        .with_child({
                            paste = create_menu_item("Paste As Child", vec![], ctx);
                            paste
                        })
                        .with_child(widgets_menu.menu),
                )
                .build(ctx),
            )
            .build(ctx);
        let menu = RcUiNodeHandle::new(menu, ctx.sender());

        Self {
            widgets_menu,
            menu,
            delete_selection,
            copy_selection,
            placement_target: Default::default(),
            paste,
        }
    }

    pub fn handle_ui_message(
        &mut self,
        message: &UiMessage,
        editor_selection: &Selection,
        controller: &mut dyn SceneController,
        engine: &mut Engine,
        sender: &MessageSender,
    ) {
        if let Some(ui_scene) = controller.downcast_mut::<UiScene>() {
            self.widgets_menu
                .handle_ui_message(sender, message, ui_scene, editor_selection);

            if let Some(MenuItemMessage::Click) = message.data::<MenuItemMessage>() {
                if message.destination() == self.delete_selection {
                    if let Selection::Ui(ui_selection) = editor_selection {
                        sender.send(Message::DoUiSceneCommand(
                            ui_selection
                                .make_deletion_command(&ui_scene.ui, editor_selection.clone()),
                        ));
                    }
                } else if message.destination() == self.copy_selection {
                    if let Selection::Ui(ui_selection) = editor_selection {
                        ui_scene
                            .clipboard
                            .fill_from_selection(ui_selection, &ui_scene.ui);
                    }
                } else if message.destination() == self.paste {
                    if let Selection::Ui(ui_selection) = editor_selection {
                        if let Some(first) = ui_selection.widgets.first() {
                            if !ui_scene.clipboard.is_empty() {
                                sender.do_ui_scene_command(PasteWidgetCommand::new(*first));
                            }
                        }
                    }
                }
            } else if let Some(PopupMessage::Placement(Placement::Cursor(target))) = message.data()
            {
                if message.destination() == self.menu.handle() {
                    self.placement_target = *target;

                    // Check if there's something to paste and deactivate "Paste" if nothing.
                    engine.user_interface.send_message(WidgetMessage::enabled(
                        self.paste,
                        MessageDirection::ToWidget,
                        !ui_scene.clipboard.is_empty(),
                    ));
                }
            }
        }
    }
}
