use wasm_bindgen::prelude::*;

use prompt_shield::{Scanner, default_config, parse_config};

#[wasm_bindgen(js_name = scan)]
pub fn scan_with_default(text: &str) -> JsValue {
    let config = default_config();
    let scanner = Scanner::new(&config);
    let result = scanner.scan(text);
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen(js_name = scanWithConfig)]
pub fn scan_with_config(text: &str, config_toml: &str) -> Result<JsValue, JsError> {
    let config = parse_config(config_toml).map_err(|e| JsError::new(&e.to_string()))?;
    let scanner = Scanner::new(&config);
    let result = scanner.scan(text);
    serde_wasm_bindgen::to_value(&result).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = defaultConfigToml)]
pub fn default_config_toml() -> String {
    include_str!("../../../config/default.toml").to_string()
}

#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
