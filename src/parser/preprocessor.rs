pub fn strip_comments(s: String) -> String {
    let mut buf = String::with_capacity(s.len());
    let mut iter = s.chars();
    while let Some(ch) = iter.next() {
        match ch {
            ';' => {
                while let Some(ch) = iter.next() {
                    match ch {
                        ch if ch == '\n' => {
                            buf.push(ch);
                            break;
                        }
                        _ => (),
                    }
                }
            }
            ch => buf.push(ch),
        }
    }
    buf
}

pub fn process(s: String) -> String {
    let mut buf = String::with_capacity(s.len());
    let indentations = s.lines().map(|line| {
        let mut indents = 0;
        for ch in line.chars() {
            if ch.is_whitespace() {
                indents += 1;
            } else {
                break;
            }
        }
        indents
    });

    let lines = s.lines().map(|line| line.trim());

    let indented_lines: Vec<_> = lines
        .zip(indentations)
        .filter(|&(line, _)| !line.is_empty())
        .collect();

    let mut indent_layers: Vec<u32> = vec![];
    for (line, &(text, indent)) in indented_lines.iter().enumerate() {
        indent_layers.push(indent);
        buf.push_str(" (");
        buf.push_str(text);

        let next_indent = if line == indented_lines.len() - 1 {
            0
        } else {
            indented_lines[line + 1].1
        };

        let mut indent_layers2 = vec![];
        for &prev_indent in indent_layers.iter().rev() {
            if prev_indent >= next_indent {
                buf.push(')');
            } else {
                indent_layers2.push(prev_indent);
            }
        }
        indent_layers = indent_layers2;
    }

    buf
}
