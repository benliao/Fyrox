use crate::scene::commands::RevertSceneNodePropertyCommand;
use crate::{
    scene::commands::{
        make_set_node_property_command,
        terrain::{AddTerrainLayerCommand, DeleteTerrainLayerCommand},
    },
    GameSceneCommand,
};
use fyrox::{
    core::pool::Handle,
    gui::inspector::{CollectionChanged, FieldKind, PropertyChanged},
    scene::{node::Node, terrain::Terrain},
};
use std::any::TypeId;

pub struct SceneNodePropertyChangedHandler;

impl SceneNodePropertyChangedHandler {
    fn try_get_command(
        &self,
        args: &PropertyChanged,
        handle: Handle<Node>,
        _node: &mut Node,
    ) -> Option<GameSceneCommand> {
        // Terrain is special and have its own commands for specific properties.
        if args.path() == Terrain::LAYERS && args.owner_type_id == TypeId::of::<Terrain>() {
            match args.value {
                FieldKind::Collection(ref collection_changed) => match **collection_changed {
                    CollectionChanged::Add(_) => {
                        Some(GameSceneCommand::new(AddTerrainLayerCommand::new(handle)))
                    }
                    CollectionChanged::Remove(index) => Some(GameSceneCommand::new(
                        DeleteTerrainLayerCommand::new(handle, index),
                    )),
                    CollectionChanged::ItemChanged { .. } => None,
                },
                _ => None,
            }
        } else {
            None
        }
    }
}

impl SceneNodePropertyChangedHandler {
    pub fn handle(
        &self,
        args: &PropertyChanged,
        handle: Handle<Node>,
        node: &mut Node,
    ) -> Option<GameSceneCommand> {
        self.try_get_command(args, handle, node).or_else(|| {
            if args.is_inheritable() {
                // Prevent reverting property value if there's no parent resource.
                if node.resource().is_some() {
                    Some(GameSceneCommand::new(RevertSceneNodePropertyCommand::new(
                        args.path(),
                        handle,
                    )))
                } else {
                    None
                }
            } else {
                make_set_node_property_command(handle, args)
            }
        })
    }
}
