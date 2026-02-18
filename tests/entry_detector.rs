use ug2_client::asset_downloader::entry_detector::*;

#[test]
fn test_chunk_detection_self() {
    let chunk = r#"(self["webpackChunkdiscord_app"]=self["webpackChunkdiscord_app"]||[]).push([[1234],{5678:function(e,t,n){"use strict";}}]);"#;
    assert!(is_webpack_chunk(chunk));
}

#[test]
fn test_chunk_detection_this() {
    let chunk = r#"(this.webpackChunkdiscord_app=this.webpackChunkdiscord_app||[]).push([[40532],{517364:()=>{}}]);"#;
    assert!(is_webpack_chunk(chunk));
}

#[test]
fn test_chunk_with_use_strict() {
    let chunk = r#""use strict";(this.webpackChunkdiscord_app=this.webpackChunkdiscord_app||[]).push([[465],{700465:(s,e,a)=>{}}]);"#;
    assert!(is_webpack_chunk(chunk));
}

#[test]
fn test_chunk_with_license_comment() {
    let chunk = r#"/*! For license information please see abc.js.LICENSE.txt */
(this.webpackChunkdiscord_app=this.webpackChunkdiscord_app||[]).push([[81819],{}]);"#;
    assert!(is_webpack_chunk(chunk));
}

#[test]
fn test_entry_webpack_runtime() {
    let entry = r#"(()=>{"use strict";var e,d,c,a,f,b,t,r,n,o,i={},s={};function l(e){var d=s[e];if(void 0!==d)return d.exports;var c=s[e]={id:e,loaded:!1,exports:{}};i[e].call(c.exports,c,c.exports,l);c.loaded=!0;return c.exports}l.m=i;l.c=s;"#;
    assert!(!is_webpack_chunk(entry));
}

#[test]
fn test_entry_function_style() {
    let entry = r#"!function(){"use strict";var e={12345:function(e){e.exports={}}};"#;
    assert!(!is_webpack_chunk(entry));
}

#[test]
fn test_extract_single_chunk_id() {
    let content = r#"(this.webpackChunkdiscord_app=this.webpackChunkdiscord_app||[]).push([[40532],{517364:()=>{}}]);"#;
    assert_eq!(extract_chunk_ids(content), vec![40532]);
}

#[test]
fn test_extract_multiple_chunk_ids() {
    let content = r#"(this.webpackChunkdiscord_app=this.webpackChunkdiscord_app||[]).push([[81819,32162,99322],{517364:()=>{}}]);"#;
    assert_eq!(extract_chunk_ids(content), vec![81819, 32162, 99322]);
}

#[test]
fn test_has_entry_factory_deferred() {
    let tail = r#"[40532,56054,97621,41446,54313,38634].map(e.E)}),5);var t=t=>e(e.s=t);e.O(0,[40532],(()=>(t(128594),t(535666),t(784633),t(289364))));e.O()}]);
//# sourceMappingURL=07e5e273fbca67f2a275.js.map"#;
    assert!(has_entry_factory(tail));
}

#[test]
fn test_has_entry_factory_simple() {
    let tail = r#"},t=>{var e=e=>t(t.s=e);e(128594),e(535666),e(784633),e(127124)}]);
//# sourceMappingURL=e0d02106cf52f3b8851e.js.map"#;
    assert!(has_entry_factory(tail));
}

#[test]
fn test_regular_chunk_no_entry() {
    let tail = r#"r=(this.height_-1)/(this.labels_.length-1),n=1;n<this.labels_.length;++n)e.fillText(this.labels_[n],t,r*n)}}};return e}();return e}();e.exports=t}}]);
//# sourceMappingURL=b62c62429a41fb1f5911.js.map"#;
    assert!(!has_entry_factory(tail));
}

#[test]
fn test_regular_chunk_jsx() {
    let tail = r#"(0,i.jsx)(t.zxk,{onClick:e,children:r.Z.Messages.OKAY})})]})}}}]);
//# sourceMappingURL=36a7e76e72fc807c0457.js.map"#;
    assert!(!has_entry_factory(tail));
}

#[test]
fn test_deferred_deps_extraction() {
    let tail = r#"e.O(0,[40532],(()=>(t(128594),t(535666))));e.O()}]);"#;
    let caps: Vec<_> = DEFERRED_DEPS_RE
        .captures_iter(tail)
        .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
        .collect();
    assert_eq!(caps, vec!["40532"]);
}

#[test]
fn test_deferred_deps_multiple() {
    let tail = r#"e.O(0,[40532,56054],(()=>(t(128594))));e.O()}]);"#;
    let deps: Vec<u64> = DEFERRED_DEPS_RE
        .captures_iter(tail)
        .filter_map(|c| c.get(1))
        .flat_map(|m| m.as_str().split(',').filter_map(|s| s.trim().parse().ok()))
        .collect();
    assert_eq!(deps, vec![40532, 56054]);
}
