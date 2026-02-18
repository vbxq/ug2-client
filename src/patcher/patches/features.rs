use crate::patcher::Patch;
use regex::Regex;
use std::sync::LazyLock;

pub struct PreventLocalStorageDeletion;

static LS_DELETE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"delete\s+(window|globalThis)\.localStorage"#).unwrap()
});

impl Patch for PreventLocalStorageDeletion {
    fn name(&self) -> &str { "prevent_localstorage_deletion" }

    fn apply(&self, content: &str) -> String {
        LS_DELETE_RE
            .replace_all(content, "void 0")
            .to_string()
    }
}

pub struct FastIdentifyFix;

impl Patch for FastIdentifyFix {
    fn name(&self) -> &str { "fast_identify" }

    fn apply(&self, content: &str) -> String {
        content.replace(
            "?this._doFastConnectIdentify():this._doResumeOrIdentify()",
            "?this._doResumeOrIdentify():this._doResumeOrIdentify()",
        )
    }
}

pub struct GatewayReconnectPatch;

static RECONNECT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(this\.)isFastConnect\s*=\s*!1"#).unwrap()
});

impl Patch for GatewayReconnectPatch {
    fn name(&self) -> &str { "gateway_reconnect" }

    fn apply(&self, content: &str) -> String {
        RECONNECT_RE
            .replace_all(content, "${1}isFastConnect=!0")
            .to_string()
    }
}

pub struct RemoveQrCodeLogin;

static QR_CODE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"\w\?\(\d,\w\.jsx\)\(\w*,\{authTokenCallback:this\.handleAuthToken\}\):null"#).unwrap()
});

impl Patch for RemoveQrCodeLogin {
    fn name(&self) -> &str { "remove_qr_login" }

    fn apply(&self, content: &str) -> String {
        QR_CODE_RE.replace_all(content, "null").to_string()
    }
}

pub struct NoXssWarning;

static SELF_XSS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"null\s*!=\s*\w+\.\w+\.Messages\.SELF_XSS_HEADER"#).unwrap()
});

impl Patch for NoXssWarning {
    fn name(&self) -> &str { "no_xss_warning" }

    fn apply(&self, content: &str) -> String {
        SELF_XSS_RE
            .replace_all(content, "false")
            .to_string()
    }
}
