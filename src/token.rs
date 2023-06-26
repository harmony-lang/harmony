use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

impl SourceLocation {
    pub fn merge(&self, other: &SourceLocation) -> SourceLocation {
        let mut location = self.clone();
        let length: i64 = (self.line + self.column + self.length) as i64
            - (other.line + other.column + other.length) as i64;
        if length > 0 {
            location.length = length as usize;
        }
        location
    }

    pub fn to_string(&self) -> String {
        format!(
            "{}:{}:{}",
            self.file.to_string(),
            self.line.to_string(),
            self.column.to_string()
        )
    }

    pub fn default() -> SourceLocation {
        SourceLocation {
            file: String::new(),
            line: 0,
            column: 0,
            length: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    Identifier,
    IntegerLiteral,
    FloatLiteral,
    StringLiteral,
    CharacterLiteral,
    BooleanLiteral,

    // Keywords
    Module,
    Import,
    As,
    Exposing,
    Extern,
    Enum,
    Fun,
    Case,
    Of,
    End,
    If,
    Then,
    Else,
    Let,
    In,

    // Types
    Int,
    Float,
    String,
    Char,
    Bool,
    Any,
    Unit,

    // Punctuation
    OpenParenthesis,  // (
    CloseParenthesis, // )
    OpenBracket,      // [
    CloseBracket,     // ]
    Dot,              // .
    DoubleDot,        // ..   (range)
    Comma,            // ,
    Colon,            // :
    DoubleColon,      // ::   (list constructor (cons))
    Pipe,             // |
    Arrow,            // ->
    FatArrow,         // =>

    // Operators
    Plus,              // +     (addition)
    PlusPlus,          // ++    (concatenation)
    Minus,             // -     (subtraction)
    Asterisk,          // *     (multiplication)
    Slash,             // /     (division)
    Percent,           // %     (modulo)
    Equals,            // =     (assignment)
    DoubleEquals,      // ==    (equality)
    NotEquals,         // /=    (inequality)
    LessThan,          // <     (less than)
    LessThanEquals,    // <=    (less than or equal to)
    GreaterThan,       // >     (greater than)
    GreaterThanEquals, // >=    (greater than or equal to)
    And,               // &&    (and)
    Or,                // ||    (or)
    Not,               // !     (not)

    // Special
    Newline,
    Whitespace,
    EndOfFile,
    Unknown,
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::Identifier => write!(f, "<identifier>"),
            TokenKind::IntegerLiteral => write!(f, "<integer literal>"),
            TokenKind::FloatLiteral => write!(f, "<float literal>"),
            TokenKind::StringLiteral => write!(f, "<string literal>"),
            TokenKind::CharacterLiteral => write!(f, "<character literal>"),
            TokenKind::BooleanLiteral => write!(f, "<boolean literal>"),
            TokenKind::Module => write!(f, "module"),
            TokenKind::Import => write!(f, "import"),
            TokenKind::As => write!(f, "as"),
            TokenKind::Exposing => write!(f, "exposing"),
            TokenKind::Extern => write!(f, "extern"),
            TokenKind::Enum => write!(f, "enum"),
            TokenKind::Fun => write!(f, "fun"),
            TokenKind::Case => write!(f, "case"),
            TokenKind::Of => write!(f, "of"),
            TokenKind::End => write!(f, "end"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Then => write!(f, "then"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::Let => write!(f, "let"),
            TokenKind::In => write!(f, "in"),
            TokenKind::Int => write!(f, "int"),
            TokenKind::Float => write!(f, "float"),
            TokenKind::String => write!(f, "string"),
            TokenKind::Char => write!(f, "char"),
            TokenKind::Bool => write!(f, "bool"),
            TokenKind::Any => write!(f, "any"),
            TokenKind::Unit => write!(f, "unit"),
            TokenKind::OpenParenthesis => write!(f, "("),
            TokenKind::CloseParenthesis => write!(f, ")"),
            TokenKind::OpenBracket => write!(f, "["),
            TokenKind::CloseBracket => write!(f, "]"),
            TokenKind::Dot => write!(f, "."),
            TokenKind::DoubleDot => write!(f, ".."),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::DoubleColon => write!(f, "::"),
            TokenKind::Pipe => write!(f, "|"),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::FatArrow => write!(f, "=>"),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::PlusPlus => write!(f, "++"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Asterisk => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::Equals => write!(f, "="),
            TokenKind::DoubleEquals => write!(f, "=="),
            TokenKind::NotEquals => write!(f, "/="),
            TokenKind::LessThan => write!(f, "<"),
            TokenKind::LessThanEquals => write!(f, "<="),
            TokenKind::GreaterThan => write!(f, ">"),
            TokenKind::GreaterThanEquals => write!(f, ">="),
            TokenKind::And => write!(f, "&&"),
            TokenKind::Or => write!(f, "||"),
            TokenKind::Not => write!(f, "!"),
            TokenKind::Newline => write!(f, "<newline>"),
            TokenKind::Whitespace => write!(f, "<whitespace>"),
            TokenKind::EndOfFile => write!(f, "<end of file>"),
            TokenKind::Unknown => write!(f, "<unknown>"),
        }
    }
}

impl TokenKind {
    pub fn is_binary_operator(&self) -> bool {
        match self {
            TokenKind::Plus
            | TokenKind::PlusPlus
            | TokenKind::Minus
            | TokenKind::Asterisk
            | TokenKind::Slash
            | TokenKind::Percent
            | TokenKind::DoubleEquals
            | TokenKind::NotEquals
            | TokenKind::LessThan
            | TokenKind::LessThanEquals
            | TokenKind::GreaterThan
            | TokenKind::GreaterThanEquals
            | TokenKind::And
            | TokenKind::Or => true,
            _ => false,
        }
    }

    pub fn is_unary_operator(&self) -> bool {
        match self {
            TokenKind::Minus | TokenKind::Not => true,
            _ => false,
        }
    }

    pub fn precedence(&self) -> u8 {
        match self {
            TokenKind::Or => 1,
            TokenKind::And => 2,
            TokenKind::DoubleEquals | TokenKind::NotEquals => 3,
            TokenKind::LessThan
            | TokenKind::LessThanEquals
            | TokenKind::GreaterThan
            | TokenKind::GreaterThanEquals => 4,
            TokenKind::Plus | TokenKind::Minus => 5,
            TokenKind::Asterisk | TokenKind::Slash | TokenKind::Percent => 6,
            TokenKind::PlusPlus => 7,
            _ => 0,
        }
    }
}
