//! Low-level USDA array parsing helpers.

use super::UsdImporter;

impl UsdImporter {
    /// Parse a vec3 array from USDA format
    pub(crate) fn parse_vec3_array(line: &str) -> Option<Vec<[f32; 3]>> {
        // Pattern: [(x, y, z), (x, y, z), ...]
        let line = line.trim();

        // Check if this is an array data line
        if !line.contains('(') || !line.contains(')') {
            return None;
        }

        let mut result = Vec::new();
        let mut paren_depth = 0;
        let mut current_triple = [0.0f32; 3];
        let mut triple_idx = 0;
        let mut current_num = String::new();

        for c in line.chars() {
            match c {
                '(' => {
                    paren_depth += 1;
                    triple_idx = 0;
                }
                ')' => {
                    if paren_depth > 0 && !current_num.is_empty() {
                        if let Ok(n) = current_num.parse::<f32>() {
                            current_triple[triple_idx] = n;
                        }
                        current_num.clear();
                    }
                    if paren_depth > 0 {
                        result.push(current_triple);
                    }
                    paren_depth -= 1;
                }
                ',' => {
                    if paren_depth > 0 && !current_num.is_empty() {
                        if let Ok(n) = current_num.parse::<f32>() {
                            if triple_idx < 3 {
                                current_triple[triple_idx] = n;
                            }
                            triple_idx += 1;
                        }
                        current_num.clear();
                    }
                }
                c if c.is_ascii_digit() || c == '.' || c == '-' || c == 'e' || c == 'E' => {
                    current_num.push(c);
                }
                _ => {
                    // Whitespace or other chars
                }
            }
        }

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    /// Parse a color array from USDA format
    pub(crate) fn parse_color_array(line: &str) -> Option<Vec<[f32; 4]>> {
        // Similar to vec3 but with possible alpha
        if let Some(vec3s) = Self::parse_vec3_array(line) {
            // Convert to vec4 with alpha = 1.0
            Some(vec3s.into_iter().map(|v| [v[0], v[1], v[2], 1.0]).collect())
        } else {
            None
        }
    }

    /// Parse an int array from USDA format (handles multi-line arrays)
    pub(crate) fn parse_int_array(line: &str) -> Option<Vec<u32>> {
        // We must skip past the type declaration "int[]" to avoid breaking
        let data_start = if let Some(eq_pos) = line.find('=') {
            &line[eq_pos + 1..]
        } else {
            line
        };

        let mut result = Vec::new();
        let mut current_num = String::new();

        for c in data_start.chars() {
            match c {
                ',' | ']' | ' ' | '\t' | '\r' | '\n' => {
                    if !current_num.is_empty() {
                        if let Ok(n) = current_num.parse::<i32>() {
                            if n >= 0 {
                                result.push(n as u32);
                            }
                        }
                        current_num.clear();
                    }
                }
                c if c.is_ascii_digit() || c == '-' => {
                    current_num.push(c);
                }
                _ => {} // Ignore '[' and other chars
            }
        }

        // Push any remaining number at the end of the line
        if !current_num.is_empty() {
            if let Ok(n) = current_num.parse::<i32>() {
                if n >= 0 {
                    result.push(n as u32);
                }
            }
        }

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    #[cfg(test)]
    pub(crate) fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        haystack
            .windows(needle.len())
            .position(|window| window == needle)
    }
}
