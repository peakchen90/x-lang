// 将字符串码点转为 utf-16 编码
// see: https://datatracker.ietf.org/doc/html/rfc2781#section-2.1
pub fn encode_utf16_str(str: &str) -> Vec<u16> {
    let str_chars = str.chars().collect::<Vec<char>>();
    let mut result = vec![];
    for char in str_chars.iter() {
        let mut code = *char as u32;
        if code < 0x10000 {
            result.push(code as u16);
        } else {
            code -= 0x10000;
            // U' = yyyyyyyyyyxxxxxxxxxx (code binary)
            // W1 = 110110yyyyyyyyyy
            // W2 = 110111xxxxxxxxxx
            let mut w1: u16 = 0xD800 | ((code >> 10) as u16); // 取U'的高10位放至W1低位处
            let mut w2: u16 = 0xDC00 | (code as u16) & 0b1111111111; // 取U'的低10位放至W2低位处
            result.push(w1);
            result.push(w2);
        }
    }

    result
}

// TODO: 暂不能使用
// 将 utf-16 编码字符串解密成字符串
// see: https://datatracker.ietf.org/doc/html/rfc2781#section-2.2
/*pub fn decode_utf16_str(str_codes: Vec<u16>) -> String {
    let mut result = String::new();
    let mut i = 0;
    let mut w1;
    let mut w2;
    let mut code: u32;
    while i < str_codes.len() {
        w1 = str_codes[0];
        if w1 < 0xD800 || w1 > 0xDFFF {
            result.push(w1 as char);
        } else {
            if !(w1 >= 0xD800 && w1 <= 0xDBFF) {
                // Invalid: 兼容错误
                result.push(w1 as char);
                i += 1;
                continue;
            }

            let next = str_codes.get(i + 1);
            if next.is_none() {
                // Invalid: 兼容错误
                result.push(w1 as char);
                i += 1;
                continue;
            }

            w2 = *next.unwrap();
            if !(w2 >= 0xDC00 && w2 <= 0xDFFF) {
                // Invalid: 兼容错误
                result.push(w1 as char);
                result.push(w2 as char);
                i += 1;
                continue;
            }

            code = ((w1 & 0b1111111111) as u32) << 10; // w1低10位放入code的高位（共20位）
            code |= (w2 & 0b1111111111) as u32; // w2低10位放入code的低位（共20位）
            result.push(code as char);
        }
        i += 1;
    }

    result
}*/

// 返回 utf16 编码的字符码点集合，并以 0 作为结束符
pub fn get_string_utf16_chars(value: &str) -> Vec<u16> {
    let mut utf16_chars = encode_utf16_str(value);
    utf16_chars.push(0);
    utf16_chars
}
