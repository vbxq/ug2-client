use crate::patcher::Patch;

pub struct SentryRedirect {
    target_url: String,
}

impl SentryRedirect {
    pub fn new(target_url: &str) -> Self {
        Self { target_url: target_url.to_string() }
    }
}

impl Patch for SentryRedirect {
    fn name(&self) -> &str { "sentry_redirect" }

    fn apply(&self, content: String) -> String {
        if !content.contains("fa97a90475514c03") {
            return content;
        }
        content.replace(
            // TODO: dynamically retrieve this, it probably changed by now
            "https://fa97a90475514c03a42f80cd36d147c4@sentry.io/140984",
            &self.target_url,
        )
    }
}

pub struct StatusPageRedirect {
    target_url: String,
}

impl StatusPageRedirect {
    pub fn new(target_url: &str) -> Self {
        Self { target_url: target_url.to_string() }
    }
}

impl Patch for StatusPageRedirect {
    fn name(&self) -> &str { "status_page_redirect" }

    fn apply(&self, content: String) -> String {
        if !content.contains("status.discord.com") && !content.contains("discordstatus.com") {
            return content;
        }
        content
            .replace("status.discord.com", &self.target_url)
            .replace("discordstatus.com", &self.target_url)
    }
}
