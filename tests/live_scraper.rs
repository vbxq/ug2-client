use ug2_client::discord_scraper::live_scraper::parse_discord_html;

#[test]
fn test_parse_discord_html() {
    let html = r#"<!DOCTYPE html><html><head>
<script nonce="x">window.GLOBAL_ENV = {"VERSION_HASH":"abc123def456","RELEASE_CHANNEL":"canary","BUILT_AT":"1771352999811","BUILD_NUMBER":"497946"}</script>
<script defer src="/assets/web.65877e3d81a538c8.js"></script>
<script defer src="/assets/sentry.fe742426952e96ab.js"></script>
<link href="/assets/web.cfdd2a8bad98202f.css" rel="stylesheet">
</head><body></body></html>"#;

    let result = parse_discord_html(html).unwrap();
    assert_eq!(result.build_hash, "abc123def456");
    assert_eq!(result.channel, "canary");
    assert_eq!(result.scripts.len(), 3);
    assert!(result.scripts.contains(&"web.65877e3d81a538c8.js".to_string()));
    assert!(result.scripts.contains(&"sentry.fe742426952e96ab.js".to_string()));
    assert!(result.scripts.contains(&"web.cfdd2a8bad98202f.css".to_string()));
}
