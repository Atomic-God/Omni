use ingestion::fallback::{SymbolExtractor, SymbolOnlyFallback};

#[test]
fn test_symbol_extractor_text() {
    let text = "Hello World";
    let bytes = text.as_bytes();
    let extractor = SymbolExtractor;
    let result = extractor.extract_symbols(bytes);
    assert_eq!(result, "Hello World");
}

#[test]
fn test_symbol_extractor_binary() {
    let bytes = vec![0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0xFF];
    let extractor = SymbolExtractor;
    let result = extractor.extract_symbols(&bytes);
    // Should be hex tokens
    // "0xCA 0xFE 0xBA 0xBE 0x00 0xFF "?
    // Let's check logic:
    // for chunk in bytes.chunks(4) {
    //    s.push_str("0x");
    //    for b in chunk { s.push_str(&format!("{:02X}", b)); }
    //    s.push(' ');
    // }
    // Chunk 1: CA FE BA BE -> "0xCAFEBABE "
    // Chunk 2: 00 FF -> "0x00FF "
    assert_eq!(result, "0xCAFEBABE 0x00FF ");
}

#[test]
fn test_symbol_extractor_mixed() {
    // Mostly text but some binary
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"Hello World");
    bytes.push(0x00);
    // 11 chars + 1 binary = 12 bytes. 11 are text. 11 > 6. So it should be text.
    let extractor = SymbolExtractor;
    let result = extractor.extract_symbols(&bytes);
    // String::from_utf8_lossy
    assert!(result.contains("Hello World"));
}
