#[derive(Debug, PartialEq, Eq)]
enum Token {
    Cry,
    Anger,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    MoveRight,
    MoveLeft,
    Increment,
    Decrement,
    Output,
    Input,
    LoopStart,
    LoopEnd,
}

pub fn compile(source: &str) -> Result<Vec<Instruction>, String> {
    let tokens = tokenize(source)?;
    decode(&tokens)
}

fn tokenize(source: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut line = 1;
    let mut column = 0;

    for character in source.chars() {
        if character == '\n' {
            line += 1;
            column = 0;
            continue;
        }

        column += 1;

        match character {
            '😭' => tokens.push(Token::Cry),
            '💢' => tokens.push(Token::Anger),
            character if character.is_whitespace() => {}
            _ => {
                return Err(format!(
                    "invalid character '{character}' at line {line}, column {column}; only 😭, 💢, and whitespace are allowed"
                ));
            }
        }
    }

    Ok(tokens)
}

fn decode(tokens: &[Token]) -> Result<Vec<Instruction>, String> {
    if tokens.len() % 3 != 0 {
        return Err(format!(
            "program has {} tokens; instructions require complete groups of three",
            tokens.len()
        ));
    }

    let instructions = tokens
        .chunks_exact(3)
        .map(|group| match group {
            [Token::Cry, Token::Cry, Token::Anger] => Instruction::MoveRight,
            [Token::Cry, Token::Cry, Token::Cry] => Instruction::MoveLeft,
            [Token::Cry, Token::Anger, Token::Anger] => Instruction::Increment,
            [Token::Cry, Token::Anger, Token::Cry] => Instruction::Decrement,
            [Token::Anger, Token::Cry, Token::Anger] => Instruction::Input,
            [Token::Anger, Token::Cry, Token::Cry] => Instruction::Output,
            [Token::Anger, Token::Anger, Token::Anger] => Instruction::LoopStart,
            [Token::Anger, Token::Anger, Token::Cry] => Instruction::LoopEnd,
            _ => unreachable!(),
        })
        .collect();

    Ok(instructions)
}
