//! Pattern matching for AOB (Array of Bytes) scans

/// AOB pattern with wildcards
#[derive(Debug, Clone)]
pub struct Pattern {
    bytes: Vec<Option<u8>>,
}

impl Pattern {
    /// Parse a pattern string like "48 8B ?? 50 FF"
    pub fn parse(pattern: &str) -> Result<Self, String> {
        let bytes: Result<Vec<Option<u8>>, String> = pattern
            .split_whitespace()
            .map(|s| {
                if s == "??" || s == "?" {
                    Ok(None)
                } else {
                    u8::from_str_radix(s, 16)
                        .map(Some)
                        .map_err(|e| format!("Invalid hex: {}", e))
                }
            })
            .collect();

        Ok(Self { bytes: bytes? })
    }

    /// Match pattern against buffer
    pub fn matches(&self, buffer: &[u8]) -> bool {
        if buffer.len() < self.bytes.len() {
            return false;
        }

        for (i, pattern_byte) in self.bytes.iter().enumerate() {
            if let Some(b) = pattern_byte {
                if buffer[i] != *b {
                    return false;
                }
            }
        }

        true
    }

    /// Find all occurrences in buffer
    pub fn find_all(&self, buffer: &[u8]) -> Vec<usize> {
        let mut results = Vec::new();

        for i in 0..=buffer.len().saturating_sub(self.bytes.len()) {
            if self.matches(&buffer[i..]) {
                results.push(i);
            }
        }

        results
    }

    /// Get pattern length
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Check if pattern is empty
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_parse() {
        let pattern = Pattern::parse("48 8B ?? 50").unwrap();
        assert_eq!(pattern.len(), 4);
    }

    #[test]
    fn test_pattern_match() {
        let pattern = Pattern::parse("48 8B ?? 50").unwrap();
        assert!(pattern.matches(&[0x48, 0x8B, 0xFF, 0x50]));
        assert!(pattern.matches(&[0x48, 0x8B, 0x00, 0x50]));
        assert!(!pattern.matches(&[0x48, 0x8B, 0xFF, 0x51]));
    }

    #[test]
    fn test_pattern_find() {
        let pattern = Pattern::parse("48 ?? 50").unwrap();
        let buffer = vec![0x00, 0x48, 0xFF, 0x50, 0x00, 0x48, 0x00, 0x50];
        let results = pattern.find_all(&buffer);
        assert_eq!(results, vec![1, 5]);
    }
}
