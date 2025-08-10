use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use web_sys::window;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SharedProgram {
    pub name: String,
    pub code: String,
}

pub struct UrlSharing;

impl UrlSharing {
    /// Encode a program into a URL-safe base64 string
    pub fn encode_program(name: &str, code: &str) -> Result<String, String> {
        let shared_program = SharedProgram {
            name: name.to_string(),
            code: code.to_string(),
        };

        let json = serde_json::to_string(&shared_program)
            .map_err(|e| format!("Failed to serialize program: {}", e))?;

        Ok(URL_SAFE_NO_PAD.encode(json.as_bytes()))
    }

    /// Decode a program from a URL-safe base64 string
    pub fn decode_program(encoded: &str) -> Result<SharedProgram, String> {
        let decoded_bytes = URL_SAFE_NO_PAD
            .decode(encoded.as_bytes())
            .map_err(|e| format!("Failed to decode base64: {}", e))?;

        let json = String::from_utf8(decoded_bytes)
            .map_err(|e| format!("Invalid UTF-8 in decoded data: {}", e))?;

        serde_json::from_str(&json).map_err(|e| format!("Failed to deserialize program: {}", e))
    }

    /// Generate a shareable URL for the current program
    pub fn generate_share_url(name: &str, code: &str) -> Result<String, String> {
        let window = window().ok_or("No window object available")?;
        let location = window.location();

        let base_url = format!(
            "{}//{}{}",
            location.protocol().map_err(|_| "Failed to get protocol")?,
            location.host().map_err(|_| "Failed to get host")?,
            location.pathname().map_err(|_| "Failed to get pathname")?
        );

        let encoded_program = Self::encode_program(name, code)?;
        Ok(format!("{}?share={}", base_url, encoded_program))
    }

    /// Extract shared program from current URL
    pub fn extract_from_url() -> Option<SharedProgram> {
        let window = window()?;
        let location = window.location();
        let search = location.search().ok()?;

        if search.is_empty() {
            return None;
        }

        // Parse URL parameters
        let params = Self::parse_url_params(&search);
        let encoded_program = params.get("share")?;

        Self::decode_program(encoded_program).ok()
    }

    /// Parse URL search parameters into a HashMap
    fn parse_url_params(search: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();

        // Remove leading '?' if present
        let search = search.strip_prefix('?').unwrap_or(search);

        for pair in search.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                // URL decode the value
                if let Ok(decoded_value) = js_sys::decode_uri_component(value) {
                    params.insert(
                        key.to_string(),
                        decoded_value.as_string().unwrap_or_default(),
                    );
                }
            }
        }

        params
    }

    /// Copy text to clipboard
    pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
        let window = window().ok_or("No window object available")?;

        // Use the clipboard API if available
        if let Ok(navigator) = js_sys::Reflect::get(&window, &"navigator".into()) {
            if let Ok(clipboard) = js_sys::Reflect::get(&navigator, &"clipboard".into()) {
                if !clipboard.is_undefined() {
                    let clipboard: web_sys::Clipboard = clipboard.into();
                    let _promise = clipboard.write_text(text);
                    return Ok(());
                }
            }
        }

        Err("Clipboard API not available".to_string())
    }
}
