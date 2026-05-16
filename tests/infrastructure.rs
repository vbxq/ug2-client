use ug2_client::patcher::Patch;
use ug2_client::patcher::patches::infrastructure::*;

#[test]
fn test_sentry_redirect() {
    let patch = SentryRedirect::new("https://yoursentry.vbxq.re");
    let input: String = "dsn:\"https://fa97a90475514c03a42f80cd36d147c4@sentry.io/140984\"".into();
    assert!(patch.apply(input.clone()).contains("yoursentry.vbxq.re"));
    assert!(!patch.apply(input).contains("sentry.io"));
}

#[test]
fn test_status_redirect() {
    let patch = StatusPageRedirect::new("status.vbxq.re");
    assert_eq!(patch.apply("status.discord.com".into()), "status.vbxq.re");
    assert_eq!(patch.apply("discordstatus.com".into()), "status.vbxq.re");
}

#[test]
fn test_cdn_redirect_rewrites_hosts() {
    let patch = CdnRedirect::new("cdn.celeste.gg", "media.celeste.gg", vec![]);
    let input = "fetch(\"https://cdn.discordapp.com/attachments/x.png\");src=\"//media.discordapp.net/y.jpg\";";
    let expected = "fetch(\"https://cdn.celeste.gg/attachments/x.png\");src=\"//media.celeste.gg/y.jpg\";";
    assert_eq!(patch.apply(input.into()), expected);
}

#[test]
fn test_cdn_redirect_noop_when_hosts_unchanged() {
    let patch = CdnRedirect::new("cdn.discordapp.com", "media.discordapp.net", vec![]);
    let input = "https://cdn.discordapp.com/x";
    assert_eq!(patch.apply(input.into()), input);
}

#[test]
fn test_cdn_redirect_only_replaces_configured_hosts() {
    let patch = CdnRedirect::new("cdn.celeste.gg", "media.discordapp.net", vec![]);
    let input = "https://cdn.discordapp.com/a https://media.discordapp.net/b";
    let expected = "https://cdn.celeste.gg/a https://media.discordapp.net/b";
    assert_eq!(patch.apply(input.into()), expected);
}

#[test]
fn test_cdn_redirect_preserves_bypass_paths() {
    let bypass = vec!["/assets/".to_string(), "/detectables/".to_string()];
    let patch = CdnRedirect::new("cdn.celeste.gg", "media.celeste.gg", bypass);
    let input = concat!(
        "https://cdn.discordapp.com/attachments/x.png ",
        "https://cdn.discordapp.com/assets/y.png ",
        "https://cdn.discordapp.com/detectables/games.json ",
        "https://media.discordapp.net/external/foo ",
        "https://media.discordapp.net/assets/bar.png"
    );
    let expected = concat!(
        "https://cdn.celeste.gg/attachments/x.png ",
        "https://cdn.discordapp.com/assets/y.png ",
        "https://cdn.discordapp.com/detectables/games.json ",
        "https://media.celeste.gg/external/foo ",
        "https://media.discordapp.net/assets/bar.png"
    );
    assert_eq!(patch.apply(input.into()), expected);
}

#[test]
fn test_cdn_redirect_reapplies_correctly_on_already_patched_content() {
    let bypass = vec!["/assets/".to_string()];
    let patch = CdnRedirect::new("cdn.celeste.gg", "media.celeste.gg", bypass);
    let input = "https://cdn.celeste.gg/attachments/x.png https://cdn.celeste.gg/assets/y.png";
    let expected = "https://cdn.celeste.gg/attachments/x.png https://cdn.discordapp.com/assets/y.png";
    assert_eq!(patch.apply(input.into()), expected);
}
