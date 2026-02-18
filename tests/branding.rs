use ug2_client::patcher::Patch;
use ug2_client::patcher::patches::branding::*;

#[test]
fn test_nitro_rebranding() {
    let patch = NitroRebranding::new("Underground");
    assert_eq!(patch.apply("Discord Nitro Basic"), "Underground Premium Basic");
    assert_eq!(patch.apply("\"Nitro\""), "\"Premium\"");
    assert_eq!(patch.apply("Get Nitro now"), "Get Premium now");
}

#[test]
fn test_discord_rebranding() {
    let patch = DiscordRebranding::new("Underground");
    assert_eq!(patch.apply(" Discord is great"), " Underground is great");
    assert_eq!(patch.apply("Discord is great"), "Underground is great");
    assert_eq!(patch.apply("Discord's features"), "Underground's features");
}

#[test]
fn test_title_rebranding() {
    let patch = TitleRebranding::new("Underground");
    let input = r#"let o={base:n(723702).isPlatformEmbedded?void 0:"Discord"}"#;
    let expected = r#"let o={base:n(723702).isPlatformEmbedded?void 0:"Underground"}"#;
    assert_eq!(patch.apply(input), expected);
}

#[test]
fn test_server_to_guild() {
    let patch = ServerToGuild;
    assert_eq!(patch.apply(" Server "), " Guild ");
    assert_eq!(patch.apply("\"Servers\""), "\"Guilds\"");
    assert_eq!(patch.apply(" server "), " guild ");
}
