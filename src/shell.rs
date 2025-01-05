
use core::str::{from_utf8, Split};

use crate::io::uart;

static MAX_COMMAND_LEN: usize = 128;

pub fn run_shell() {
    loop {
        uart::puts("bk> ");

        let mut cmd_buf = [0u8; MAX_COMMAND_LEN];

        let cmd_len = uart::gets(&mut cmd_buf);
        let cmd_str = from_utf8(&cmd_buf[..cmd_len]);

        if let Ok(cmd) = cmd_str {
            let exit = parse_and_execute(cmd);
            if exit {
                break;
            }
        } else {
            uart::puts("Invalid command. Must be valid utf8.\r\n");
        }
    }
}

fn parse_and_execute(cmd: &str) -> bool {
    if cmd.is_empty() {
        uart::puts("\r\n");
        return false;
    }

    let mut words = cmd.split(' ');
    let cmd = words.next();
    if let Some(cmd) = cmd {
        for (name, func) in COMMANDS {
            if name == cmd {
                return func(words);
            }
        }
        uart::puts("Unrecognized command: '");
        uart::puts(cmd);
        uart::puts("'\r\n");
    } else {
        // no command
        uart::puts("\r\n");
    }
    false
}

const COMMANDS: [(&str, fn(Split<'_, char>) -> bool); 2] = [
    ("echo", echo),
    ("exe", exe),
];

fn echo(args: Split<'_, char>) -> bool {
    for arg in args {
        uart::puts(arg);
        uart::putb(b' ');
    }
    uart::puts("\r\n");
    false
}

fn exe(args: Split<'_, char>) -> bool {
    false
}
