use ug2_client::asset_downloader::extractor::extract_asset_refs;

#[test]
fn test_extract_hex_hashes() {
    let content = r#"something("b456855ec667950dcf68") + other("cfb9efe961b2bf3647bc")"#;
    let refs = extract_asset_refs(content);
    assert!(refs.contains("b456855ec667950dcf68"));
    assert!(refs.contains("cfb9efe961b2bf3647bc"));
}

#[test]
fn test_extract_chunk_hashes() {
    let content = r#"__webpack_require__.u=e=>""+({87494:"2681623fb3f7aa56",99979:"575c2e07eec17302"})[e]+".js""#;
    let refs = extract_asset_refs(content);
    assert!(refs.contains("2681623fb3f7aa56"));
    assert!(refs.contains("575c2e07eec17302"));
}

#[test]
fn test_extract_chunk_hashes_scientific() {
    let content = r#"{1e4:"b6b788238d60d9e8",10018:"5a6163200c89c118"}"#;
    let refs = extract_asset_refs(content);
    assert!(refs.contains("b6b788238d60d9e8"));
    assert!(refs.contains("5a6163200c89c118"));
}

#[test]
fn test_extract_asset_urls() {
    let content = r#"background: url(/assets/e689380400b1f2d2c6320a823a1ab079.svg)"#;
    let refs = extract_asset_refs(content);
    assert!(refs.contains("e689380400b1f2d2c6320a823a1ab079.svg"));
}

#[test]
fn test_extract_exports() {
    let content = r#"n.exports=a.p+"40532.f4ff6c4a39fa78f07880.css""#;
    let refs = extract_asset_refs(content);
    assert!(refs.contains("40532.f4ff6c4a39fa78f07880.css"));
}

#[test]
fn test_no_false_positives() {
    let content = "var x = 42; function hello() {}";
    let refs = extract_asset_refs(content);
    assert!(refs.is_empty());
}
