//create structs that can break our language down into tokens
use std::collections::HashMap;
use std::io;
use thiserror::Error;
#[derive(Error,Debug)]
pub enum LexerError{
    #[error("IO Error")]
    FileIO(#[from] io::Error),
    #[error("Was expecting {expected:?},found {found:?}")]
    MissingExpectedSymbol{
        expected:TokenType,
        found:Token
    },
    #[error("Can't find opening symbol for {symbol:?}")]
    MisbalancedSymbol {
        symbol:String,
    },
    #[error("Can't find {symbol:?}")]
    UnknownSymbol {
        symbol:String,
    },
    #[error("Cant find numeric {raw:?}")]
    NumericLiteralInvalidChar{
        raw:String
    }


    
}
#[derive(Debug)]
pub enum NumericHint{
    Integer,
    FloatingPoint,
}
pub type Token = TokenType;
pub type BalancingDepthType = i32;
#[derive(Debug)]
pub enum TokenType{
    //End of token stream 
    EOF,

    //Operators are actions that we take on an entity * 
    Operator(String),
    //like println
    Identifier(String),
    //
    Char(char),
    //
    Numeric{
        raw:String,
        hint:NumericHint,
    },
    //
    Unknown(char),
    //Brackets
    Punctuation{
        raw:char,
        kind:PunctuationKind
    },
    
}
#[derive(Debug)]
pub enum PunctuationKind{
    //'( { [ 
    Open(usize),
    //]})
    Close(usize),
    // , ; 
    Separator
}

pub struct Lexer<'a>{
    pub line :usize,
    pub column:usize,
    
    //Raw format in terms of bytes
    pub codepoint_offset : usize,
    chars : std::iter::Peekable<std::str::Chars<'a>>,
    balancing_state: HashMap<char,i32>,
}

impl <'a> Lexer<'a>{
    pub fn new(chars:&'a str)->Lexer{
        Lexer{
            line:1,
            column:1,
            chars : chars.chars().peekable(),
            codepoint_offset:1,
            balancing_state : HashMap::new(),


        }
    }
    fn match_balance_character(c:&char)->char{
        return match c{
            '}'=>'{',
            ']'=>'[',
            ')'=>'(',
            _=>panic!("Fuck")
        };
    }
    fn push_open(&mut self,c:&char) -> BalancingDepthType{
        if let Some(v) = self.balancing_state.get_mut(&c){
            *v+=1;
            return *v;
        }else{
            self.balancing_state.insert(*c,1);
            return 0;
        }
    }
    fn pop_close(&mut self,c:&char) -> Result<BalancingDepthType,LexerError>{
        //match and reduce //TEST
        if let Some(v) = self.balancing_state.get_mut(&(Self::match_balance_character(&c))){
            if *v==1{
                *v-=1;
                return Ok(*v);
            }else{
                return Err(LexerError::MisbalancedSymbol{
                    symbol:String::from(*c)
            });
        }
    }else{
        return Err(LexerError::MisbalancedSymbol{
            symbol:String::from(*c)
    });
    }

}
    fn consume_digit(&mut self,raw:&str,for_radix:u32) -> Result<char,LexerError>{
        match self.chars.next(){
            Some(c) if !c.is_digit(for_radix)=>{
                return Err(LexerError::NumericLiteralInvalidChar { raw: raw.to_string() });
            },
            None =>{
                return Err(LexerError::NumericLiteralInvalidChar { raw: raw.to_string()});
            },
            Some(c)=> Ok(c)
        }
    }
    pub fn parse_integer(&mut self,c:char)->Result<TokenType,LexerError>{
        let mut seen_dot = false;
        let mut num = String::from(c);
        let mut seen_exp = false;
        //Make these mutable as and when required
        let radix = 10;
        let exp_radix = 10;
        if c == '.' {
            let s = self.consume_digit(&num,10)?;
            num.push(s);
            seen_dot=true;


        }
        loop{
            match self.chars.peek(){
                Some(v) if *v =='.' && !seen_dot && !seen_exp =>{
                    num.push(*v);
                    self.consume_char();
                    seen_dot = true;
                },
                Some(v) if *v == 'e' || *v == 'E' && !seen_exp =>{
                    num.push(*v);
                    self.consume_char();
                    seen_exp = true;
                    match self.chars.peek(){
                        Some(v) if *v =='+' || *v == '-'=>{
                            num.push(*v);
                            self.consume_char();
                        }
                        _ =>{}
                    }
                    //It shouldn't end with E or e
                    //x= 2e is invalid 
                    num.push(self.consume_digit(&num,exp_radix)?);
                },
                Some(v) if v.is_digit(radix) =>{
                    num.push(*v);
                    self.consume_char();
                },
                Some(v) if v.is_ascii_alphabetic() || c.is_digit(10)=>{
                    num.push(*v);
                    return Err(LexerError::NumericLiteralInvalidChar { raw:  num })
                },
                _ =>{
                    break Ok(TokenType::Numeric { raw: num, hint: if seen_dot || seen_exp {NumericHint::FloatingPoint} else {NumericHint::Integer} })
                }
            }
        }
        // let x = ;
    }
    //Function returns a Result<TokenType this is for converting my punctuations to a token
    pub fn transform_to_type(&mut self,c: char)->Result<TokenType,LexerError>{
        match c{
            '(' | '{'|'[' => Ok(TokenType::Punctuation { raw: c, kind: PunctuationKind::Open(self.push_open(&c).try_into().unwrap()) }),
            //This might be wrong
            ')' |']'|'}' => Ok(TokenType:: Punctuation{ raw:c, kind:PunctuationKind::Close(self.pop_close(&c)?.try_into().unwrap())}),
            '0'..='9' | '.' => Ok(self.parse_integer(c)?),
            _ => Err(LexerError::UnknownSymbol{symbol:c.to_string()})
        }
    }

    pub fn  consume_char(&mut self)->Option<char>{
        match self.chars.next(){
            Some(v)=>{
                self.column +=1;
                if v == '\n' {
                    self.line+=1;
                    self.column =1;
                }
                self.codepoint_offset+=1;
                return Some(v);
            }
            None =>None,
        }
    }
    fn skip_whitespace(&mut self){
        while let Some(c) = self.chars.peek(){
            if!c.is_whitespace(){
                break;
            }
            self.consume_char();
                

        }
    }
    pub fn next_token(&mut self)->Result<TokenType,LexerError>{
        self.skip_whitespace();
        if let Some(c) = self.consume_char(){
            return self.transform_to_type(c);
        }else{
            return Ok(TokenType::EOF);
        }
    }
}