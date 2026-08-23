//! Lightweight, zero-dependency JSON parser and serializer for DRL-Rust MCP.

use std::collections::BTreeMap;
use std::fmt;

/// Represents a JSON value.
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
  /// JSON `null`.
  Null,
  /// JSON boolean (`true` or `false`).
  Bool(bool),
  /// JSON numeric value.
  Number(f64),
  /// Numeric literal retained lexically when converting it to `f64` would
  /// lose integer precision. Tool validators reject this value rather than
  /// executing with a silently rounded argument.
  RawNumber(String),
  /// JSON string value.
  String(String),
  /// JSON array of values.
  Array(Vec<JsonValue>),
  /// JSON object mapping string keys to values.
  Object(BTreeMap<String, JsonValue>),
}

impl JsonValue {
  /// Returns `true` if the value is `Null`.
  #[must_use]
  pub fn is_null(&self) -> bool {
    matches!(self, Self::Null)
  }

  /// Returns the string slice if this is a `String`.
  #[must_use]
  pub fn as_str(&self) -> Option<&str> {
    match self {
      Self::String(s) => Some(s.as_str()),
      _ => None,
    }
  }

  /// Returns the boolean if this is a `Bool`.
  #[must_use]
  pub fn as_bool(&self) -> Option<bool> {
    match self {
      Self::Bool(b) => Some(*b),
      _ => None,
    }
  }

  /// Returns the number as `f64` if this is a `Number`.
  #[must_use]
  pub fn as_f64(&self) -> Option<f64> {
    match self {
      Self::Number(n) => Some(*n),
      Self::RawNumber(raw) => raw.parse().ok(),
      _ => None,
    }
  }

  /// Returns the number as `i64` if this is a `Number` and within integer range.
  #[must_use]
  pub fn as_i64(&self) -> Option<i64> {
    match self {
      Self::Number(n) if n.fract() == 0.0 => Some(*n as i64),
      Self::RawNumber(raw) => raw.parse().ok(),
      _ => None,
    }
  }

  /// Returns the number as `u64` if this is a `Number` and a non-negative integer.
  #[must_use]
  pub fn as_u64(&self) -> Option<u64> {
    match self {
      Self::Number(n) if *n >= 0.0 && n.fract() == 0.0 => Some(*n as u64),
      Self::RawNumber(raw) => raw.parse().ok(),
      _ => None,
    }
  }

  /// Returns the number as `u32` if this is a `Number` and a valid `u32`.
  #[must_use]
  pub fn as_u32(&self) -> Option<u32> {
    self.as_u64().and_then(|v| u32::try_from(v).ok())
  }

  /// Returns a reference to the array if this is an `Array`.
  #[must_use]
  pub fn as_array(&self) -> Option<&Vec<JsonValue>> {
    match self {
      Self::Array(arr) => Some(arr),
      _ => None,
    }
  }

  /// Returns a mutable reference to the array if this is an `Array`.
  pub fn as_array_mut(&mut self) -> Option<&mut Vec<JsonValue>> {
    match self {
      Self::Array(arr) => Some(arr),
      _ => None,
    }
  }

  /// Returns a reference to the object if this is an `Object`.
  #[must_use]
  pub fn as_object(&self) -> Option<&BTreeMap<String, JsonValue>> {
    match self {
      Self::Object(obj) => Some(obj),
      _ => None,
    }
  }

  /// Returns a mutable reference to the object if this is an `Object`.
  pub fn as_object_mut(&mut self) -> Option<&mut BTreeMap<String, JsonValue>> {
    match self {
      Self::Object(obj) => Some(obj),
      _ => None,
    }
  }

  /// Looks up a key in an object.
  #[must_use]
  pub fn get(&self, key: &str) -> Option<&JsonValue> {
    self.as_object().and_then(|obj| obj.get(key))
  }

  /// Parses a JSON string into a `JsonValue`.
  pub fn parse(input: &str) -> Result<Self, String> {
    let mut parser = JsonParser::new(input);
    let value = parser.parse_value()?;
    parser.skip_whitespace();
    if !parser.is_eof() {
      return Err(format!(
        "Unexpected trailing characters at index {}",
        parser.pos
      ));
    }
    Ok(value)
  }

  /// Serializes this JSON value into a compact JSON string.
  #[must_use]
  pub fn to_compact_string(&self) -> String {
    let mut out = String::new();
    self.write_compact(&mut out);
    out
  }

  fn write_compact(&self, out: &mut String) {
    match self {
      Self::Null => out.push_str("null"),
      Self::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
      Self::Number(n) => {
        if n.is_nan() || n.is_infinite() {
          out.push_str("null");
        } else if n.fract() == 0.0 && *n >= (i64::MIN as f64) && *n <= (u64::MAX as f64) {
          if *n < 0.0 {
            out.push_str(&format!("{}", *n as i64));
          } else {
            out.push_str(&format!("{}", *n as u64));
          }
        } else {
          out.push_str(&format!("{n}"));
        }
      }
      Self::RawNumber(raw) => out.push_str(raw),
      Self::String(s) => {
        out.push('"');
        for c in s.chars() {
          match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0C}' => out.push_str("\\f"),
            other if other.is_control() => {
              out.push_str(&format!("\\u{:04x}", other as u32));
            }
            other => out.push(other),
          }
        }
        out.push('"');
      }
      Self::Array(items) => {
        out.push('[');
        for (i, item) in items.iter().enumerate() {
          if i > 0 {
            out.push(',');
          }
          item.write_compact(out);
        }
        out.push(']');
      }
      Self::Object(entries) => {
        out.push('{');
        for (i, (key, value)) in entries.iter().enumerate() {
          if i > 0 {
            out.push(',');
          }
          Self::String(key.clone()).write_compact(out);
          out.push(':');
          value.write_compact(out);
        }
        out.push('}');
      }
    }
  }
}

impl fmt::Display for JsonValue {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.to_compact_string())
  }
}

impl From<&str> for JsonValue {
  fn from(s: &str) -> Self {
    Self::String(s.to_string())
  }
}

impl From<String> for JsonValue {
  fn from(s: String) -> Self {
    Self::String(s)
  }
}

impl From<bool> for JsonValue {
  fn from(b: bool) -> Self {
    Self::Bool(b)
  }
}

impl From<u32> for JsonValue {
  fn from(n: u32) -> Self {
    Self::Number(f64::from(n))
  }
}

impl From<u64> for JsonValue {
  fn from(n: u64) -> Self {
    Self::Number(n as f64)
  }
}

impl From<i32> for JsonValue {
  fn from(n: i32) -> Self {
    Self::Number(f64::from(n))
  }
}

impl From<f64> for JsonValue {
  fn from(n: f64) -> Self {
    Self::Number(n)
  }
}

/// Returns whether a JSON numeric literal is an integer larger than the range
/// that `f64` can represent without losing integer precision.
fn number_exceeds_exact_integer_range(raw: &str) -> bool {
  const SAFE_INTEGER_MAX: &str = "9007199254740992";

  let unsigned = raw.strip_prefix('-').unwrap_or(raw);
  let (mantissa, exponent) = match unsigned.split_once(['e', 'E']) {
    Some((mantissa, exponent)) => {
      let exponent = match exponent.parse::<i64>() {
        Ok(exponent) => exponent,
        Err(_) => return true,
      };
      (mantissa, exponent)
    }
    None => (unsigned, 0),
  };
  let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
  let digits = format!("{whole}{fraction}");
  let decimal_index = whole.len() as i64 + exponent;
  if decimal_index <= 0 {
    return false;
  }

  let digit_count = digits.len() as i64;
  if decimal_index < digit_count
    && digits[decimal_index as usize..]
      .bytes()
      .any(|digit| digit != b'0')
  {
    return false;
  }

  let integer_end = decimal_index.min(digit_count) as usize;
  let significant = digits[..integer_end].trim_start_matches('0');
  if significant.is_empty() {
    return false;
  }

  let trailing_zeros_count = (decimal_index - digit_count).max(0);
  if trailing_zeros_count > SAFE_INTEGER_MAX.len() as i64 {
    return true;
  }
  let trailing_zeros = trailing_zeros_count as usize;
  let integer_length = significant.len() + trailing_zeros;
  if integer_length != SAFE_INTEGER_MAX.len() {
    return integer_length > SAFE_INTEGER_MAX.len();
  }
  if trailing_zeros == 0 {
    return significant > SAFE_INTEGER_MAX;
  }

  let mut padded = String::with_capacity(integer_length);
  padded.push_str(significant);
  padded.extend(std::iter::repeat_n('0', trailing_zeros));
  padded.as_str() > SAFE_INTEGER_MAX
}

struct JsonParser<'a> {
  chars: Vec<char>,
  pos: usize,
  _phantom: std::marker::PhantomData<&'a str>,
}

impl<'a> JsonParser<'a> {
  fn new(input: &'a str) -> Self {
    Self {
      chars: input.chars().collect(),
      pos: 0,
      _phantom: std::marker::PhantomData,
    }
  }

  fn is_eof(&self) -> bool {
    self.pos >= self.chars.len()
  }

  fn peek(&self) -> Option<char> {
    self.chars.get(self.pos).copied()
  }

  fn advance(&mut self) -> Option<char> {
    let c = self.peek();
    if c.is_some() {
      self.pos += 1;
    }
    c
  }

  fn skip_whitespace(&mut self) {
    while let Some(c) = self.peek() {
      if c.is_whitespace() {
        self.advance();
      } else {
        break;
      }
    }
  }

  fn parse_value(&mut self) -> Result<JsonValue, String> {
    self.skip_whitespace();
    match self.peek() {
      Some('n') => self.parse_null(),
      Some('t') | Some('f') => self.parse_bool(),
      Some('"') => self.parse_string().map(JsonValue::String),
      Some('[') => self.parse_array(),
      Some('{') => self.parse_object(),
      Some(c) if c == '-' || c.is_ascii_digit() => self.parse_number(),
      Some(other) => Err(format!(
        "Unexpected character '{other}' at index {} in JSON input",
        self.pos
      )),
      None => Err("Unexpected end of JSON input".to_string()),
    }
  }

  fn parse_null(&mut self) -> Result<JsonValue, String> {
    if self.consume_str("null") {
      Ok(JsonValue::Null)
    } else {
      Err(format!("Expected 'null' at index {}", self.pos))
    }
  }

  fn parse_bool(&mut self) -> Result<JsonValue, String> {
    if self.consume_str("true") {
      Ok(JsonValue::Bool(true))
    } else if self.consume_str("false") {
      Ok(JsonValue::Bool(false))
    } else {
      Err(format!("Expected boolean at index {}", self.pos))
    }
  }

  fn consume_str(&mut self, expected: &str) -> bool {
    let len = expected.chars().count();
    if self.pos + len <= self.chars.len() {
      let slice: String = self.chars[self.pos..self.pos + len].iter().collect();
      if slice == expected {
        self.pos += len;
        return true;
      }
    }
    false
  }

  fn parse_string(&mut self) -> Result<String, String> {
    if self.advance() != Some('"') {
      return Err(format!("Expected '\"' at index {}", self.pos));
    }
    let mut out = String::new();
    while let Some(c) = self.advance() {
      match c {
        '"' => return Ok(out),
        '\\' => {
          let esc = self
            .advance()
            .ok_or_else(|| "Unfinished escape in string".to_string())?;
          match esc {
            '"' => out.push('"'),
            '\\' => out.push('\\'),
            '/' => out.push('/'),
            'b' => out.push('\u{08}'),
            'f' => out.push('\u{0C}'),
            'n' => out.push('\n'),
            'r' => out.push('\r'),
            't' => out.push('\t'),
            'u' => {
              let mut hex = String::with_capacity(4);
              for _ in 0..4 {
                let h = self
                  .advance()
                  .ok_or_else(|| "Unfinished unicode escape".to_string())?;
                hex.push(h);
              }
              let code = u32::from_str_radix(&hex, 16)
                .map_err(|e| format!("Invalid hex unicode escape \\u{hex}: {e}"))?;
              let decoded_char = char::from_u32(code)
                .ok_or_else(|| format!("Invalid unicode char from code point: {code}"))?;
              out.push(decoded_char);
            }
            other => return Err(format!("Invalid escape sequence '\\{other}'")),
          }
        }
        other => out.push(other),
      }
    }
    Err("Unterminated string literal".to_string())
  }

  fn parse_number(&mut self) -> Result<JsonValue, String> {
    let start = self.pos;
    if self.peek() == Some('-') {
      self.advance();
    }
    let mut has_digits = false;
    while let Some(c) = self.peek() {
      if c.is_ascii_digit() {
        has_digits = true;
        self.advance();
      } else {
        break;
      }
    }
    if !has_digits {
      return Err(format!("Invalid number format at index {start}"));
    }

    if self.peek() == Some('.') {
      self.advance();
      while let Some(c) = self.peek() {
        if c.is_ascii_digit() {
          self.advance();
        } else {
          break;
        }
      }
    }

    if let Some('e') | Some('E') = self.peek() {
      self.advance();
      if let Some('+') | Some('-') = self.peek() {
        self.advance();
      }
      while let Some(c) = self.peek() {
        if c.is_ascii_digit() {
          self.advance();
        } else {
          break;
        }
      }
    }

    let raw: String = self.chars[start..self.pos].iter().collect();
    let num: f64 = raw
      .parse()
      .map_err(|e| format!("Failed to parse number '{raw}': {e}"))?;
    if number_exceeds_exact_integer_range(&raw) {
      return Ok(JsonValue::RawNumber(raw));
    }
    Ok(JsonValue::Number(num))
  }

  fn parse_array(&mut self) -> Result<JsonValue, String> {
    if self.advance() != Some('[') {
      return Err(format!("Expected '[' at index {}", self.pos));
    }
    self.skip_whitespace();
    if self.peek() == Some(']') {
      self.advance();
      return Ok(JsonValue::Array(Vec::new()));
    }

    let mut items = Vec::new();
    loop {
      let val = self.parse_value()?;
      items.push(val);
      self.skip_whitespace();
      match self.peek() {
        Some(',') => {
          self.advance();
        }
        Some(']') => {
          self.advance();
          break;
        }
        Some(other) => {
          return Err(format!(
            "Expected ',' or ']' in array at index {}, found '{other}'",
            self.pos
          ));
        }
        None => return Err("Unterminated array literal".to_string()),
      }
    }
    Ok(JsonValue::Array(items))
  }

  fn parse_object(&mut self) -> Result<JsonValue, String> {
    if self.advance() != Some('{') {
      return Err(format!("Expected '{{' at index {}", self.pos));
    }
    self.skip_whitespace();
    if self.peek() == Some('}') {
      self.advance();
      return Ok(JsonValue::Object(BTreeMap::new()));
    }

    let mut entries = BTreeMap::new();
    loop {
      self.skip_whitespace();
      let key = self.parse_string()?;
      self.skip_whitespace();
      if self.advance() != Some(':') {
        return Err(format!("Expected ':' after key at index {}", self.pos));
      }
      let val = self.parse_value()?;
      entries.insert(key, val);
      self.skip_whitespace();
      match self.peek() {
        Some(',') => {
          self.advance();
        }
        Some('}') => {
          self.advance();
          break;
        }
        Some(other) => {
          return Err(format!(
            "Expected ',' or '}}' in object at index {}, found '{other}'",
            self.pos
          ));
        }
        None => return Err("Unterminated object literal".to_string()),
      }
    }
    Ok(JsonValue::Object(entries))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_json_primitives() {
    assert_eq!(JsonValue::parse("null").unwrap(), JsonValue::Null);
    assert_eq!(JsonValue::parse("true").unwrap(), JsonValue::Bool(true));
    assert_eq!(JsonValue::parse("false").unwrap(), JsonValue::Bool(false));
    assert_eq!(JsonValue::parse("123").unwrap(), JsonValue::Number(123.0));
    assert_eq!(JsonValue::parse("-45.5").unwrap(), JsonValue::Number(-45.5));
    assert_eq!(
      JsonValue::parse("9007199254740993").unwrap(),
      JsonValue::RawNumber("9007199254740993".to_string())
    );
    assert_eq!(
      JsonValue::parse("9007199254740992.0").unwrap(),
      JsonValue::Number(9_007_199_254_740_992.0)
    );
    assert_eq!(
      JsonValue::parse("9007199254740993.0").unwrap(),
      JsonValue::RawNumber("9007199254740993.0".to_string())
    );
    assert_eq!(
      JsonValue::parse("\"hello\\nworld\"").unwrap(),
      JsonValue::String("hello\nworld".to_string())
    );
  }

  #[test]
  fn test_json_composite() {
    let raw = r#"{"name":"Imp","hp":15,"active":true,"tags":["demon","ranged"]}"#;
    let val = JsonValue::parse(raw).unwrap();
    assert_eq!(val.get("name").unwrap().as_str().unwrap(), "Imp");
    assert_eq!(val.get("hp").unwrap().as_u32().unwrap(), 15);
    assert!(val.get("active").unwrap().as_bool().unwrap());
    assert_eq!(val.get("tags").unwrap().as_array().unwrap().len(), 2);
  }

  #[test]
  fn test_json_roundtrip() {
    let mut map = BTreeMap::new();
    map.insert("command".to_string(), JsonValue::from("Move"));
    map.insert("step".to_string(), JsonValue::from(42));
    let val = JsonValue::Object(map);
    let s = val.to_compact_string();
    let parsed = JsonValue::parse(&s).unwrap();
    assert_eq!(val, parsed);
  }
}
