use std::io::{BufRead, BufReader, Cursor};

use anyhow::{Ok, Result};



#[derive(Debug, Clone)]
pub enum RESPValue {
    SimpleString(String),
    BulkString(String),
    Array(Vec<String>),
    Integer(String),
    Nil,
}

impl RESPValue {
    pub fn serialize(self) -> String {
        match self {
            RESPValue::SimpleString(value) => format!("+{}\r\n", value),
            RESPValue::BulkString(value) => format!("${}\r\n{}\r\n", value.len(), value),
            RESPValue::Array(items) => {
                let mut result = format!("*{}\r\n", items.len());
                for item in items {
                    result.push_str(&format!("${}\r\n{}\r\n", item.len(), item));
                }
                result
            },
            RESPValue::Integer(value) => format!(":{}\r\n", value),
            RESPValue::Nil => "$-1\r\n".to_string(),
        }
    }

    pub fn try_deserialize(input: &[u8]) -> Result<Self> {
        let cursor = Cursor::new(input);
        let mut reader = BufReader::new(cursor);
        let mut buf = String::new();

        
        reader.read_line(&mut buf)?;
        
        // Handling array
        if buf.starts_with('*') {
            
            let mut result = Vec::new();

            let length = buf[1..].trim().parse::<usize>()?;
            buf.clear();

            for _ in 0..length {
                
                // Reading redundant line with $<length> for each element, we can ignore it for now
                reader.read_line(&mut buf)?;
                buf.clear();
                reader.read_line(&mut buf)?;

                result.push(buf.trim().to_string());
                buf.clear();
            }

            Ok(RESPValue::Array(result))
        } else {
            todo!()
        }
    }
}