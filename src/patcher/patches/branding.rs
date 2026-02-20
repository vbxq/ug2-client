use crate::patcher::Patch;

pub struct NitroRebranding {
    instance_name: String,
}

impl NitroRebranding {
    pub fn new(instance_name: &str) -> Self {
        Self { instance_name: instance_name.to_string() }
    }
}

impl Patch for NitroRebranding {
    fn name(&self) -> &str { "nitro_rebranding" }

    fn apply(&self, content: String) -> String {
        if !content.contains("Nitro") {
            return content;
        }
        let name = &self.instance_name;
        content
            .replace("Discord Nitro", &format!("{} Premium", name))
            .replace("\"Nitro\"", "\"Premium\"")
            .replace("Nitro ", "Premium ")
            .replace(" Nitro", " Premium")
            .replace("[Nitro]", "[Premium]")
            .replace("*Nitro*", "*Premium*")
            .replace("\"Nitro. ", "\"Premium. ")
    }
}

pub struct DiscordRebranding {
    instance_name: String,
}

impl DiscordRebranding {
    pub fn new(instance_name: &str) -> Self {
        Self { instance_name: instance_name.to_string() }
    }
}

impl Patch for DiscordRebranding {
    fn name(&self) -> &str { "discord_rebranding" }

    fn apply(&self, content: String) -> String {
        if !content.contains("Discord") {
            return content;
        }
        let name = &self.instance_name;
        content
            .replace(" Discord ", &format!(" {} ", name))
            .replace("Discord ", &format!("{} ", name))
            .replace(" Discord", &format!(" {}", name))
            .replace("Discord's", &format!("{}'s", name))
            .replace("*Discord*", &format!("*{}*", name))
    }
}

pub struct TitleRebranding {
    instance_name: String,
}

impl TitleRebranding {
    pub fn new(instance_name: &str) -> Self {
        Self { instance_name: instance_name.to_string() }
    }
}

impl Patch for TitleRebranding {
    fn name(&self) -> &str { "title_rebranding" }

    fn apply(&self, content: String) -> String {
        if !content.contains("isPlatformEmbedded") {
            return content;
        }
        content.replace(
            r#"isPlatformEmbedded?void 0:"Discord""#,
            &format!(r#"isPlatformEmbedded?void 0:"{}""#, self.instance_name),
        )
    }
}

pub struct ServerToGuild;

impl Patch for ServerToGuild {
    fn name(&self) -> &str { "server_to_guild" }

    fn apply(&self, content: String) -> String {
        if !content.contains("Server") && !content.contains("server") {
            return content;
        }

        let replacements: &[(&str, &str)] = &[
            ("\"Server\"", "\"Guild\""),
            ("\"Server ", "\"Guild "),
            (" Server\"", " Guild\""),
            (" Server ", " Guild "),
            ("\"Server.\"", "\"Guild.\""),
            (" Server.\"", " Guild.\""),
            ("\"Server,\"", "\"Guild,\""),
            (" Server,\"", " Guild,\""),
            (" Server,", " Guild,"),
            ("\"Servers\"", "\"Guilds\""),
            ("\"Servers ", "\"Guilds "),
            (" Servers\"", " Guilds\""),
            (" Servers ", " Guilds "),
            ("\"Servers.\"", "\"Guilds.\""),
            (" Servers.\"", " Guilds.\""),
            ("\"Servers,\"", "\"Guilds,\""),
            (" Servers,\"", " Guilds,\""),
            (" Servers,", " Guilds,"),
            ("\nServers", "\nGuilds"),
        ];

        let mut result = content;
        for (from, to) in replacements {
            result = result.replace(from, to);
            result = result.replace(&from.to_lowercase(), &to.to_lowercase());
        }
        result
    }
}
