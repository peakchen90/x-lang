use ansi_term::Colour;
use std::cmp::max;
use crate::externs;

pub enum CodeFrameMessageType {
    Warn,
    Error,
}

// 生成 n 个长度的空白字符
pub fn gen_space_str(width: usize) -> String {
    [' '].repeat(width).iter().collect()
}

// 返回无符号整数的位数
pub fn get_uint_width(value: usize) -> usize {
    let mut width = 1;
    let mut n = value;
    while n >= 10 {
        width += 1;
        n /= 10;
    }
    width
}

// 返回固定宽度的数字字符
pub fn pad_str(value: &str, width: usize, pad_char: char) -> String {
    let mut result = String::new();
    let mut gap = width as isize - value.len() as isize;
    if gap >= 0 {
        for _ in 0..gap {
            result.push(pad_char);
        }
        result.push_str(value);
    } else {
        let gap = -gap as usize;
        for (i, ch) in value.chars().enumerate() {
            if i >= gap {
                result.push(ch);
            }
        }
    }
    result
}

// 返回在源代码中的行列信息
pub fn get_source_position(source: &str, index: usize) -> Option<(usize, usize)> {
    let mut position = (1usize, 1usize);
    let source_chars = source.chars().collect::<Vec<char>>();
    for (i, ch) in source_chars.iter().enumerate() {
        if i == index {
            return Some(position);
        }
        if *ch == '\n' {
            position.0 += 1;
            position.1 = 1;
        } else {
            position.1 += 1;
        }
    }
    None
}

// 打印代码帧信息，返回目标位置的行列信息
pub fn print_code_frame(
    source: &str,
    pos: usize,
    message: &str,
    msg_type: CodeFrameMessageType,
) -> Option<(usize, usize)> {
    let mut before_lines = vec![];
    let mut after_lines = vec![];

    // 分割提示信息的前后代码片段（打印目标位置，上面3行，下面2行）
    let code_lines = source.lines().collect::<Vec<&str>>();
    let last_line = code_lines.len();
    let last_column = code_lines.last().map_or(0, |s| s.len());
    let source_target = get_source_position(source, pos);
    let target = source_target.map_or((last_line, last_column + 1), |v| v);
    let target_line = target.0 as isize;
    for (i, str) in code_lines.iter().enumerate() {
        let i = i as isize;
        if i >= target_line - 3 && i < target_line {
            before_lines.push(str.to_string());
        } else if i >= target_line && i < target_line + 2 {
            after_lines.push(str.to_string());
        }
    }

    let primary_color = match msg_type {
        CodeFrameMessageType::Warn => Colour::RGB(180, 140, 20),
        CodeFrameMessageType::Error => Colour::RGB(255, 30, 30),
    };
    let code_color = match msg_type {
        CodeFrameMessageType::Warn => Colour::RGB(160, 140, 70),
        CodeFrameMessageType::Error => Colour::RGB(180, 0, 0),
    };

    // 行号数字长度 (最大行号为目标位置加上下面的2行)
    let line_no_width = get_uint_width(target_line as usize + 2);
    let mut iter_line = max(target_line - 3, 0);

    // 打印提示信息前面代码
    for str in before_lines.iter() {
        iter_line += 1;
        let line_no = pad_str(&iter_line.to_string(), line_no_width, ' ');

        #[cfg(not(feature = "wasm"))]
        {
            print!("{}", code_color.bold().paint(format!("{} | ", line_no)));
            println!("{}", code_color.paint(str));
        }
        #[cfg(feature = "wasm")]
        externs::__logError__(&format!("{} | {}", line_no, str));
    }

    // 打印提示信息（需预留行号空白位置）
    let mut message_str = gen_space_str((target.1 - 1) + (line_no_width + 3));
    message_str.push_str("^ ");
    message_str.push_str(message);

    #[cfg(not(feature = "wasm"))]
    println!("{}", primary_color.bold().paint(&message_str));
    #[cfg(feature = "wasm")]
    externs::__logError__(&message_str);

    // 打印提示信息后面代码
    for str in after_lines.iter() {
        iter_line += 1;
        let line_no = pad_str(&iter_line.to_string(), line_no_width, ' ');

        #[cfg(not(feature = "wasm"))]
        {
            print!("{}", code_color.bold().paint(format!("{} | ", line_no)));
            println!("{}", code_color.paint(str));
        }
        #[cfg(feature = "wasm")]
        externs::__logError__(&format!("{} | {}", line_no, str));
    }

    source_target
}

// 打印警告代码帧信息
pub fn print_warn_frame(
    source: &str,
    pos: usize,
    message: &str,
) -> Option<(usize, usize)> {
    print_code_frame(source, pos, message, CodeFrameMessageType::Warn)
}

// 打印错误代码帧信息
pub fn print_error_frame(
    source: &str,
    pos: usize,
    message: &str,
) -> Option<(usize, usize)> {
    print_code_frame(source, pos, message, CodeFrameMessageType::Error)
}
