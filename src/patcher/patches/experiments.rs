use crate::patcher::Patch;

pub struct EnableDevExperiments;

impl Patch for EnableDevExperiments {
    fn name(&self) -> &str { "enable_dev_experiments" }

    fn apply(&self, content: &str) -> String {
        content.replace(
            "DeveloperExperimentStore\";isDeveloper=!1",
            "DeveloperExperimentStore\";isDeveloper=!0",
        )
    }
}
