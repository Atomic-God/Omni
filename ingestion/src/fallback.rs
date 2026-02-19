// Symbol-Only Fallback for Unknown/Binary Data
pub trait SymbolOnlyFallback {
    fn extract_symbols(&self, bytes: &[u8]) -> String;
}

pub struct SymbolExtractor;

impl SymbolOnlyFallback for SymbolExtractor {
    fn extract_symbols(&self, bytes: &[u8]) -> String {
        // Simple heuristic: Treat byte sequences as opaque symbols if valid UTF-8 is sparse.
        // Convert to hex-string representation for "symbolic" handling of binary data.
        // e.g., "0xCA 0xFE 0xBA 0xBE"

        // Check if mostly text
        let text_chars = bytes.iter().filter(|&&b| b >= 32 && b <= 126).count();
        if text_chars > bytes.len() / 2 {
             // Treat as lossy utf8
             String::from_utf8_lossy(bytes).to_string()
        } else {
             // Treat as hex tokens
             let mut s = String::new();
             for chunk in bytes.chunks(4) { // 4-byte words
                 s.push_str("0x");
                 for b in chunk {
                     s.push_str(&format!("{:02X}", b));
                 }
                 s.push(' ');
             }
             s
        }
    }
}
