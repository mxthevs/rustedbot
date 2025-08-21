use super::Command;
use once_cell::sync::Lazy;
use std::sync::{Arc, RwLock};

type CommandList = Vec<Arc<dyn Command + Send + Sync>>;
static COMMANDS: Lazy<RwLock<CommandList>> = Lazy::new(|| RwLock::new(Vec::new()));

pub fn register(command: Arc<dyn Command + Send + Sync>) {
    COMMANDS.write().unwrap().push(command);
}

pub struct Registry;

impl Registry {
    pub fn all() -> Vec<Arc<dyn Command + Send + Sync>> {
        let cmds = COMMANDS.read().unwrap();
        cmds.clone()
    }

    pub fn get(name: &str) -> Option<Arc<dyn Command + Send + Sync>> {
        let cmds = COMMANDS.read().unwrap();
        cmds.iter().find(|c| c.name() == name).cloned()
    }
}

pub fn is_builtin(name: &str) -> bool {
    Registry::get(name).is_some()
}

#[macro_export]
macro_rules! register_command {
    ($t:ty) => {
        #[ctor::ctor]
        fn register() {
            $crate::commands::registry::register(std::sync::Arc::new(<$t>::default()));
        }
    };
}
