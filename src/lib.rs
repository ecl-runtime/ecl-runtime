use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone)]
pub enum ConfigValue {
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<ConfigValue>),
    Map(HashMap<String, ConfigValue>),
}

#[derive(Debug, Error)]
pub enum ECLError {
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Type error: {0}")]
    TypeError(String),
    #[error("Missing key: {0}")]
    MissingKey(String),
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),
}

pub struct ECLRuntime;

impl ECLRuntime {
    pub fn load_yaml(path: &str) -> Result<ConfigValue, ECLError> {
        let content = std::fs::read_to_string(path)?;
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(&content)?;
        Self::convert_value(yaml_value)
    }

    pub fn load_json(path: &str) -> Result<ConfigValue, ECLError> {
        let content = std::fs::read_to_string(path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| ECLError::ParseError(e.to_string()))?;
        Self::convert_json_value(json_value)
    }

    pub fn load_env(path: &str) -> Result<ConfigValue, ECLError> {
        let content = std::fs::read_to_string(path)?;
        let mut map = HashMap::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() != 2 {
                continue;
            }
            let key = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();
            let config_value = if value == "true" {
                ConfigValue::Boolean(true)
            } else if value == "false" {
                ConfigValue::Boolean(false)
            } else if let Ok(i) = value.parse::<i64>() {
                ConfigValue::Integer(i)
            } else if let Ok(f) = value.parse::<f64>() {
                ConfigValue::Float(f)
            } else {
                ConfigValue::String(value)
            };
            map.insert(key, config_value);
        }
        Ok(ConfigValue::Map(map))
    }

    fn convert_value(value: serde_yaml::Value) -> Result<ConfigValue, ECLError> {
        match value {
            serde_yaml::Value::Bool(b) => Ok(ConfigValue::Boolean(b)),
            serde_yaml::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(ConfigValue::Integer(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(ConfigValue::Float(f))
                } else {
                    Err(ECLError::TypeError("Invalid number".to_string()))
                }
            }
            serde_yaml::Value::String(s) => Ok(ConfigValue::String(s)),
            serde_yaml::Value::Sequence(seq) => {
                let values: Result<Vec<ConfigValue>, _> = seq
                    .into_iter()
                    .map(Self::convert_value)
                    .collect();
                Ok(ConfigValue::Array(values?))
            }
            serde_yaml::Value::Mapping(map) => {
                let mut hashmap = HashMap::new();
                for (k, v) in map {
                    let key = match k {
                        serde_yaml::Value::String(s) => s,
                        _ => return Err(ECLError::TypeError("Map keys must be strings".to_string())),
                    };
                    let value = Self::convert_value(v)?;
                    hashmap.insert(key, value);
                }
                Ok(ConfigValue::Map(hashmap))
            }
            serde_yaml::Value::Null => Err(ECLError::TypeError("Null values not allowed".to_string())),
            serde_yaml::Value::Tagged(_) => Err(ECLError::TypeError("Tagged values not supported".to_string())),
        }
    }

    fn convert_json_value(value: serde_json::Value) -> Result<ConfigValue, ECLError> {
        match value {
            serde_json::Value::Bool(b) => Ok(ConfigValue::Boolean(b)),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(ConfigValue::Integer(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(ConfigValue::Float(f))
                } else {
                    Err(ECLError::TypeError("Invalid number".to_string()))
                }
            }
            serde_json::Value::String(s) => Ok(ConfigValue::String(s)),
            serde_json::Value::Array(arr) => {
                let values: Result<Vec<ConfigValue>, _> = arr
                    .into_iter()
                    .map(Self::convert_json_value)
                    .collect();
                Ok(ConfigValue::Array(values?))
            }
            serde_json::Value::Object(obj) => {
                let mut hashmap = HashMap::new();
                for (k, v) in obj {
                    let value = Self::convert_json_value(v)?;
                    hashmap.insert(k, value);
                }
                Ok(ConfigValue::Map(hashmap))
            }
            serde_json::Value::Null => Err(ECLError::TypeError("Null values not allowed".to_string())),
        }
    }
}

impl ConfigValue {
    pub fn as_bool(&self) -> Result<bool, ECLError> {
        match self {
            ConfigValue::Boolean(b) => Ok(*b),
            _ => Err(ECLError::TypeError(format!("Expected bool, got {:?}", self))),
        }
    }

    pub fn as_i64(&self) -> Result<i64, ECLError> {
        match self {
            ConfigValue::Integer(i) => Ok(*i),
            _ => Err(ECLError::TypeError(format!("Expected integer, got {:?}", self))),
        }
    }

    pub fn as_string(&self) -> Result<String, ECLError> {
        match self {
            ConfigValue::String(s) => Ok(s.clone()),
            _ => Err(ECLError::TypeError(format!("Expected string, got {:?}", self))),
        }
    }

    pub fn as_f64(&self) -> Result<f64, ECLError> {
        match self {
            ConfigValue::Float(f) => Ok(*f),
            _ => Err(ECLError::TypeError(format!("Expected float, got {:?}", self))),
        }
    }

    pub fn get(&self, key: &str) -> Result<ConfigValue, ECLError> {
        match self {
            ConfigValue::Map(m) => m.get(key)
                .cloned()
                .ok_or_else(|| ECLError::MissingKey(key.to_string())),
            _ => Err(ECLError::TypeError("Not a map".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_yaml() {
        std::fs::write("test.yaml", r#"
port: 8080
debug: true
"#).unwrap();
        let cfg = ECLRuntime::load_yaml("test.yaml").unwrap();
        assert_eq!(cfg.get("port").unwrap().as_i64().unwrap(), 8080);
        assert_eq!(cfg.get("debug").unwrap().as_bool().unwrap(), true);
        std::fs::remove_file("test.yaml").ok();
    }

    #[test]
    fn test_load_json() {
        std::fs::write("test.json", r#"
{"port":8080,"debug":true}
"#).unwrap();
        let cfg = ECLRuntime::load_json("test.json").unwrap();
        assert_eq!(cfg.get("port").unwrap().as_i64().unwrap(), 8080);
        std::fs::remove_file("test.json").ok();
    }

    #[test]
    fn test_load_env() {
        std::fs::write("test.env", "PORT=8080\nDEBUG=true").unwrap();
        let cfg = ECLRuntime::load_env("test.env").unwrap();
        assert_eq!(cfg.get("PORT").unwrap().as_i64().unwrap(), 8080);
        std::fs::remove_file("test.env").ok();
    }
}

