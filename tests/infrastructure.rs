use ug2_client::patcher::Patch;
use ug2_client::patcher::patches::infrastructure::*;

#[test]
fn test_sentry_redirect() {
    let patch = SentryRedirect::new("https://yoursentry.vbxq.re");
    let input = "dsn:\"https://fa97a90475514c03a42f80cd36d147c4@sentry.io/140984\"";
    assert!(patch.apply(input).contains("yoursentry.vbxq.re"));
    assert!(!patch.apply(input).contains("sentry.io"));
}

#[test]
fn test_status_redirect() {
    let patch = StatusPageRedirect::new("status.vbxq.re");
    assert_eq!(patch.apply("status.discord.com"), "status.vbxq.re");
    assert_eq!(patch.apply("discordstatus.com"), "status.vbxq.re");
}
