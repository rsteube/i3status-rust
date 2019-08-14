use crate::scheduler::Task;
use crossbeam_channel::Sender;
use std::process::Command;
use std::time::Duration;

use crate::block::{Block, ConfigBlock};
use crate::config::Config;
use crate::de::deserialize_duration;
use crate::errors::*;
use crate::input::{I3BarEvent, MouseButton};
use crate::widget::{I3BarWidget, State};
use crate::widgets::button::ButtonWidget;

use uuid::Uuid;

pub struct Pamac {
    output: ButtonWidget,
    id: String,
    update_interval: Duration,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct PamacConfig {
    /// Update interval in seconds
    #[serde(default = "PamacConfig::default_interval", deserialize_with = "deserialize_duration")]
    pub interval: Duration,
}

impl PamacConfig {
    fn default_interval() -> Duration {
        Duration::from_secs(60 * 10)
    }
}

impl ConfigBlock for Pamac {
    type Config = PamacConfig;

    fn new(block_config: Self::Config, config: Config, _tx_update_request: Sender<Task>) -> Result<Self> {
        Ok(Pamac {
            id: Uuid::new_v4().simple().to_string(),
            update_interval: block_config.interval,
            output: ButtonWidget::new(config, "pamac").with_icon("update"),
        })
    }
}

fn get_update_count() -> Result<usize> {
    // Get update count
    Ok(String::from_utf8(
        Command::new("sh")
            .env("LC_ALL", "C")
            .args(&["-c", "pamac checkupdates -q"])
            .output()
            .block_error("pamac", "There was a problem running the pamac command")?
            .stdout,
    )
    .block_error("pamac", "there was a problem parsing the output")?
    .lines()
    .count())
}

impl Block for Pamac {
    fn update(&mut self) -> Result<Option<Duration>> {
        let count = get_update_count()?;
        self.output.set_text(match count {
            0 => "".to_owned(),
            _ => format!("{}", count),
        });
        self.output.set_state(match count {
            0 => State::Idle,
            _ => State::Info,
        });
        Ok(Some(self.update_interval))
    }

    fn view(&self) -> Vec<&I3BarWidget> {
        vec![&self.output]
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn click(&mut self, event: &I3BarEvent) -> Result<()> {
        if event.name.as_ref().map(|s| s == "pamac").unwrap_or(false) && event.button == MouseButton::Left {
            Command::new("sh").args(&["-c", "pamac-manager --updates"]).output().ok();
        }
        Ok(())
    }
}
