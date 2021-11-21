use ansi_term::Colour;

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
        for i in 0..gap {
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

// 打印代码帧信息
pub fn print_code_frame(
    source: &str,
    position: usize,
    message: &str,
    msg_type: CodeFrameMessageType,
) {
    let mut line = 1usize;
    let mut column = 1usize;
    let mut target = (0usize, 0usize);
    let mut current = String::new();
    let mut before_lines = vec![];
    let mut after_lines = vec![];

    let source_chars = source.chars().collect::<Vec<char>>();
    for (i, ch) in source_chars.iter().enumerate() {
        if i == position {
            target = (line, column);
        }
        if *ch == '\n' {
            if target.0 < 1 || line <= target.0 {
                before_lines.push(current);
            } else {
                after_lines.push(current);
            }
            current = String::new();
            line += 1;
            column = 1;
        } else {
            current.push(*ch);
            column += 1;
        }
    }
    // 最后一行
    if target.0 < 1 || line <= target.0 {
        before_lines.push(current);
    } else {
        after_lines.push(current);
    }

    let primary_color = match msg_type {
        CodeFrameMessageType::Warn => Colour::RGB(180, 140, 20),
        CodeFrameMessageType::Error => Colour::RGB(255, 30, 30),
    };
    let code_color = match msg_type {
        CodeFrameMessageType::Warn => Colour::RGB(160, 140, 70),
        CodeFrameMessageType::Error => Colour::RGB(180, 0, 0),
    };

    // 打印目标位置，上面3行，下面2行
    let line_no_width = get_uint_width(target.0 + 2);

    let mut iter_line = 0usize;
    // 提示信息前面代码
    for (i, str) in before_lines.iter().enumerate() {
        iter_line += 1;
        if i >= before_lines.len() - 3 {
            let line_no = pad_str(&iter_line.to_string(), line_no_width, ' ');
            print!("{}", code_color.bold().paint(format!("{} | ", line_no)));
            println!("{}", code_color.paint(str));
        }
    }
    // 提示信息
    if target.1 > 0 {
        let mut message_str = gen_space_str(target.1 + line_no_width + 3);
        message_str.push_str("^ ");
        message_str.push_str(message);
        println!("{}", primary_color.bold().paint(&message_str));
    }
    // 提示信息后面代码
    for (i, str) in after_lines.iter().enumerate() {
        iter_line += 1;
        if i < 2 {
            let line_no = pad_str(&iter_line.to_string(), line_no_width, ' ');
            print!("{}", code_color.bold().paint(format!("{} | ", line_no)));
            println!("{}", code_color.paint(str));
        } else {
            break;
        }
    }
}

// 打印警告代码帧信息
pub fn print_warn_frame(source: &str, position: usize, message: &str) {
    print_code_frame(source, position, message, CodeFrameMessageType::Warn)
}

// 打印错误代码帧信息
pub fn print_error_frame(source: &str, position: usize, message: &str) {
    print_code_frame(source, position, message, CodeFrameMessageType::Error)
}
