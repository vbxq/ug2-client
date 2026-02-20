use ug2_client::patcher::Patch;
use ug2_client::patcher::patches::features::*;

#[test]
fn test_prevent_localstorage() {
    let patch = PreventLocalStorageDeletion;
    let input = "if(window.localStorage){delete window.localStorage}";
    let result = patch.apply(input.into());
    assert!(result.contains("void 0"));
    assert!(!result.contains("delete window.localStorage"));
}

#[test]
fn test_fast_identify_new() {
    let patch = FastIdentifyFix;
    let input = "this.isFastConnect=e,e?this._doFastConnectIdentify():this._doResumeOrIdentify()";
    let result = patch.apply(input.into());
    assert!(result.contains("this._doResumeOrIdentify()"));
    assert!(!result.contains("_doFastConnectIdentify"));
    assert_eq!(result, "this.isFastConnect=e,e?this._doResumeOrIdentify():this._doResumeOrIdentify()");
}

#[test]
fn test_gateway_reconnect() {
    let patch = GatewayReconnectPatch;
    assert_eq!(
        patch.apply("this.isFastConnect=!1".into()),
        "this.isFastConnect=!0"
    );
}

#[test]
fn test_gateway_reconnect_with_context() {
    let patch = GatewayReconnectPatch;
    let input = "this.hasConnectedOnce=!1,this.isFastConnect=!1,this.identifyCount=0";
    let result = patch.apply(input.into());
    assert!(result.contains("isFastConnect=!0"));
}

#[test]
fn test_no_xss_warning() {
    let patch = NoXssWarning;
    let input = r#"if(null!=a.A.Messages.SELF_XSS_HEADER)if(console.log(`%c${a.A.Messages.SELF_XSS_HEADER}`"#;
    let result = patch.apply(input.into());
    assert!(result.contains("if(false)"));
    assert!(!result.contains("null!="));
}
