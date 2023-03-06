use std::env;
use std::io::{stdin, stdout, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};

fn main() {
    loop {
        // 使用 `>` 作为提示
        print!("> ");
        // 显式地刷新它，这样确保它在 read_line 之前打印
        stdout().flush().unwrap();

        // 建一个新字符串，用于保存用户输入
        let mut input = String::new();
        // stdin().read_line 将会在用户输入处阻塞，直到用户按下回车键，然后它将整个用户输入的内容（包括回车键的空行）写入字符串
        stdin().read_line(&mut input).unwrap();

        // read_line 会在最后留下一个换行符，在处理用户的输入后会被删除
        // 使用 input.trim() 删除换行符等空白符
        let mut commands = input.trim().split(" | ").peekable();
        let mut previous_command = None;

        while let Some(command) = commands.next() {
            // everything after the first whitespace character is interpreted as args to the command
            // 通过将用户输入拆分为空格字符，并将第一个空格之前的内容作为命令的名称
            let mut parts = command.trim().split_whitespace();
            // 而将第一个空格之后的内容作为参数传递给该命令
            let command = parts.next().unwrap();
            let args = parts;

            match command {
                // 添加 shell 内建功能 cd 功能到我们的 shell 中
                "cd" => {
                    // 如果没有提供路径参数，则默认 '/' 路径
                    let new_dir = args.peekable().peek().map_or("/", |x| *x);
                    let root = Path::new(new_dir);
                    if let Err(e) = env::set_current_dir(&root) {
                        eprintln!("{}", e);
                    }

                    previous_command = None;
                }
                "exit" => return,
                command => {
                    let stdin = previous_command.map_or(Stdio::inherit(), |output: Child| {
                        Stdio::from(output.stdout.unwrap())
                    });

                    let stdout = if commands.peek().is_some() {
                        // there is another command piped behind this one
                        // prepare to send output to the next command
                        Stdio::piped()
                    } else {
                        // there are no more commands piped behind this one
                        // send output to shell stdout
                        Stdio::inherit()
                    };

                    let output = Command::new(command)
                        .args(args)
                        .stdin(stdin)
                        .stdout(stdout)
                        .spawn();

                    match output {
                        Ok(output) => {
                            previous_command = Some(output);
                        }
                        Err(e) => {
                            previous_command = None;
                            eprintln!("{}", e);
                        }
                    };
                }
            }
        }

        if let Some(mut final_command) = previous_command {
            // block until the final command has finished
            // 添加调用 wait 来等待每个子命令的处理，以确保我们不会在当前处理完成之前，提示用户输入额外的信息
            final_command.wait().unwrap();
        }
    }
}