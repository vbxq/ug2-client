use crate::config::BrandingConfig;
use crate::server::state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

pub async fn serve_index(State(state): State<AppState>) -> Response {
    let active_build = state.active_build.read().await;
    let build_hash = match active_build.as_ref() {
        Some(h) => h.clone(),
        None => {
            return (StatusCode::SERVICE_UNAVAILABLE, "You have successfully installed ug2-client, but you have no active build configured. Go to /selector to pick one.").into_response()
        }
    };
    drop(active_build);

    if let Ok(Some(data)) = state.fs_cache.get_asset(&build_hash, "index.html").await {
        if let Ok(html) = String::from_utf8(data) {
            return Html(html).into_response();
        }
    }

    use crate::db::models::discord_build;
    use sea_orm::*;

    let build = discord_build::Entity::find()
        .filter(discord_build::Column::BuildHash.eq(&build_hash))
        .one(&state.db)
        .await;

    match build {
        Ok(Some(build)) => {
            let index_scripts: Vec<String> =
                serde_json::from_value(build.index_scripts).unwrap_or_default();
            let branding = &state.config.patch_config.branding;
            let patches = &state.config.patch_config.patches;

            if index_scripts.is_empty() {
                tracing::warn!("No index_scripts for build {}, client won't load. Download the build first.", build_hash);
            }

            let html = generate_index(&build_hash, &index_scripts, branding, patches);
            Html(html).into_response()
        }
        _ => (StatusCode::SERVICE_UNAVAILABLE, "No build data available").into_response(),
    }
}

fn generate_index(
    build_hash: &str,
    scripts: &[String],
    branding: &BrandingConfig,
    patches: &crate::config::PatchToggles,
) -> String {
    let global_env_js = generate_global_env(branding, build_hash);

    let css_tags: String = scripts
        .iter()
        .filter(|s| s.trim_start_matches("/assets/").ends_with(".css"))
        .map(|s| {
            let asset = s.trim_start_matches("/assets/");
            format!(r#"    <link rel="stylesheet" href="/assets/{}">"#, html_escape(asset))
        })
        .collect::<Vec<_>>()
        .join("\n");

    let script_tags: String = scripts
        .iter()
        .filter(|s| !s.trim_start_matches("/assets/").ends_with(".css"))
        .map(|s| {
            let asset = s.trim_start_matches("/assets/");
            format!(r#"    <script src="/assets/{}" defer></script>"#, html_escape(asset))
        })
        .collect::<Vec<_>>()
        .join("\n");

    let fast_identify_js = if patches.fast_identify {
        FAST_IDENTIFY_SCRIPT
    } else {
        ""
    };

    let dev_experiments_js = if patches.enable_dev_experiments {
        DEV_EXPERIMENTS_SCRIPT
    } else {
        ""
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">

<head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{title}</title>

    <script>
        // Intercept XHR & fetch: Discord forces https: on API endpoints,
        // but our server runs on HTTP. Rewrite same-host https -> http.
        (function() {{
            if (location.protocol !== "http:") return;
            var hs = "https://" + location.host;
            var hp = "http://" + location.host;
            function rw(u) {{
                return (typeof u === "string" && u.indexOf(hs) === 0)
                    ? hp + u.slice(hs.length) : u;
            }}
            var xOpen = XMLHttpRequest.prototype.open;
            XMLHttpRequest.prototype.open = function(m, u) {{
                arguments[1] = rw(u);
                return xOpen.apply(this, arguments);
            }};
            var oFetch = window.fetch;
            window.fetch = function(i, o) {{
                return oFetch.call(this, rw(i), o);
            }};
        }})();

        window.__OVERLAY__ = /overlay/.test(location.pathname);
        window.__BILLING_STANDALONE__ = /^\/billing/.test(location.pathname);
{global_env}
        window.localStorage.setItem("gatewayURL", window.GLOBAL_ENV.GATEWAY_ENDPOINT);
        window.localStorage.setItem(
            "DeveloperOptionsStore",
            '{{"trace":false,"canary":true,"logGatewayEvents":false,"logOverlayEvents":false,"logAnalyticsEvents":false,"sourceMapsEnabled":false,"axeEnabled":false}}'
        );
    </script>
{css_tags}
{fast_identify}
</head>

<body>
    <div id="app-mount"></div>
{script_tags}
{dev_experiments}
</body>

</html>"#,
        title = branding.instance_name,
        global_env = global_env_js,
        css_tags = css_tags,
        fast_identify = fast_identify_js,
        script_tags = script_tags,
        dev_experiments = dev_experiments_js,
    )
}

fn generate_global_env(branding: &BrandingConfig, build_hash: &str) -> String {
    let gateway_expr = if let Some(ref gw) = branding.gateway_url {
        format!(r#""{}""#, gw)
    } else {
        r#"`${location.protocol === "https:" ? "wss://" : "ws://"}${location.host}`"#.to_string()
    };

    format!(
        r#"        window.GLOBAL_ENV = {{
            API_ENDPOINT: `//${{location.host}}/api`,
            API_VERSION: 9,
            GATEWAY_ENDPOINT: {gateway},
            WEBAPP_ENDPOINT: `//${{location.host}}`,
            CDN_HOST: "cdn.discordapp.com",
            ASSET_ENDPOINT: `//${{location.host}}`,
            PUBLIC_PATH: "/assets/",
            MEDIA_PROXY_ENDPOINT: "https://media.discordapp.net",
            WIDGET_ENDPOINT: `//${{location.host}}/widget`,
            INVITE_HOST: `${{location.host}}/invite`,
            GUILD_TEMPLATE_HOST: `${{location.host}}/template`,
            GIFT_CODE_HOST: `${{location.host}}/gift`,
            RELEASE_CHANNEL: "canary",
            MARKETING_ENDPOINT: "//discord.com",
            BRAINTREE_KEY: "production_5st77rrc_49pp2rp4phym7387",
            STRIPE_KEY: "pk_live_CUQtlpQUF0vufWpnpUmQvcdi",
            NETWORKING_ENDPOINT: "//router.discordapp.net",
            RTC_LATENCY_ENDPOINT: `//${{location.host}}/rtc`,
            ACTIVITY_APPLICATION_HOST: "discordsays.com",
            PROJECT_ENV: "production",
            REMOTE_AUTH_ENDPOINT: "//localhost:3020",
            SENTRY_TAGS: {{ buildId: "{build_hash}", buildType: "normal" }},
            MIGRATION_SOURCE_ORIGIN: `https://${{location.host}}`,
            MIGRATION_DESTINATION_ORIGIN: `https://${{location.host}}`,
            HTML_TIMESTAMP: Date.now(),
            ALGOLIA_KEY: "aca0d7082e4e63af5ba5917d5e96bed0"
        }};"#,
        gateway = gateway_expr,
        build_hash = build_hash,
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

const FAST_IDENTIFY_SCRIPT: &str = r#"
    <!-- fast identify -->
    <script>
        (() => {
            if (window.WebSocket == null) return;
            if (window.__OVERLAY__) return;

            const getStorage = (key) => {
                try {
                    return JSON.parse(localStorage.getItem(key));
                } catch (e) {
                    return undefined;
                }
            };

            const token = getStorage("token");
            if (!token) return;

            const encoding = window.DiscordNative != null || window.require != null ? "etf" : "json";
            const url = window.GLOBAL_ENV.GATEWAY_ENDPOINT +
                "/?encoding=" + encoding +
                "&v=" + window.GLOBAL_ENV.API_VERSION +
                "&compress=zlib-stream";

            console.log("[FAST IDENTIFY] connecting to:", url);

            const socket = new WebSocket(url);
            socket.binaryType = "arraybuffer";
            const start = Date.now();
            const state = { open: false, identity: false, gateway: url, messages: [] };

            socket.onopen = function () {
                console.log(`[FAST IDENTIFY] connected in ${Date.now() - start}ms`);
                state.open = true;
                console.log("[FAST IDENTIFY] Sending payload");
                state.identity = true;
                const payload = {
                    op: 2,
                    d: {
                        token: token,
                        capabilities: 509,
                        properties: {
                            ...(getStorage("deviceProperties") || {}),
                            browser_user_agent: navigator.userAgent,
                        },
                        compress: false,
                        presence: {
                            status: getStorage("UserSettingsStore")?.status || "online",
                            since: 0,
                            activities: [],
                            afk: false,
                        },
                    }
                };
                socket.send(JSON.stringify(payload));
            };

            socket.onclose = socket.onerror = (e) => {
                console.log("[FAST IDENTIFY] Failed", e);
                window._ws = null;
            };

            socket.onmessage = (message) => {
                state.messages.push(message);
            };

            window._ws = { ws: socket, state };
        })();
    </script>
"#;

const DEV_EXPERIMENTS_SCRIPT: &str = r#"    <script>
        window.webpackChunkdiscord_app.push([[ Math.random() ], {}, (req) => { wpRequire = req; }]);
        mod = Object.values(wpRequire.c).find(x => typeof x?.exports?.Z?.isDeveloper !== "undefined");
        usermod = Object.values(wpRequire.c).find(x => x?.exports?.default?.getUsers)
        nodes = Object.values(mod.exports.Z._dispatcher._actionHandlers._dependencyGraph.nodes)
        try {
            nodes.find(x => x.name == "ExperimentStore").actionHandler["OVERLAY_INITIALIZE"]({user: {flags: 1}})
        } catch (e) {}
        oldGetUser = usermod.exports.default.__proto__.getCurrentUser;
        usermod.exports.default.__proto__.getCurrentUser = () => ({isStaff: () => true})
        nodes.find(x => x.name == "DeveloperExperimentStore").actionHandler["CONNECTION_OPEN"]()
        usermod.exports.default.__proto__.getCurrentUser = oldGetUser
    </script>"#;
