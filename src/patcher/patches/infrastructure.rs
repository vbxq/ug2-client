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

pub struct CdnRedirect {
    cdn_host: String,
    media_host: String,
}

impl CdnRedirect {
    pub fn new(cdn_host: &str, media_host: &str) -> Self {
        Self {
            cdn_host: cdn_host.to_string(),
            media_host: media_host.to_string(),
        }
    }
}

impl Patch for CdnRedirect {
    fn name(&self) -> &str { "cdn_redirect" }

    fn apply(&self, content: String) -> String {
        let rewrite_cdn = self.cdn_host != "cdn.discordapp.com" && content.contains("cdn.discordapp.com");
        let rewrite_media = self.media_host != "media.discordapp.net" && content.contains("media.discordapp.net");
        if !rewrite_cdn && !rewrite_media {
            return content;
        }
        let mut result = content;
        if rewrite_cdn {
            result = result.replace("cdn.discordapp.com", &self.cdn_host);
        }
        if rewrite_media {
            result = result.replace("media.discordapp.net", &self.media_host);
        }
        result
    }
}
