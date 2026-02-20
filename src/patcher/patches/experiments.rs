use crate::patcher::Patch;

pub struct EnableDevExperiments;

impl Patch for EnableDevExperiments {
    fn name(&self) -> &str { "enable_dev_experiments" }

    fn apply(&self, content: String) -> String {
        if !content.contains("DeveloperExperimentStore") {
            return content;
        }
        content.replace(
            "DeveloperExperimentStore\";isDeveloper=!1",
            "DeveloperExperimentStore\";isDeveloper=!0",
        )
    }
}
