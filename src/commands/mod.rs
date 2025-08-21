pub mod addcmd;
pub mod cmd;
pub mod delcmd;
pub mod gtasa;
pub mod node;
pub mod odds;
pub mod ping;
pub mod trust;
pub mod untrust;
pub mod updcmd;
pub mod wttr;

pub mod registry;
use async_trait::async_trait;

#[async_trait]
pub trait Command {
    fn name(&self) -> &'static str;

    fn requires_trust(&self) -> bool {
        false
    }

    async fn execute(&self, args: &str, sender: &str) -> String;
}
