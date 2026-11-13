pub mod resp;
pub mod database;

use std::{collections::HashMap, io::{Read, Write}, net::TcpListener, thread};

use anyhow::Ok;

use crate::{database::DatabaseRef, resp::RESPValue};

fn main() {
    let _ = run_server();
}


fn run_server() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    let database = DatabaseRef::default();

    for stream in listener.incoming() {
        let mut stream = stream?;
        let database = database.clone();
        
        thread::spawn(move || {
            if let Err(e) = handle_client(&mut stream, database) {
                println!("Error handling client: {}", e);
            }
        });
    }
    Ok(())
}

fn handle_client(stream: &mut std::net::TcpStream, database: DatabaseRef) -> anyhow::Result<()> {
    let mut reader: [u8; 1024] = [0; 1024];

    let mut handlers = HashMap::<&str, fn(&CommandContext) -> anyhow::Result<RESPValue>>::new();
    handlers.insert("PING", handle_ping);
    handlers.insert("ECHO", handle_echo);
    handlers.insert("GET", handle_get);
    handlers.insert("SET", handle_set);
    handlers.insert("RPUSH", handle_rpush);
    handlers.insert("LPUSH", handle_lpush);
    handlers.insert("LRANGE", handle_lrange);
    handlers.insert("LLEN", handle_llen);
    handlers.insert("LPOP", handle_lpop);
    handlers.insert("BLPOP", handle_blpop);

    let mut context = CommandContext {
        command: vec![],
        db: database.clone(),
    };

    loop {
        let len = stream.read(&mut reader)?;
        let resp_value = RESPValue::try_deserialize(&reader[..len])?;

        match resp_value {
            RESPValue::Array(items) => {

                context.command.clear();
                context.command.extend(items.clone());

                let mut iter = items.iter();

                if let Some(command) = iter.next() {
                    let cmd = command.to_uppercase();
                    
                    if let Some(handler) = handlers.get(cmd.as_str()) {
                        let answer = handler(&context)?;
                        stream.write_all(answer.serialize().as_bytes())?;
                    } else {
                        let answer = RESPValue::SimpleString("OK".to_string());
                        stream.write_all(answer.serialize().as_bytes())?;
                    }
                }
            },
            _ => todo!(),
        }

        stream.flush()?;
    }
}

pub struct CommandContext {
    pub command: Vec<String>,
    pub db: DatabaseRef
}

fn handle_ping(_ctx: &CommandContext) -> anyhow::Result<RESPValue> {
    let answer = RESPValue::SimpleString("PONG".to_string());
    Ok(answer)
}

fn handle_echo(ctx: &CommandContext) -> anyhow::Result<RESPValue> {
    Ok(RESPValue::BulkString(ctx.command.get(1).unwrap().clone()))
}

fn handle_get(ctx: &CommandContext) -> anyhow::Result<RESPValue> {
    let key = ctx.command.get(1).unwrap().clone();
    let answer = ctx.db.get(key);

    let answer = match answer {
        Some(value) => RESPValue::BulkString(value),
        None => RESPValue::Nil,
    };
    Ok(answer)
}

fn handle_set(ctx: &CommandContext) -> anyhow::Result<RESPValue> {
    let key = ctx.command.get(1).unwrap().clone();
    let value = ctx.command.get(2).unwrap().clone();

    if let Some(expire_key) = ctx.command.get(3) {
        let expire_key = expire_key.to_uppercase();
        let expire_value = ctx.command.get(4).unwrap().clone().parse::<u64>()?;

        let expire_at = match expire_key.as_str() {
            "EX" => Some(std::time::Instant::now() + std::time::Duration::from_secs(expire_value)),
            "PX" => Some(std::time::Instant::now() + std::time::Duration::from_millis(expire_value)),
            _ => None,
        };

        ctx.db.set(key, value, expire_at)?;
    } else {
        ctx.db.set(key, value, None)?;
    }

    let answer = RESPValue::SimpleString("OK".to_string());
    Ok(answer)
}

fn handle_rpush(ctx: &CommandContext) -> anyhow::Result<RESPValue> {
    let key = ctx.command.get(1).unwrap().clone();

    let mut len = 0;
    for value in ctx.command.iter().skip(2) {
        len = ctx.db.append_to_list(key.clone(), value.clone())?;
    }
    
    Ok(RESPValue::Integer(len.to_string()))
}

fn handle_lpush(ctx: &CommandContext) -> anyhow::Result<RESPValue> {
    let key = ctx.command.get(1).unwrap().clone();

    let mut len = 0;
    for value in ctx.command.iter().skip(2) {
        len = ctx.db.prepend_to_list(key.clone(), value.clone())?;
    }
    
    Ok(RESPValue::Integer(len.to_string()))
}

fn handle_lrange(ctx: &CommandContext) -> anyhow::Result<RESPValue> {
    let key = ctx.command.get(1).unwrap().clone();
    let start: isize = ctx.command.get(2).unwrap().clone().parse()?;
    let stop: isize = ctx.command.get(3).unwrap().clone().parse()?;

    let list = ctx.db.list_range(key, start, stop)?;

    Ok(RESPValue::Array(list))
}

fn handle_llen(ctx: &CommandContext) -> anyhow::Result<RESPValue> {
    let key = ctx.command.get(1).unwrap().clone();

    let len = ctx.db.list_len(key)?;

    Ok(RESPValue::Integer(len.to_string()))
}

fn handle_lpop(ctx: &CommandContext) -> anyhow::Result<RESPValue> {
    let key = ctx.command.get(1).unwrap().clone();
    if let Some(count_str) = ctx.command.get(2) {
        let count = count_str.parse::<usize>()?;
        let values = ctx.db.left_pop_multiple(key, count)?;
        let answer = RESPValue::Array(values);
        Ok(answer)
    } else {
        let value = ctx.db.left_pop(key)?;
    
        let answer = match value {
            Some(value) => RESPValue::BulkString(value),
            None => RESPValue::Nil,
        };
        Ok(answer)
    }
}

fn handle_blpop(ctx: &CommandContext) -> anyhow::Result<RESPValue> {
    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        let value = ctx.db.left_pop(ctx.command.get(1).unwrap().clone())?;
        if let Some(value) = value {
            return Ok(RESPValue::Array(vec![ctx.command.get(1).unwrap().clone(), value]));
        }
    }
}