// Simple calculator implementation
pub fn calculator_query(input: &str) -> Option<f32> {
    let Ok(postfix) = tokenize(input).and_then(infix_to_postfix)
    else {
        return None
    };
    // println!("Postfix: {}", postfix.join(" "));

    let result = evaluate_postfix(postfix)?;
    // println!("Result: {result}");
    Some(result)
}

// Convert input string into tokens of digits and operators
fn tokenize(input: &str) -> Result<Vec<&str>, &'static str> {
    let mut tokens = Vec::new();
    let mut chars = input.char_indices().peekable();

    // For each char
    while let Some((start, ch)) = chars.next() {
        // Ignore whitespace
        if ch.is_whitespace() {
            continue;
        }

        // Group consecutive digits into a number
        if ch.is_ascii_digit() {
            let mut end = start + ch.len_utf8();

            // Determine full number length
            while let Some(&(index, next)) = chars.peek() {
                if next.is_ascii_digit() {
                    chars.next();
                    end = index + next.len_utf8();
                } else {
                    break;
                }
            }

            // Add full number
            tokens.push(&input[start..end]);
            continue;
        }

        // Operators and parentheses
        if "+-*/^()".contains(ch) {
            let end = start + ch.len_utf8();
            tokens.push(&input[start..end]);
            continue;
        }

        return Err("Invalid character");
    }

    Ok(tokens)
}

// Return precedence of operators
fn precedence(op: &str) -> Option<u8> {
  match op {
      "+" | "-" => Some(1),
      "*" | "/" => Some(2),
      "^" => Some(3),
      _ => None,
  }
}

fn is_right_associative(op: &str) -> bool {
    op == "^"
}

fn infix_to_postfix(tokens: Vec<&str>) -> Result<Vec<&str>, &str> {
  let mut operator_stack: Vec<&str> = Vec::new();
  let mut output: Vec<&str> = Vec::new();

  for token in tokens {
        match token {
            "(" => {
                operator_stack.push(token);
            }
            ")" => {
                // Pop operators until the matching opening parenthesis.
                while let Some(&top) = operator_stack.last() {
                    if top == "(" {
                        break;
                    }
                    output.push(operator_stack.pop().unwrap());
                }

                // Ensure there was a matching opening parenthesis.
                if operator_stack.pop() != Some("(") {
                    return Err("Mismatched parentheses");
                }
            }

            op if precedence(op).is_some() => {
                let current_precedence = precedence(op).unwrap();

                // Pop operators with higher precedence, or equal
                // precedence when the current operator is left-associative.
                while let Some(&top) = operator_stack.last() {
                    let Some(top_precedence) = precedence(top) else {
                        // An opening parenthesis stops operator popping.
                        break;
                    };

                    let should_pop =
                        top_precedence > current_precedence
                        || (top_precedence == current_precedence
                            && !is_right_associative(op));

                    if !should_pop {
                        break;
                    }
                    output.push(operator_stack.pop().unwrap());
                }

                operator_stack.push(op);
            }

            // Operand (number, variable, etc.).
            operand => {
                output.push(operand);
            }
        }
    }

    // Pop remaining operators.
    while let Some(op) = operator_stack.pop() {
        if op == "(" {
            return Err("Mismatched parentheses");
        }

        output.push(op);
    }

    Ok(output)
}

fn evaluate_postfix(postfix: Vec<&str>) -> Option<f32> {
    let mut stack: Vec<f32> = Vec::new();

    for token in postfix {
        // Token is number
        if let Ok(num) = token.parse::<f32>() {
            stack.push(num);
        } else { // operator
            let b = stack.pop()?;
            let a = stack.pop()?;

            stack.push(match token {
                "+" => a + b,
                "-" => a - b,
                "*" => a * b,
                "/" => a / b,
                "^" => a.powf(b),
                _ => return None,
            });
        }
    }

    stack.pop()
}